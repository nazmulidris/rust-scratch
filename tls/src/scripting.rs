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
use miette::Diagnostic;
use miette::IntoDiagnostic;
use r3bl_core::ok;
use std::{
    env,
    fmt::Display,
    fs,
    io::{ErrorKind, Write as _},
    ops::{AddAssign, Deref},
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
};
use strum_macros::{Display, EnumString};
use thiserror::Error;

// 00: add brew_install mod
// 00: add apt_install mod

/// This macro to create a [std::process::Command] that receives a set of arguments and
/// returns it.
///
/// # Example of command and args
///
/// ```
/// use tls::create_command;
/// use std::process::Command;
///
/// let mut command = create_command!(
///     command => "echo",
///     args => "Hello, world!",
/// );
/// let output = command.output().expect("Failed to execute command");
/// assert!(output.status.success());
/// assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, world!\n");
/// ```
///
/// # Example of command, env, and args
///
/// ```
/// use tls::create_command;
/// use tls::environment;
/// use std::process::Command;
///
/// let my_path = "/usr/bin";
/// let env_vec = environment::get_path_envs(my_path);
/// let mut command = create_command!(
///     command => "printenv",
///     envs => env_vec,
///     args => "PATH",
/// );
/// let output = command.output().expect("Failed to execute command");
/// assert!(output.status.success());
/// assert_eq!(String::from_utf8_lossy(&output.stdout), "/usr/bin\n");
/// ```
///
/// # Example of command, env, args, and stdin
///
/// ```
/// use tls::create_command;
/// use tls::environment;
/// use std::process::{Command, Stdio};
///
/// let my_path = "/usr/bin";
/// let env_vec = environment::get_path_envs(my_path);
/// let mut command = create_command!(
///     command => "cat",
///     envs => env_vec,
///     stdin => Stdio::piped(),
///     args => "/etc/passwd",
/// );
/// let output = command.output().expect("Failed to execute command");
/// assert!(output.status.success());
/// assert!(!output.stdout.is_empty());
/// ```
#[macro_export]
macro_rules! create_command {
    // Variant that receives a command and args.
    (command=> $cmd:expr, args=> $($args:expr),* $(,)?) => {{
        let mut it = std::process::Command::new($cmd);
        $(
            it.arg($args);
        )*
        it
    }};

    // Variant that receives a command, env (vec), and args.
    (command=> $cmd:expr, envs=> $envs:expr, args=> $($args:expr),* $(,)?) => {{
        let mut it = std::process::Command::new($cmd);
        it.envs($envs);
        $(
            it.arg($args);
        )*
        it
    }};

    // Variant that receives a command, env (vec), piped stdin, and args.
    (command=> $cmd:expr, envs=> $envs:expr, stdin=> $stdin:expr, args=> $($args:expr),* $(,)?) => {{
        let mut it = std::process::Command::new($cmd);
        it.envs($envs);
        it.stdin($stdin);
        $(
            it.arg($args);
        )*
        it
    }};
}

/// Use this macro instead of [tracing::debug!] to make the output easier to read.
///
/// - It simply applies a display width to the message [debug::TRACING_MSG_WIDTH]
///   characters).
/// - This ensures that the first message is always this width, its clipped if too long,
///   and padded with spaces if too short.
///
/// # Arguments
/// 1. The first argument is the message that will be displayed. This can be any type that
///    implements the [Display] trait.
/// 2. The second argument is the body of the message. This can be any type that
///    implements the [Debug] trait.
///
/// More info: <https://doc.rust-lang.org/std/fmt/index.html>
///
/// This works hand in hand with [debug::tracing_init] to ensure that the output is
/// formatted with minimal noise.
///
/// # Example
///
/// ```
/// use tls::tracing_debug;
/// tracing_debug!(
///     "Hello, wor .. 20 ch!", // Must implement Display trait.
///     "Body has more space .... will be clipped to 50 ch!" // Must implement Debug trait.
/// );
/// ```
///
/// Here's what the [tracing::debug!] macro looks like:
///
/// ```
/// use tracing::debug;
/// tracing::debug!("{:10} = {:20}", "bar", "donkey");
/// ```
#[macro_export]
macro_rules! tracing_debug {
    ($msg:expr, $body:expr) => {
        let (_msg_display_trunc, _body_debug_trunc) =
            $crate::tracing_debug_impl::prepare_tracing_debug(&$msg, &$body);
        tracing::debug!("{} = {}", _msg_display_trunc, _body_debug_trunc);
    };
}

pub mod tracing_debug_impl {
    pub fn prepare_tracing_debug(
        msg: &impl std::fmt::Display,
        body: &impl std::fmt::Debug,
    ) -> (String, String) {
        let msg_display = format!("{}", msg);
        let body_debug = format!("{:?}", body);
        let msg_display_trunc = truncate_or_pad(&msg_display, crate::debug::TRACING_MSG_WIDTH);
        let body_debug_trunc = truncate_or_pad(&body_debug, crate::debug::TRACING_BODY_WIDTH);
        (msg_display_trunc, body_debug_trunc)
    }

    pub fn truncate_or_pad(string: &str, width: usize) -> String {
        if string.len() > width {
            string.chars().take(width).collect()
        } else {
            let mut padded_string = string.to_string();
            padded_string.push_str(&" ".repeat(width - string.len()));
            padded_string
        }
    }

