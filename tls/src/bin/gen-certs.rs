/*
 *   Copyright (c) 2024 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use constants::*;

use crossterm::style::Stylize as _;
use r3bl_core::{ok, with};
use std::{env, path::Path, process::Command};
use strum_macros::{Display, EnumString};
use tls::{
    command,
    command_runner::pipe,
    directory_change,
    environment::{EnvKeys, EnvVarsSlice},
    fs_path::{self, try_pwd},
    fs_paths, fs_paths_exist,
    scripting::{
        directory_create::{self, MkdirOptions},
        download::try_download_file_overwrite_existing,
        environment, github_api, permissions,
    },
    tracing_debug,
    tracing_debug_helper::{self, constants::TRACING_MSG_WIDTH, truncate_or_pad_from_left},
    with_saved_pwd,
};

pub mod constants {
    pub const CERTS_DIR: &str = "certs";
    pub const BIN_DIR: &str = "bin";
    pub const GENERATED_DIR: &str = "generated";
    pub const CONFIG_DIR: &str = "config";

    pub const CONFIG_FILE_CA: &str = "ca-config.json";
    pub const CONFIG_FILE_CA_CSR: &str = "ca-csr.json";
    pub const CONFIG_FILE_SERVER_CSR: &str = "server-csr.json";

    pub const CONFIG_VALUE_CA_CN: &str = "ca";
    pub const CONFIG_VALUE_SERVER_CN: &str = "server";

    pub const CA_PEM_FILE: &str = "ca.pem";
    pub const CA_KEY_PEM_FILE: &str = "ca-key.pem";
    pub const SERVER_PEM_FILE: &str = "server.pem";

    pub const CFSSL_BIN: &str = "cfssl";
    pub const CFSSLJSON_BIN: &str = "cfssljson";

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    pub const OS_ARCH: &str = "linux_amd64";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub const OS_ARCH: &str = "darwin_arm64";
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    pub const OS_ARCH: &str = "windows_amd64.exe";
}

#[derive(Display, EnumString)]
pub enum GithubLocation {
    #[strum(serialize = "cloudflare")]
    Org,
    #[strum(serialize = "cfssl")]
    Repo,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    // Ensure that the current working directory is the `tls` crate.
    let root_dir = {
        let it = try_pwd()?;
        if !it.ends_with("tls") {
            miette::bail!("You might be in the wrong folder; please run this in the root folder of the `tls` crate");
        };
        it
    };

    // Setup tracing.
    tracing_debug_helper::tracing_init();

    tracing_debug!("pwd at start", fs_path::try_pwd());

    // Add to PATH: "(realpath(.)/certs/bin)"
    let amended_path_envs = {
        let amended_env_path = {
            let fq_pwd = try_pwd()?;
            let path_to_cfssl_bin = tls::fs_paths!(with_root: fq_pwd => CERTS_DIR => BIN_DIR);
            environment::try_get_path_prefixed(path_to_cfssl_bin)?
        };
        environment::get_env_vars(EnvKeys::Path, &amended_env_path)
    };

    download_cfssl_binaries_if_needed(&root_dir).await?;
    generate_certs_using_cfssl_bin(&root_dir, &amended_path_envs)?;
    install_openssl_if_needed();
    display_status_using_openssl_bin(&root_dir, &amended_path_envs)?;

    tracing_debug!("pwd at end", fs_path::try_pwd());

    ok!()
}

// 00: make sure this works; parameterize everything & use constants mod above
fn generate_certs_using_cfssl_bin(
    root_dir: &Path,
    amended_path_envs: EnvVarsSlice,
) -> miette::Result<()> {
    tracing_debug!("generate certs", fs_path::try_pwd());
    // Pushd into the `certs/generated` directory. Generate CA and server certificates.
    with_saved_pwd!({
        let generated_dir = fs_paths!(with_root: root_dir => CERTS_DIR => GENERATED_DIR);
        let generated_dir_display_string =
            truncate_or_pad_from_left(&generated_dir.display().to_string(), TRACING_MSG_WIDTH)
                .magenta();

        // Create the generated directory if it does not exist.
        directory_create::try_mkdir(&generated_dir, MkdirOptions::CreateIntermediateDirectories)?;
        directory_change::try_cd(&generated_dir)?;

        // Generate root certificate (CA) and sign it.
        //
        // Creates the following files:
        // - ca.csr: certificate signing request
        // - ca-key.pem: private key
        // - ca.pem: public key; used in the Rust client code
        pipe(
            &mut command!(
                program => CFSSL_BIN,
                envs => amended_path_envs,
                args => "gencert",
                        "-initca", fs_paths!(with_root: root_dir => CERTS_DIR => CONFIG_DIR => CONFIG_FILE_CA_CSR),
            ),
            &mut command!(
                program => CFSSLJSON_BIN,
                envs => amended_path_envs,
                args => "-bare", CONFIG_VALUE_CA_CN,
            ),
        )?;

        println!(
            "🎉 Generated CA certificate & key in {}",
            generated_dir_display_string.clone().magenta()
        );

        // Generate server certificate (and private key) and sign it with the CA.
        //
        // Arguments:
        // - `-config ../ca-config.json` is the configuration file, which contains lifetimes for
        //   the certificates.
        // - `-profile server` is from `ca-config.json`
        //
        // Generates the following files:
        // - server.csr: certificate signing request
        // - server-key.pem: private key; used in the Rust server code
        // - server.pem: public key; used in the Rust server code
        pipe(
            &mut command!(
                program => CFSSL_BIN,
                envs => amended_path_envs,
                args => "gencert",
                        "-ca", CA_PEM_FILE,
                        "-ca-key", CA_KEY_PEM_FILE,
                        "-config", fs_paths!(with_root: root_dir => CERTS_DIR => CONFIG_DIR => CONFIG_FILE_CA),
                        "-profile", CONFIG_VALUE_SERVER_CN, fs_paths!(with_root: root_dir => CERTS_DIR => CONFIG_DIR => CONFIG_FILE_SERVER_CSR),
            ),
            &mut command!(
                program => CFSSLJSON_BIN,
                envs => amended_path_envs,
                args => "-bare", CONFIG_VALUE_SERVER_CN,
            ),
        )?;
        println!(
            "🎉 Generated server certificate (issued by CA) & key in {}",
            generated_dir_display_string.clone().magenta()
        );

        ok!()
    })
}

// 00: get this working
fn display_status_using_openssl_bin(
    root_dir: &Path,
    amended_path_envs: EnvVarsSlice,
) -> miette::Result<()> {
    with_saved_pwd!({
        // Pushd into the `certs/generated` directory. Generate CA and server certificates.
        directory_change::try_cd(fs_paths!(with_empty_root => CERTS_DIR => GENERATED_DIR))?;

        println!(
            "\x1b[32m🎉 Generated certificates in the \x1b[33m{}\x1b[0m directory.",
            env::current_dir().unwrap().display()
        );
        println!("\x1b[34m🔍 Verifying certificates...\x1b[0m");

        Command::new("openssl")
            .args(["x509", "-noout", "-text", "-in", "ca.pem"])
            .status()
            .expect("Failed to execute openssl for CA");

        Command::new("openssl")
            .args(["x509", "-noout", "-text", "-in", "server.pem"])
            .status()
            .expect("Failed to execute openssl for server");

        let status = Command::new("openssl")
            .args(["verify", "-CAfile", "ca.pem", "server.pem"])
            .status()
            .expect("Failed to execute openssl verify");

        if status.success() {
            println!("\x1b[32m🎉 Certificates are valid\x1b[0m");
        } else {
            println!("\x1b[31m❗ Certificates are invalid\x1b[0m");
        }

        ok!()
    })
}

// 00: if openssl is not installed, then handle install it using brew (add to scripting.rs)
fn install_openssl_if_needed() {}

async fn download_cfssl_binaries_if_needed(root_dir: &Path) -> miette::Result<()> {
    tracing_debug!("download binaries", fs_path::try_pwd());
    with_saved_pwd!({
        let bin_folder = fs_paths!(with_root: root_dir => CERTS_DIR => BIN_DIR);
        with!(
            &bin_folder,
            as root,
            run {
                // Early return if the `certs/bin` directory & files exist.
                let cfssl_file = fs_paths!(with_root: root => CFSSL_BIN);
                let cfssljson_file = fs_paths!(with_root: root => CFSSLJSON_BIN);
                if fs_paths_exist!(&root, &cfssl_file, &cfssljson_file) {
                    let cfssl_file_trunc_left =
                        truncate_or_pad_from_left(&cfssl_file.display().to_string(), TRACING_MSG_WIDTH);
                    let cfssljson_file_trunc_left =
                        truncate_or_pad_from_left(&cfssljson_file.display().to_string(), TRACING_MSG_WIDTH);
                    println!(
                        "🎉 {} and {} binaries already exist.",
                        cfssl_file_trunc_left.magenta(),
                        cfssljson_file_trunc_left.magenta(),
                    );
                    return ok!();
                };
            }
        );

        // Create a new `certs/bin` directory.
        directory_create::try_mkdir(
            &bin_folder,
            MkdirOptions::CreateIntermediateDirectoriesAndPurgeExisting,
        )?;

        // Pushd into the `certs/bin` directory.
        directory_change::try_cd(bin_folder)?;

        // Try to get latest release tag for the binaries from their GitHub repo.
        let (cfssl_bin_url, cfssljson_bin_url) = {
            let org = &GithubLocation::Org.to_string();
            let repo = &GithubLocation::Repo.to_string();
            let ver = &github_api::try_get_latest_release_tag_from_github(org, repo).await?;

            let root = format!(
                "https://github.com/{org}/{repo}/releases/download",
                org = org,
                repo = repo
            );

            let cfssl_bin_url = format!(
                "{root}/v{ver}/{bin}_{ver}_{os}",
                root = root,
                ver = ver,
                bin = CFSSL_BIN,
                os = OS_ARCH
            );

            let cfssljson_bin_url = format!(
                "{root}/v{ver}/{bin}_{ver}_{os}",
                root = root,
                ver = ver,
                bin = CFSSLJSON_BIN,
                os = OS_ARCH
            );

            // Print the latest URLs of the binaries.
            println!("🌐 URLs of latest versions of binaries...");
            println!("💾 {}: {}", stringify!(cfssl_bin_url), cfssl_bin_url);
            println!(
                "💾 {}: {}",
                stringify!(cfssljson_bin_url),
                cfssljson_bin_url
            );

            (cfssl_bin_url.to_string(), cfssljson_bin_url.to_string())
        };

        // Download the binaries into the `certs/bin` directory.
        println!(
            "📦 Downloading binaries {}, {} to {} ...",
            CFSSL_BIN.blue(),
            CFSSLJSON_BIN.blue(),
            try_pwd()?.display().to_string().magenta()
        );

        try_download_file_overwrite_existing(&cfssl_bin_url, CFSSL_BIN).await?;
        try_download_file_overwrite_existing(&cfssljson_bin_url, CFSSLJSON_BIN).await?;

        // Make them executable.
        permissions::try_set_file_executable(CFSSL_BIN)?;
        permissions::try_set_file_executable(CFSSLJSON_BIN)?;

        // Display success message.
        println!(
            "🎉 Downloaded {} and {} executable binaries to: {}",
            CFSSL_BIN.blue(),
            CFSSLJSON_BIN.blue(),
            fs_path::try_pwd()?.display().to_string().magenta()
        );

        ok!()
    })
}
