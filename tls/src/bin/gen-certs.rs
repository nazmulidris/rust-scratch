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

use crossterm::style::Stylize as _;
use r3bl_core::ok;
use std::{env, path::PathBuf, process::Command};
use strum_macros::{Display, EnumString};
use tls::scripting::{
    self,
    directory::{self, MkdirOptions},
    download::download_file_overwrite_existing,
    environment, github_api,
    path::{self},
    permissions,
};

#[derive(Display, EnumString)]
pub enum FolderLocations {
    #[strum(serialize = "bin")]
    BinFolder,

    #[strum(serialize = "generated")]
    GeneratedFolder,
}

#[derive(Display, EnumString)]
pub enum BinaryFileName {
    #[strum(serialize = "cfssl")]
    CFSSL,

    #[strum(serialize = "cfssljson")]
    CFSSLJSON,

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[strum(serialize = "linux_amd64")]
    OsArch,

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[strum(serialize = "darwin_arm64")]
    OsArch,

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    #[strum(serialize = "windows_amd64.exe")]
    OsArch,
}

#[derive(Display, EnumString)]
pub enum GithubLocation {
    #[strum(serialize = "cloudflare")]
    Org,
    #[strum(serialize = "cfssl")]
    Repo,
}

impl TryFrom<FolderLocations> for PathBuf {
    type Error = miette::Error;

    /// Returns the fully qualified path of the location. Eg: not just `bin`, but
    /// `/home/user/some/path/bin`, where the prefix or root of this path is the current
    /// working directory of the process.
    fn try_from(value: FolderLocations) -> Result<Self, Self::Error> {
        let path = path::fq_path_relative_to_pwd(value.to_string())?;
        ok!(path)
    }
}

fn main() -> miette::Result<()> {
    // Setup tracing.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .compact()
        .without_time()
        .init();

    tracing::debug!("pwd at start" = ?path::pwd());

    // Add to PATH.
    let my_path = environment::get_path_prefixed(PathBuf::try_from(FolderLocations::BinFolder)?)?;

    tracing::debug!("my_path" = %format!("{:.50}{}", my_path, "...<clip>".red()));

    // 00: remove comments & write rust version of that script
    // If cfssl or cfssljson files do not exist, download them.
    // Command::new("fish")
    //     .env("PATH", my_path)
    //     .arg("get-cfssl-binaries.fish")
    //     .status()
    //     .expect("Failed to execute get-cfssl-binaries.fish");

    // If `generated` directory exists, delete it. And create a new one.
    scripting::directory::mkdir(
        PathBuf::try_from(FolderLocations::GeneratedFolder)?,
        MkdirOptions::CreateIntermediateFoldersAndPurgeExisting,
    )?;

    // Generate CA and server certificates in the `generated` directory.
    let dir_stack = &mut scripting::directory_stack::Stack::get_mut_singleton()?;
    _ = dir_stack
        .lock()
        .unwrap()
        .pushd(PathBuf::try_from(FolderLocations::GeneratedFolder)?)?;

    tracing::debug!("pwd after pushd" = ?path::pwd());

    // 00: remove comments below
    // generate_certs();

    // 00: remove comments below
    // display_status();

    _ = dir_stack.lock().unwrap().popd()?;

    tracing::debug!("pwd after popd" = ?path::pwd());

    ok!()
}

async fn download_cfssl_binaries() -> miette::Result<()> {
    // If cfssl or cfssljson files do not exist, download them.
    let (bin_folder, cfssl_file, cfssljson_file) = {
        let bin_folder = PathBuf::try_from(FolderLocations::BinFolder)?;
        let cfssl_file = bin_folder.join(BinaryFileName::CFSSL.to_string());
        let cfssljson_file = bin_folder.join(BinaryFileName::CFSSLJSON.to_string());
        (bin_folder, cfssl_file, cfssljson_file)
    };

    // Early return if binaries already exist.
    if cfssl_file.exists() && cfssljson_file.exists() {
        println!(
            "🎉 {} and {} binaries already exist.",
            cfssl_file.display().to_string().magenta(),
            cfssljson_file.display().to_string().magenta()
        );
        return ok!();
    }

    println!("📦 Downloading binaries...");
    directory::mkdir(
        bin_folder,
        MkdirOptions::CreateIntermediateFoldersAndPurgeExisting,
    )?;

    // Get latest release tag for the binaries from their GitHub repo.
    let (cfssl_bin_url, cfssljson_bin_url) = {
        let org = &GithubLocation::Org.to_string();
        let repo = &GithubLocation::Repo.to_string();
        let release_version = github_api::get_latest_release_tag_from_github(org, repo).await?;
        let root_url = format!("https://github.com/{}/{}/releases/download", org, repo);
        let cfssl_bin_url = format!(
            "{}/v{}/{}_{}_{}",
            root_url,
            release_version,
            BinaryFileName::CFSSL,
            release_version,
            BinaryFileName::OsArch
        );
        let cfssljson_bin_url = format!(
            "{}/v{}/{}_{}_{}",
            root_url,
            release_version,
            BinaryFileName::CFSSLJSON,
            release_version,
            BinaryFileName::OsArch
        );
        (cfssl_bin_url, cfssljson_bin_url)
    };

    // Print variables.
    println!("💾 {}: {}", stringify!(cfssl_bin_url), cfssl_bin_url);
    println!("💾 {}: {}", stringify!(cfssljson), cfssljson_bin_url);

    // Download binaries.
    download_file_overwrite_existing(&cfssl_bin_url, &cfssl_file).await?;
    download_file_overwrite_existing(&cfssljson_bin_url, &cfssljson_file).await?;

    // Make them executable.
    permissions::set_file_executable(&cfssl_file)?;
    permissions::set_file_executable(&cfssljson_file)?;

    // Display success message.
    println!(
        "🎉 Downloaded {} and {} executable binaries to: {}",
        &cfssl_file.display().to_string().magenta(),
        &cfssljson_file.display().to_string().magenta(),
        path::pwd()?.display().to_string().magenta()
    );

    ok!()
}

fn generate_certs() {
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
}

fn display_status() {
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
}