    #[test]
    fn test_truncate_or_pad() {
        let long_string = "Hello, world!";
        let short_string = "Hi!";
        let width = 10;

        assert_eq!(truncate_or_pad(long_string, width), "Hello, wor");
        assert_eq!(truncate_or_pad(short_string, width), "Hi!       ");
    }
}

pub mod debug {
    pub const TRACING_MSG_WIDTH: usize = 25;
    pub const TRACING_BODY_WIDTH: usize = 70;

    /// Works with [tracing_debug!] to initialize the tracing subscriber to output the
    /// least amount of noise (no line number, target, file, etc).
    pub fn tracing_init() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .pretty()
            .compact()
            .with_file(false)
            .with_target(false)
            .with_line_number(false)
            .without_time()
            .init();
    }
}

/// Use this macro to make it more ergonomic to work with [PathBuf]s.
///
/// # Example - create a new path
///
/// ```
/// use tls::fs_paths;
/// use std::path::{PathBuf, Path};
///
/// let my_path = fs_paths![with_empty_root => "usr/bin" => "bash"];
/// assert_eq!(my_path, PathBuf::from("usr/bin/bash"));
///
/// let my_path = fs_paths![with_empty_root => "usr" => "bin" => "bash"];
/// assert_eq!(my_path, PathBuf::from("usr/bin/bash"));
/// ```
///
/// # Example - join to an existing path
///
/// ```
/// use tls::fs_paths;
/// use std::path::{PathBuf, Path};
///
/// let root = PathBuf::from("/home/user");
/// let my_path = fs_paths![with_root => root => "Downloads" => "rust"];
/// assert_eq!(my_path, PathBuf::from("/home/user/Downloads/rust"));
///
/// let root = PathBuf::from("/home/user");
/// let my_path = fs_paths![with_root => root => "Downloads" => "rust"];
/// assert_eq!(my_path, PathBuf::from("/home/user/Downloads/rust"));
/// ```
#[macro_export]
macro_rules! fs_paths {
    // Join to an existing root path.
    (with_root=> $path:expr=> $($x:expr)=>*) => {{
        let mut it = $path.clone();
        $(
            it = it.join($x);
        )*
        it
    }};

    // Create a new path w/ no pre-existing root.
    (with_empty_root=> $($x:expr)=>*) => {{
        use std::path::{PathBuf};
        let mut it = PathBuf::new();
        $(
            it = it.join($x);
        )*
        it
    }}
}

/// Use this macro to ensure that all the paths provided exist on the filesystem, in which
/// case it will return true If any of the paths do not exist, the function will return
/// false. No error will be returned in case any of the paths are invalid or there aren't
/// enough permissions to check if the paths exist.
///
/// # Example
///
/// ```
/// use tls::fs_paths_exist;
/// use tls::fs_paths;
/// use r3bl_test_fixtures::create_temp_dir;
///
/// let temp_dir = create_temp_dir().unwrap();
/// let path_1 = fs_paths![with_root => temp_dir => "some_dir"];
/// let path_2 = fs_paths![with_root => temp_dir => "another_dir"];
///
/// assert!(!fs_paths_exist!(path_1, path_2));
/// ```
#[macro_export]
macro_rules! fs_paths_exist {
    ($($x:expr),*) => {'block: {
        $(
            if !std::fs::metadata($x).is_ok() {
                break 'block false;
            };
        )*
        true
    }};
}

#[derive(Debug, Error, Diagnostic)]
pub enum FsOpError {
    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),

    #[error("Directory does not exist: {0}")]
    DirectoryDoesNotExist(String),

    #[error("File already exists: {0}")]
    FileAlreadyExists(String),

    #[error("Directory already exists: {0}")]
    DirectoryAlreadyExists(String),

    #[error("Insufficient permissions: {0}")]
    PermissionDenied(String),

    #[error("Invalid name: {0}")]
    InvalidName(String),

    #[error("Failed to perform fs operation directory: {0}")]
    IoError(#[from] std::io::Error),
}

pub type FsOpResult<T> = miette::Result<T, FsOpError>;

/// This macro is used to wrap a block with code that saves the current working directory,
/// runs the block of code for the test, and then restores the original working directory.
/// It also ensures that the test is run serially.
///
/// Be careful when manipulating the current working directory in tests using
/// [env::set_current_dir] as it can affect other tests that run in parallel.
#[macro_export]
macro_rules! serial_preserve_pwd_test {
    ($name:ident, $block:block) => {
        #[serial_test::serial]
        #[test]
        fn $name() {
            with_saved_pwd!($block);
        }
    };
}

/// This macro is used to wrap a block with code that saves the current working directory,
/// runs the block of code for the test, and then restores the original working directory.
///
/// Use this in conjunction with [serial_test::serial] in order to make sure that multiple
/// threads are not changing the current working directory at the same time (even with
/// this macro). In other words, use this macro [serial_preserve_pwd_test!] for tests.
#[macro_export]
macro_rules! with_saved_pwd {
    ($block:block) => {{
        let og_pwd = env::current_dir().unwrap();
        let result = { $block };
        env::set_current_dir(og_pwd).unwrap();
        result
    }};
}

pub mod http_client {
    use super::*;

    pub mod constants {
        pub const USER_AGENT: &str = "scripting.rs/1.0";
    }

