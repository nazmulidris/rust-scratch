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
use r3bl_tui::script::environment::{self, EnvVarsSlice};
use r3bl_tui::script::{
    apt_install::{check_if_package_is_installed, install_package},
    github_api,
};
use r3bl_tui::script::{
    directory_change,
    directory_create::{self, MkdirOptions},
    permissions, pipe, try_download_file_overwrite_existing, try_pwd, Run as _,
};
use r3bl_tui::{command, fs_path, fs_paths, fs_paths_exist, with_saved_pwd};
use r3bl_tui::{ok, truncate_from_left, with};
use r3bl_tui::{try_initialize_logging_global, DisplayPreference};
use std::path::Path;
use strum_macros::{Display, EnumString};

mod constants {
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
    pub const OPENSSL_BIN: &str = "openssl";

    pub const FIELD_OUTPUT_DISPLAY_WIDTH: usize = 50;

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
    try_initialize_logging_global(DisplayPreference::Stdout)?;

    // % is Display, ? is Debug.
    tracing::debug!("pwd at start" = ?fs_path::try_pwd());

    // Add to PATH: "(realpath(.)/certs/bin)"
    let amended_path_envs = {
        let amended_path_value = {
            let fq_pwd = try_pwd()?;
            let path_to_cfssl_bin = fs_paths!(with_root: fq_pwd => CERTS_DIR => BIN_DIR);
            environment::try_get_path_prefixed(path_to_cfssl_bin)?
        };
        environment::gen_path_env_vars(&amended_path_value)
    };

    download_cfssl_binaries_if_needed(&root_dir).await?;
    generate_certs_using_cfssl_bin(&root_dir, &amended_path_envs).await?;
    install_openssl_if_needed().await?;
    display_status_using_openssl_bin(&root_dir, &amended_path_envs).await?;

    // % is Display, ? is Debug.
    tracing::debug!("pwd at end" = ?fs_path::try_pwd());

    ok!()
}

async fn install_openssl_if_needed() -> miette::Result<()> {
    // % is Display, ? is Debug.
    tracing::debug!("install openssl if needed" = ?fs_path::try_pwd());
    if check_if_package_is_installed(OPENSSL_BIN).await? {
        println!("🎉 {} is already installed.", OPENSSL_BIN.blue());
    } else {
        //install using install_package()
        println!("📦 Installing {} using apt...", OPENSSL_BIN.blue());
        install_package(OPENSSL_BIN).await?;
        println!("🎉 {} installed successfully.", OPENSSL_BIN.blue());
    }
    ok!()
}

async fn generate_certs_using_cfssl_bin(
    root_dir: &Path,
    amended_path_envs: EnvVarsSlice<'_>,
) -> miette::Result<()> {
    // % is Display, ? is Debug.
    tracing::debug!("generate certs" = ?fs_path::try_pwd());
    // Pushd into the `certs/generated` directory. Generate CA and server certificates.
    with_saved_pwd!({
        let generated_dir = fs_paths!(with_root: root_dir => CERTS_DIR => GENERATED_DIR);
        let generated_dir_display_string = truncate_from_left(
            &generated_dir.display().to_string(),
            FIELD_OUTPUT_DISPLAY_WIDTH,
            false,
        );

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
        ).await?;

        println!(
            "🎉 Generated CA certificate & key in {}",
            generated_dir_display_string.magenta()
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
        ).await?;
        println!(
            "🎉 Generated server certificate (issued by CA) & key in {}",
            generated_dir_display_string.clone().magenta()
        );

        ok!()
    })
}

async fn display_status_using_openssl_bin(
    root_dir: &Path,
    amended_path_envs: EnvVarsSlice<'_>,
) -> miette::Result<()> {
    // % is Display, ? is Debug.
    tracing::debug!("verify certificates" = ?fs_path::try_pwd());
    with_saved_pwd!({
        // Pushd into the `certs/generated` directory. Generate CA and server certificates.
        directory_change::try_cd(fs_paths!(with_root: root_dir => CERTS_DIR => GENERATED_DIR))?;

        // Display CA certificate.
        let ca_cert_bytes = command!(
            program => OPENSSL_BIN,
            envs => amended_path_envs,
            args => "x509",
                    "-noout",
                    "-text",
                    "-in", CA_PEM_FILE,
        )
        .run()
        .await?;
        println!(
            "🎉 CA certificate size: {} bytes",
            ca_cert_bytes.len().to_string().blue()
        );

        // Display server certificate.
        let server_cert_bytes = command!(
            program => OPENSSL_BIN,
            envs => amended_path_envs,
            args => "x509",
                    "-noout",
                    "-text",
                    "-in", SERVER_PEM_FILE,
        )
        .run()
        .await?;
        println!(
            "🎉 Server certificate size: {} bytes",
            server_cert_bytes.len().to_string().blue()
        );

        // Verify that the server certificate is signed by the CA.
        _ = command!(
            program => OPENSSL_BIN,
            envs => amended_path_envs,
            args => "verify",
                    "-CAfile", CA_PEM_FILE,
                    SERVER_PEM_FILE,
        )
        .run()
        .await?;
        println!(
            "🎉 Server certificate is signed by CA {}",
            "verified".green()
        );

        ok!()
    })
}

async fn download_cfssl_binaries_if_needed(root_dir: &Path) -> miette::Result<()> {
    // % is Display, ? is Debug.
    tracing::debug!("download binaries" = ?fs_path::try_pwd());
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
                        truncate_from_left(&cfssl_file.display().to_string(), FIELD_OUTPUT_DISPLAY_WIDTH, false);
                    let cfssljson_file_trunc_left =
                        truncate_from_left(&cfssljson_file.display().to_string(), FIELD_OUTPUT_DISPLAY_WIDTH, false);
                    println!(
                        "🎉 binaries already exist: \n✅ {}\n✅ {}",
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
