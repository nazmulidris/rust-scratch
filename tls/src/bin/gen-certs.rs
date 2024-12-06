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
use std::{
    env,
    process::Command,
    sync::{Arc, Mutex},
};
use strum_macros::{Display, EnumString};
use tls::{
    directory_path::try_pwd,
    directory_stack::DirStack,
    github_api::{Separator, UrlBuilder},
    path, paths_exist,
    scripting::{
        directory_create::{self, MkdirOptions},
        directory_path::{self},
        download::try_download_file_overwrite_existing,
        environment, github_api, permissions,
    },
};

pub mod constants {
    pub const CERTS_DIR: &str = "certs";
    pub const BIN_DIR: &str = "bin";
    pub const GENERATED_DIR: &str = "generated";

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
    // Setup tracing.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .compact()
        .without_time()
        .init();

    tracing::debug!("pwd at start" = ?directory_path::try_pwd());

    // Add to PATH (./certs/bin).
    let path_to_cfssl_bin = path!( with_root: try_pwd()?, CERTS_DIR, BIN_DIR );
    let my_path = environment::try_get_path_prefixed(path_to_cfssl_bin)?;

    let dir_stack = &mut DirStack::get_mut_singleton()?;

    // 00: remove comments & write rust version of that script
    download_cfssl_binaries(dir_stack).await?;

    // 00: remove comments below
    // generate_certs_using_cfssl_bin(dir_stack, &my_path)?;

    // 00: remove comments below
    // display_status_using_openssl_bin(dir_stack, &my_path)?;

    ok!()
}

// 00: ensure this function works
async fn download_cfssl_binaries(dir_stack: &mut Arc<Mutex<DirStack>>) -> miette::Result<()> {
    let bin_folder = path!(new: CERTS_DIR, BIN_DIR);
    with!(
        &bin_folder,
        as root,
        run {
            // Early return if the `certs/bin` directory & files exist.
            let cfssl_file = path!(with_root: root, CFSSL_BIN);
            let cfssljson_file = path!(with_root: root, CFSSLJSON_BIN);
            if paths_exist!(&root, &cfssl_file, &cfssljson_file) {
                println!(
                    "🎉 {} and {} binaries already exist.",
                    cfssl_file.display().to_string().magenta(),
                    cfssljson_file.display().to_string().magenta()
                );
                return ok!();
            };
        }
    );

    // Create a new `certs/bin` directory.
    directory_create::try_mkdir(
        &bin_folder,
        MkdirOptions::CreateIntermediateFoldersAndPurgeExisting,
    )?;

    // Pushd into the `certs/bin` directory.
    let (
        /* this gets dropped immediately, doesn't bind */ _,
        /* this gets dropped like a "normal" variable" */ _dir_stack_drop_handle,
    ) = dir_stack.lock().unwrap().try_pushd(&bin_folder)?;

    // Try to get latest release tag for the binaries from their GitHub repo.
    let (cfssl_bin_url, cfssljson_bin_url) = {
        let org = &GithubLocation::Org.to_string();
        let repo = &GithubLocation::Repo.to_string();
        let release_version =
            &github_api::try_get_latest_release_tag_from_github(org, repo).await?;

        let root_url = UrlBuilder::default()
            + "https://github.com/"
            + org
            + Separator::ForwardSlash
            + repo
            + "/releases/download";

        let cfssl_bin_url = &root_url
            + "/v"
            + release_version
            + Separator::ForwardSlash
            + CFSSL_BIN
            + Separator::Underscore
            + release_version
            + Separator::Underscore
            + OS_ARCH;

        let cfssljson_bin_url = &root_url
            + "/v"
            + release_version
            + Separator::ForwardSlash
            + CFSSLJSON_BIN
            + Separator::Underscore
            + release_version
            + Separator::Underscore
            + OS_ARCH;

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
        CFSSL_BIN.magenta(),
        CFSSLJSON_BIN.magenta(),
        directory_path::try_pwd()?.display().to_string().magenta()
    );

    ok!()
}

fn generate_certs_using_cfssl_bin(
    dir_stack: &mut Arc<Mutex<DirStack>>,
    my_path: &str,
) -> miette::Result<()> {
    // Generate CA and server certificates in the `generated` directory.
    _ = dir_stack.lock().unwrap().try_pushd(GENERATED_DIR)?;

    tracing::debug!("pwd after pushd" = ?directory_path::try_pwd());

    // Generate root certificate (CA) and sign it.
    Command::new("cfssl")
        .args(["gencert", "-initca", "../config/ca-csr.json"])
        .output()
        .expect("Failed to execute cfssl gencert for CA");

    Command::new("cfssljson")
        .args(["-bare", "ca"])
        .output()
        .expect("Failed to execute cfssljson for CA");

    // Generate server certificate (and private key) and sign it with the CA.
    Command::new("cfssl")
        .args([
            "gencert",
            "-ca",
            "ca.pem",
            "-ca-key",
            "ca-key.pem",
            "-config",
            "../config/ca-config.json",
            "-profile",
            "server",
            "../config/server-csr.json",
        ])
        .output()
        .expect("Failed to execute cfssl gencert for server");

    Command::new("cfssljson")
        .args(["-bare", "server"])
        .output()
        .expect("Failed to execute cfssljson for server");

    ok!()
}

// 00: if openssl is not installed, then handle install it using brew (add to scripting.rs)
fn display_status_using_openssl_bin(
    dir_stack: &mut Arc<Mutex<DirStack>>,
    my_path: &str,
) -> miette::Result<()> {
    // Generate CA and server certificates in the `generated` directory.
    _ = dir_stack.lock().unwrap().try_pushd(GENERATED_DIR)?;

    tracing::debug!("pwd after pushd" = ?directory_path::try_pwd());

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
}