    pub fn create_client_with_user_agent(
        user_agent: Option<&str>,
    ) -> miette::Result<reqwest::Client> {
        let it = reqwest::Client::builder()
            .user_agent(user_agent.map_or_else(
                || constants::USER_AGENT.to_owned(),
                |user_agent| user_agent.to_owned(),
            ))
            .build();
        it.into_diagnostic()
    }
}

pub mod github_api {
    use std::ops::Add;

    use super::*;

    pub mod constants {
        pub const TAG_NAME: &str = "tag_name";
        pub const VERSION_PREFIX: &str = "v";
    }

    pub mod urls {
        pub const REPO_LATEST_RELEASE: &str =
            "https://api.github.com/repos/{org}/{repo}/releases/latest";
    }

    pub async fn try_get_latest_release_tag_from_github(
        org: &str,
        repo: &str,
    ) -> miette::Result<String> {
        let url = urls::REPO_LATEST_RELEASE
            .replace("{org}", org)
            .replace("{repo}", repo);

        println!("url: {}", url.as_str().magenta());

        let client = http_client::create_client_with_user_agent(None)?;
        let response = client.get(url).send().await.into_diagnostic()?;
        let response = response.error_for_status().into_diagnostic()?; // Return an error if the status != 2xx.
        let response: serde_json::Value = response.json().await.into_diagnostic()?;

        let tag_name = match response[constants::TAG_NAME].as_str() {
            Some(tag_name) => tag_name.trim_start_matches(constants::VERSION_PREFIX),
            None => miette::bail!("Failed to get tag name from JSON: {:?}", response),
        };

        ok!(tag_name.to_owned())
    }

    /// A simple URL builder that allows chaining strings together to build a URL. The URL
    /// is built by concatenating the strings together. To generate the final URL, call
    /// `to_string()` from the [Display] trait, which is implemented for [UrlBuilder].
    ///
    /// # Example
    ///
    /// ```
    /// use tls::github_api::{UrlBuilder, Separator};
    /// let url_builder = UrlBuilder::default()
    ///    + "https://" + "api.github.com" + Separator::ForwardSlash + "repos"
    ///    + Separator::ForwardSlash + "cloudflare" + Separator::Underscore + "cfssl";
    /// assert_eq!(url_builder.to_string(), "https://api.github.com/repos/cloudflare_cfssl");
    /// ```
    #[derive(Debug, Default, Clone)]
    pub struct UrlBuilder {
        pub inner: Vec<String>,
    }

    #[derive(Debug, Display, EnumString)]
    pub enum Separator {
        #[strum(serialize = "_")]
        Underscore,
        #[strum(serialize = "/")]
        ForwardSlash,
    }

    impl<T: Display> Add<T> for &UrlBuilder {
        type Output = UrlBuilder;

        fn add(self, rhs: T) -> Self::Output {
            let mut it = self.clone();
            it.inner.push(rhs.to_string());
            it
        }
    }

    impl<T: Display> Add<T> for UrlBuilder {
        type Output = Self;

        fn add(mut self, rhs: T) -> Self {
            self.inner.push(rhs.to_string());
            self
        }
    }

    impl<T: Display> AddAssign<T> for UrlBuilder {
        fn add_assign(&mut self, rhs: T) {
            self.inner.push(rhs.to_string());
        }
    }

    impl Deref for UrlBuilder {
        type Target = Vec<String>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl Display for UrlBuilder {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.inner.join(""))
        }
    }

    #[cfg(test)]
    mod tests_github_api {
        use super::*;
        use github_api::{try_get_latest_release_tag_from_github, UrlBuilder};
        use r3bl_ansi_color::{is_fully_uninteractive_terminal, TTYResult};

        /// Do not run this in CI/CD since it makes API calls to github.com.
        #[tokio::test]
        async fn test_get_latest_tag_from_github() {
            // This is for CI/CD.
            if let TTYResult::IsNotInteractive = is_fully_uninteractive_terminal() {
                return;
            }

            let org = "cloudflare";
            let repo = "cfssl";
            let tag = try_get_latest_release_tag_from_github(org, repo)
                .await
                .unwrap();
            assert!(!tag.is_empty());
            println!("Latest tag: {}", tag.magenta());
        }

        #[test]
        fn test_url_builder_add_ref() {
            let url_builder_root = UrlBuilder::default();
            let url_builder_1 = &url_builder_root + "https://r3bl.com";
            let url_builder_2 = &url_builder_root + "/blog";
            assert_eq!(url_builder_1.to_string(), "https://r3bl.com");
            assert_eq!(url_builder_2.to_string(), "/blog");
        }

        #[test]
        fn test_url_builder_add() {
            let url_builder = UrlBuilder::default();
            let url_builder = url_builder
                + "https://"
                + "api.github.com"
                + github_api::Separator::ForwardSlash
                + "repos"
                + github_api::Separator::ForwardSlash
                + "cloudflare"
                + github_api::Separator::ForwardSlash
                + "cfssl"
                + github_api::Separator::ForwardSlash
                + "releases"
                + github_api::Separator::ForwardSlash
                + "latest"
                + github_api::Separator::Underscore
                + "tag_name"
                + github_api::Separator::ForwardSlash
                + "v";

            assert_eq!(
                url_builder.to_string(),
                "https://api.github.com/repos/cloudflare/cfssl/releases/latest_tag_name/v"
            );
        }

        #[test]
        fn test_url_builder_add_assign() {
            let mut url_builder = UrlBuilder::default();
            url_builder += "https://";
            url_builder += "api.github.com";
            url_builder += github_api::Separator::ForwardSlash;
            url_builder += "repos";
            url_builder += github_api::Separator::ForwardSlash;
            url_builder += "cloudflare";
            url_builder += github_api::Separator::ForwardSlash;
            url_builder += "cfssl";
            url_builder += github_api::Separator::ForwardSlash;
            url_builder += "releases";
            url_builder += github_api::Separator::ForwardSlash;
            url_builder += "latest";
            url_builder += github_api::Separator::Underscore;
            url_builder += "tag_name";
            url_builder += github_api::Separator::ForwardSlash;
            url_builder += "v";

            assert_eq!(
                url_builder.to_string(),
                "https://api.github.com/repos/cloudflare/cfssl/releases/latest_tag_name/v"
            );
        }
    }
}

pub mod directory_change {
    use super::*;

    /// Change cwd for current process.
    pub fn try_change_directory(new_dir: impl AsRef<Path>) -> FsOpResult<()> {
        match env::set_current_dir(new_dir.as_ref()) {
            Ok(_) => ok!(),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    FsOpResult::Err(FsOpError::DirectoryDoesNotExist(err.to_string()))
                }
                ErrorKind::PermissionDenied => {
                    FsOpResult::Err(FsOpError::PermissionDenied(err.to_string()))
                }
                ErrorKind::InvalidInput => FsOpResult::Err(FsOpError::InvalidName(err.to_string())),
                _ => FsOpResult::Err(FsOpError::IoError(err)),
            },
        }
    }

    #[cfg(test)]
    mod tests_directory_change {
        use super::*;
        use directory_change::try_change_directory;
        use r3bl_test_fixtures::create_temp_dir;

        serial_preserve_pwd_test!(test_try_change_directory_permissions_errors, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                // Create a new temporary directory.
                let new_tmp_dir =
                    fs_paths!(with_root=> root => "test_change_dir_permissions_errors");
                fs::create_dir_all(&new_tmp_dir).unwrap();
                assert!(new_tmp_dir.exists());

                // Create a directory with no permissions for user.
                let no_permissions_dir = fs_paths!(with_root=> new_tmp_dir => "no_permissions_dir");
                fs::create_dir_all(&no_permissions_dir).unwrap();
                let mut permissions = fs::metadata(&no_permissions_dir).unwrap().permissions();
                permissions.set_mode(0o000);
                fs::set_permissions(&no_permissions_dir, permissions).unwrap();
                assert!(no_permissions_dir.exists());
                // Try to change to a directory with insufficient permissions.
                let result = try_change_directory(&no_permissions_dir);
                println!("✅ err: {:?}", result);
                assert!(result.is_err());
                assert!(matches!(result, Err(FsOpError::PermissionDenied(_))));

                // Change the permissions back, so that it can be cleaned up!
                let mut permissions = fs::metadata(&no_permissions_dir).unwrap().permissions();
                permissions.set_mode(0o777);
                fs::set_permissions(&no_permissions_dir, permissions).unwrap();
            });
        });

        serial_preserve_pwd_test!(test_try_change_directory_happy_path, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                // Create a new temporary directory.
                let new_tmp_dir = fs_paths!(with_root=> root => "test_change_dir_happy_path");
                fs::create_dir_all(&new_tmp_dir).unwrap();
                assert!(new_tmp_dir.exists());

                // Change to the temporary directory.
                try_change_directory(&new_tmp_dir).unwrap();
                assert_eq!(env::current_dir().unwrap(), new_tmp_dir);

                // Change back to the original directory.
                try_change_directory(&root).unwrap();
                assert_eq!(env::current_dir().unwrap(), *root);
            });
        });

        serial_preserve_pwd_test!(test_try_change_directory_non_existent, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                // Create a new temporary directory.
                let new_tmp_dir = fs_paths!(with_root=> root => "test_change_dir_non_existent");
                fs::create_dir_all(&new_tmp_dir).unwrap();
                assert!(new_tmp_dir.exists());

                // Try to change to a non-existent directory.
                let non_existent_dir = fs_paths!(with_root=> new_tmp_dir => "non_existent_dir");
                let result = try_change_directory(&non_existent_dir);
                assert!(result.is_err());
                assert!(matches!(result, Err(FsOpError::DirectoryDoesNotExist(_))));

                // Change back to the original directory.
                try_change_directory(&root).unwrap();
                assert_eq!(env::current_dir().unwrap(), *root);
            });
        });

        serial_preserve_pwd_test!(test_try_change_directory_invalid_name, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                // Create a new temporary directory.
                let new_tmp_dir = fs_paths!(with_root=> root => "test_change_dir_invalid_name");
                fs::create_dir_all(&new_tmp_dir).unwrap();
                assert!(new_tmp_dir.exists());

                // Try to change to a directory with an invalid name.
                let invalid_name_dir = fs_paths!(with_root=> new_tmp_dir => "invalid_name_dir\0");
                let result = try_change_directory(&invalid_name_dir);
                assert!(result.is_err());
                println!("✅ err: {:?}", result);
                assert!(matches!(result, Err(FsOpError::InvalidName(_))));

                // Change back to the original directory.
                try_change_directory(&root).unwrap();
                assert_eq!(env::current_dir().unwrap(), *root);
            });
        });
    }
}

pub mod directory_create {
    use super::*;

    #[derive(Debug, Display, Default)]
    pub enum MkdirOptions {
        #[default]
        CreateIntermediateDirectories,
        CreateIntermediateDirectoriesOnlyIfNotExists,
        CreateIntermediateDirectoriesAndPurgeExisting,
    }

    fn handle_err(err: std::io::Error) -> FsOpResult<()> {
        match err.kind() {
            ErrorKind::PermissionDenied => {
                FsOpResult::Err(FsOpError::PermissionDenied(err.to_string()))
            }
            ErrorKind::InvalidInput => FsOpResult::Err(FsOpError::InvalidName(err.to_string())),
            ErrorKind::ReadOnlyFilesystem => {
                FsOpResult::Err(FsOpError::PermissionDenied(err.to_string()))
            }
            _ => FsOpResult::Err(FsOpError::IoError(err)),
        }
    }

    fn create_dir_all(new_path: &Path) -> FsOpResult<()> {
        match fs::create_dir_all(new_path) {
            Ok(_) => ok!(),
            Err(err) => handle_err(err),
        }
    }

    /// Creates a new directory at the specified path.
    /// - Depending on the [MkdirOptions] the directories can be created destructively or
    ///   non-destructively.
    /// - Any intermediate folders that don't exist will be created.
    ///
    /// If any permissions issues occur or the directory can't be created due to
    /// inconsistent [MkdirOptions] then an error is returned.
    pub fn try_mkdir(new_path: impl AsRef<Path>, options: MkdirOptions) -> FsOpResult<()> {
        let new_path = new_path.as_ref();

        // Pre-process the directory creation options.
        match options {
            // This is the default option.
            MkdirOptions::CreateIntermediateDirectories => { /* Do nothing. */ }

            // This will delete the directory if it exists and then create it.
            MkdirOptions::CreateIntermediateDirectoriesAndPurgeExisting => {
                match fs::exists(new_path) {
                    // The new_path exists.
                    Ok(true) => {
                        // Remove the entire new_path.
                        if let Err(err) = fs::remove_dir_all(new_path) {
                            return handle_err(err);
                        }
                    }
                    // Encountered problem checking if the new_path exists.
                    Err(err) => return handle_err(err),
                    // The new_path does not exist.
                    _ => { /* Do nothing. */ }
                }
            }

            // This will error out if the directory already exists.
            MkdirOptions::CreateIntermediateDirectoriesOnlyIfNotExists => {
                if let Ok(true) = fs::exists(new_path) {
                    let new_dir_display = fs_path::path_as_string(new_path);
                    return FsOpResult::Err(FsOpError::DirectoryAlreadyExists(new_dir_display));
                }
            }
        }

        // Create the path.
        create_dir_all(new_path)
    }

    #[cfg(test)]
    mod tests_directory_create {
        use super::*;
        use directory_create::{try_mkdir, MkdirOptions::*};
        use r3bl_test_fixtures::create_temp_dir;

        serial_preserve_pwd_test!(test_try_mkdir, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                // Create a temporary directory.
                let tmp_root_dir = fs_paths!(with_root=> root => "test_create_clean_new_dir");
                try_mkdir(&tmp_root_dir, CreateIntermediateDirectories).unwrap();

                // Create a new directory inside the temporary directory.
                let new_dir = fs_paths!(with_root=> tmp_root_dir => "new_dir");
                try_mkdir(&new_dir, CreateIntermediateDirectories).unwrap();
                assert!(new_dir.exists());

                // Try & fail to create the same directory again non destructively.
                let result = try_mkdir(&new_dir, CreateIntermediateDirectoriesOnlyIfNotExists);
                assert!(result.is_err());
                assert!(matches!(result, Err(FsOpError::DirectoryAlreadyExists(_))));

                // Create a file inside the new directory.
                let file_path = new_dir.join("test_file.txt");
                fs::write(&file_path, "test").unwrap();
                assert!(file_path.exists());

                // Call `mkdir` again with destructive options and ensure the directory is
                // clean.
                try_mkdir(&new_dir, CreateIntermediateDirectoriesAndPurgeExisting).unwrap();

                // Ensure the directory is clean.
                assert!(new_dir.exists());
                assert!(!file_path.exists());
            });
        });
    }
}

/// Note that [PathBuf] is owned and [Path] is a slice into it.
/// - So replace `&`[PathBuf] with a `&`[Path].
/// - More details
///   [here](https://rust-lang.github.io/rust-clippy/master/index.html#ptr_arg).
pub mod fs_path {
    use super::*;

    /// Checks whether the directory exist. If won't provide any errors if there are
    /// permissions issues or the directory is invalid. Use [try_directory_exists] if you
    /// want to handle these errors.
    pub fn directory_exists(directory: impl AsRef<Path>) -> bool {
        fs::metadata(directory).is_ok_and(|metadata| metadata.is_dir())
    }

    /// Checks whether the file exists. If won't provide any errors if there are permissions
    /// issues or the file is invalid. Use [try_file_exists] if you want to handle these
    /// errors.
    pub fn file_exists(file: impl AsRef<Path>) -> bool {
        fs::metadata(file).is_ok_and(|metadata| metadata.is_file())
    }

    /// Checks whether the directory exist. If there are issues with permissions for
    /// directory access or invalid directory it will return an error. Use
    /// [directory_exists] if you want to ignore these errors.
    pub fn try_directory_exists(directory_path: impl AsRef<Path>) -> FsOpResult<bool> {
        match fs::metadata(directory_path) {
            Ok(metadata) => {
                // The directory_path might be found in the file system, but it might be a
                // file. This won't result in an error.
                ok!(metadata.is_dir())
            }
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    FsOpResult::Err(FsOpError::DirectoryDoesNotExist(err.to_string()))
                }
                ErrorKind::InvalidInput => FsOpResult::Err(FsOpError::InvalidName(err.to_string())),
                _ => FsOpResult::Err(FsOpError::IoError(err)),
            },
        }
    }

    /// Checks whether the file exist. If there are issues with permissions for file access
    /// or invalid file it will return an error. Use [file_exists] if you want to ignore
    /// these errors.
    pub fn try_file_exists(file_path: impl AsRef<Path>) -> FsOpResult<bool> {
        match fs::metadata(file_path) {
            // The file_path might be found in the file system, but it might be a
            // directory. This won't result in an error.
            Ok(metadata) => ok!(metadata.is_file()),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    FsOpResult::Err(FsOpError::FileDoesNotExist(err.to_string()))
                }
                ErrorKind::InvalidInput => FsOpResult::Err(FsOpError::InvalidName(err.to_string())),
                _ => FsOpResult::Err(FsOpError::IoError(err)),
            },
        }
    }

    /// Returns the current working directory of the process as a [PathBuf] (owned). If
    /// there are issues with permissions for directory access or invalid directory it
    /// will return an error.
    ///
    /// - `bash` equivalent: `$(pwd)`
    /// - Eg: `PathBuf("/home/user/some/path")`
    pub fn try_pwd() -> FsOpResult<PathBuf> {
        match env::current_dir() {
            Ok(pwd) => FsOpResult::Ok(pwd),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    FsOpResult::Err(FsOpError::DirectoryDoesNotExist(err.to_string()))
                }
                _ => FsOpResult::Err(FsOpError::IoError(err)),
            },
        }
    }

    /// Returns the [Path] slice as a string.
    /// - Eg: `"/home/user/some/path"`
    pub fn path_as_string(path: &Path) -> String {
        path.display().to_string()
    }

    #[cfg(test)]
    mod tests_fs_path {
        use super::*;
        use fs_path::try_pwd;
        use r3bl_test_fixtures::create_temp_dir;

        serial_preserve_pwd_test!(test_try_pwd, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                let new_dir = root.join("test_pwd");
                fs::create_dir_all(&new_dir).unwrap();
                env::set_current_dir(&new_dir).unwrap();

                let pwd = try_pwd().unwrap();
                assert!(pwd.exists());
                assert_eq!(pwd, new_dir);
            });
        });

        serial_preserve_pwd_test!(test_try_pwd_errors, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                // Create a directory, change to it, remove all permissions for user.
                let no_permissions_dir = root.join("no_permissions_dir");
                fs::create_dir_all(&no_permissions_dir).unwrap();
                env::set_current_dir(&no_permissions_dir).unwrap();
                let mut permissions = fs::metadata(&no_permissions_dir).unwrap().permissions();
                permissions.set_mode(0o000);
                fs::set_permissions(&no_permissions_dir, permissions).unwrap();
                assert!(no_permissions_dir.exists());

                // Try to get the pwd with insufficient permissions. It should work!
                let result = try_pwd();
                assert!(result.is_ok());

                // Change the permissions back, so that it can be cleaned up!
                let mut permissions = fs::metadata(&no_permissions_dir).unwrap().permissions();
                permissions.set_mode(0o777);
                fs::set_permissions(&no_permissions_dir, permissions).unwrap();

                // Delete this directory, and try pwd again. It will not longer exist.
                fs::remove_dir_all(&no_permissions_dir).unwrap();
                let result = try_pwd();
                assert!(result.is_err());
                assert!(matches!(result, Err(FsOpError::DirectoryDoesNotExist(_))));
            });
        });

        serial_preserve_pwd_test!(test_fq_path_relative_to_try_pwd, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                let sub_path = "test_fq_path_relative_to_pwd";
                let new_dir = root.join(sub_path);
                fs::create_dir_all(&new_dir).unwrap();

                env::set_current_dir(&root).unwrap();

                println!("Current directory set to: {}", root);
                println!("Current directory is    : {}", try_pwd().unwrap().display());

                let fq_path = fs_paths!(with_root=> try_pwd().unwrap() => sub_path);

                println!("Sub directory created at: {}", fq_path.display());
                println!("Sub directory exists    : {}", fq_path.exists());

                assert!(fq_path.exists());
            });
        });

        serial_preserve_pwd_test!(test_path_as_string, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                env::set_current_dir(&root).unwrap();

                let fq_path = fs_paths!(with_root=> try_pwd().unwrap() => "some_dir");
                let fq_path_str = fs_path::path_as_string(&fq_path);

                assert_eq!(fq_path_str, fq_path.display().to_string());
            });
        });

        serial_preserve_pwd_test!(test_try_file_exists, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                let new_dir = root.join("test_file_exists_dir");
                fs::create_dir_all(&new_dir).unwrap();

                let new_file = new_dir.join("test_file_exists_file.txt");
                fs::write(&new_file, "test").unwrap();

                assert!(fs_path::try_file_exists(&new_file).unwrap());
                assert!(!fs_path::try_file_exists(&new_dir).unwrap());

                fs::remove_dir_all(&new_dir).unwrap();

                // Ensure that an invalid path returns an error.
                assert!(fs_path::try_file_exists(&new_file).is_err()); // This file does not exist.
                assert!(fs_path::try_file_exists(&new_dir).is_err()); // This directory does not exist.
            });
        });

        serial_preserve_pwd_test!(test_try_file_exists_not_found_error, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                let new_dir = root.join("test_file_exists_not_found_error");

                // Try to check if the file exists. It should return an error.
                let result = fs_path::try_file_exists(&new_dir);
                assert!(result.is_err());
                assert!(matches!(result, Err(FsOpError::FileDoesNotExist(_))));
            });
        });

        serial_preserve_pwd_test!(test_try_file_exists_invalid_name_error, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                let new_dir = root.join("test_file_exists_invalid_name_error\0");

                // Try to check if the file exists. It should return an error.
                let result = fs_path::try_file_exists(&new_dir);
                assert!(result.is_err());
                assert!(matches!(result, Err(FsOpError::InvalidName(_))));
            });
        });

        serial_preserve_pwd_test!(test_try_file_exists_permissions_errors, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                // Create a directory, change to it, remove all permissions for user.
                let no_permissions_dir = root.join("no_permissions_dir");
                fs::create_dir_all(&no_permissions_dir).unwrap();
                let mut permissions = fs::metadata(&no_permissions_dir).unwrap().permissions();
                permissions.set_mode(0o000);
                fs::set_permissions(&no_permissions_dir, permissions).unwrap();
                assert!(no_permissions_dir.exists());

                // Try to check if the file exists with insufficient permissions. It should
                // work!
                let result = fs_path::try_file_exists(&no_permissions_dir);
                assert!(result.is_ok());

                // Change the permissions back, so that it can be cleaned up!
                let mut permissions = fs::metadata(&no_permissions_dir).unwrap().permissions();
                permissions.set_mode(0o777);
                fs::set_permissions(&no_permissions_dir, permissions).unwrap();
            });
        });

        serial_preserve_pwd_test!(test_try_directory_exists, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                let new_dir = root.join("test_dir_exists_dir");
                fs::create_dir_all(&new_dir).unwrap();

                let new_file = new_dir.join("test_dir_exists_file.txt");
                fs::write(&new_file, "test").unwrap();

                assert!(fs_path::try_directory_exists(&new_dir).unwrap());
                assert!(!fs_path::try_directory_exists(&new_file).unwrap());
            })
        });

        serial_preserve_pwd_test!(test_try_directory_exists_not_found_error, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                let new_dir = root.join("test_dir_exists_not_found_error");

                // Try to check if the directory exists. It should return an error.
                let result = fs_path::try_directory_exists(&new_dir);
                assert!(result.is_err());
                assert!(matches!(result, Err(FsOpError::DirectoryDoesNotExist(_))));
            });
        });

        serial_preserve_pwd_test!(test_try_directory_exists_invalid_name_error, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                let new_dir = root.join("test_dir_exists_invalid_name_error\0");

                // Try to check if the directory exists. It should return an error.
                let result = fs_path::try_directory_exists(&new_dir);
                assert!(result.is_err());
                assert!(matches!(result, Err(FsOpError::InvalidName(_))));
            });
        });

        serial_preserve_pwd_test!(test_try_directory_exists_permissions_errors, {
            with_saved_pwd!({
                // Create the root temp dir.
                let root = create_temp_dir().unwrap();

                // Create a directory, change to it, remove all permissions for user.
                let no_permissions_dir = root.join("no_permissions_dir");
                fs::create_dir_all(&no_permissions_dir).unwrap();
                let mut permissions = fs::metadata(&no_permissions_dir).unwrap().permissions();
                permissions.set_mode(0o000);
                fs::set_permissions(&no_permissions_dir, permissions).unwrap();
                assert!(no_permissions_dir.exists());

                // Try to check if the directory exists with insufficient permissions. It
                // should work!
                let result = fs_path::try_directory_exists(&no_permissions_dir);
                assert!(result.is_ok());

                // Change the permissions back, so that it can be cleaned up!
                let mut permissions = fs::metadata(&no_permissions_dir).unwrap().permissions();
                permissions.set_mode(0o777);
                fs::set_permissions(&no_permissions_dir, permissions).unwrap();
            });
        });
    }
}

pub mod environment {
    use super::*;

    #[cfg(target_os = "windows")]
    const OS_SPECIFIC_ENV_PATH_SEPARATOR: &str = ";";
    #[cfg(not(target_os = "windows"))]
    const OS_SPECIFIC_ENV_PATH_SEPARATOR: &str = ":";

    #[derive(Debug, Display, EnumString)]
    pub enum EnvKeys {
        #[strum(serialize = "PATH")]
        Path,
    }

    /// Returns the PATH environment variable as a vector of tuples.
    ///
    /// # Example
    ///
    /// ```
    /// use tls::environment::get_path_envs;
    /// let path_envs = get_path_envs("/usr/bin");
    /// let expected = vec![
    ///     ("PATH".to_string(), "/usr/bin".to_string())
    /// ];
    /// assert_eq!(path_envs, expected);
    /// ```
    pub fn get_path_envs(path: &str) -> Vec<(String, String)> {
        vec![(environment::EnvKeys::Path.to_string(), path.to_string())]
    }

    pub fn try_get(key: EnvKeys) -> miette::Result<String> {
        env::var(key.to_string()).into_diagnostic()
    }

    pub fn try_get_path_from_env() -> miette::Result<String> {
        try_get(EnvKeys::Path)
    }

    pub fn try_get_path_prefixed(prefix_path: impl AsRef<Path>) -> miette::Result<String> {
        let path = try_get_path_from_env()?;
        let add_to_path: String = format!(
            "{}{}{}",
            prefix_path.as_ref().display(),
            OS_SPECIFIC_ENV_PATH_SEPARATOR,
            path
        );
        tracing_debug!("my_path", add_to_path);
        ok!(add_to_path)
    }

    #[cfg(test)]
    mod tests_environment {
        use super::*;

        #[test]
        fn test_get_path() {
            let path = environment::try_get_path_from_env().unwrap();
            assert!(!path.is_empty());
        }

        #[test]
        fn test_get_path_prefixed() {
            let prefix_path = "/usr/bin";
            let path = environment::try_get_path_prefixed(prefix_path).unwrap();
            assert!(!path.is_empty());
            assert!(path.starts_with(prefix_path));
        }
    }
}

pub mod download {
    use super::*;
    use http_client::create_client_with_user_agent;

    pub async fn try_download_file_overwrite_existing(
        source_url: &str,
        destination_file: impl AsRef<Path>,
    ) -> miette::Result<()> {
        let destination = destination_file.as_ref();

        let client = create_client_with_user_agent(None)?;
        let response = client.get(source_url).send().await.into_diagnostic()?;
        let response = response.error_for_status().into_diagnostic()?;
        let response = response.bytes().await.into_diagnostic()?;

        let mut dest_file = fs::File::create(destination).into_diagnostic()?;
        dest_file.write_all(&response).into_diagnostic()?;

        ok!()
    }

    #[cfg(test)]
    mod tests_download {
        use super::*;
        use download::try_download_file_overwrite_existing;
        use r3bl_test_fixtures::create_temp_dir;

        #[tokio::test]
        async fn test_download_file_overwrite_existing() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            let new_dir = root.join("test_download_file_overwrite_existing");
            fs::create_dir_all(&new_dir).unwrap();

            let source_url = "https://github.com/cloudflare/cfssl/releases/download/v1.6.5/cfssljson_1.6.5_linux_amd64";
            let destination_file = new_dir.join("cfssljson");

            // Download file (no pre-existing file).
            try_download_file_overwrite_existing(source_url, &destination_file)
                .await
                .unwrap();
            assert!(destination_file.exists());

            let meta_data = destination_file.metadata().unwrap();
            let og_file_size = meta_data.len();

            // Download file again (overwrite existing).
            try_download_file_overwrite_existing(source_url, &destination_file)
                .await
                .unwrap();
            assert!(destination_file.exists());

            // Ensure that the file sizes are the same.
            let meta_data = destination_file.metadata().unwrap();
            let new_file_size = meta_data.len();
            assert_eq!(og_file_size, new_file_size);
        }
    }
}

pub mod permissions {
    use super::*;

    /// Sets the file at the specified path to be executable by owner, group, and others.
    /// - `bash` equivalent: `chmod +x file`
    /// - Eg: `set_file_executable("some_file.sh")`
    /// - The `file` must exist and be a file (not a directory).
    pub fn try_set_file_executable(file: impl AsRef<Path>) -> miette::Result<()> {
        let file = file.as_ref();
        let metadata = fs::metadata(file).into_diagnostic()?;

        if !metadata.is_file() {
            miette::bail!("This is not a file: '{}'", file.display());
        }

        // Set execute permissions for owner, group, and others on this file. 755 means:
        // - 7 (owner): read (4) + write (2) + execute (1) = 7 (rwx)
        // - 5 (group): read (4) + execute (1) = 5 (r-x)
        // - 5 (others): read (4) + execute (1) = 5 (r-x)
        fs::set_permissions(file, std::fs::Permissions::from_mode(0o755)).into_diagnostic()
    }

    #[cfg(test)]
    mod tests_permissions {
        use super::*;
        use permissions::try_set_file_executable;
        use r3bl_test_fixtures::create_temp_dir;

        #[test]
        fn test_set_file_executable() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            let new_dir = root.join("test_set_file_executable");
            fs::create_dir_all(&new_dir).unwrap();

            let new_file = new_dir.join("test_set_file_executable.sh");
            fs::write(&new_file, "echo 'Hello, World!'").unwrap();

            try_set_file_executable(&new_file).unwrap();

            let metadata = fs::metadata(&new_file).unwrap();
            let lhs = metadata.permissions();

            // Assert that the file has executable permission for owner, group, and others:
            // - The bitwise AND operation (lhs.mode() & 0o777) ensures that only the
            //   permission bits are compared, ignoring other bits that might be present in
            //   the mode.
            // - The assertion checks if the permission bits match 0o755.
            assert_eq!(lhs.mode() & 0o777, 0o755);
        }

        #[test]
        fn test_set_file_executable_on_non_file() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            let new_dir = root.join("test_set_file_executable_on_non_file");
            fs::create_dir_all(&new_dir).unwrap();

            let result = try_set_file_executable(&new_dir);
            assert!(result.is_err());
        }

        #[test]
        fn test_set_file_executable_on_non_existent_file() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            let new_dir = root.join("test_set_file_executable_on_non_existent_file");
            fs::create_dir_all(&new_dir).unwrap();

            let non_existent_file = new_dir.join("non_existent_file.sh");
            let result = try_set_file_executable(&non_existent_file);
            assert!(result.is_err());
        }
    }
}
