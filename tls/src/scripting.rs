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
use miette::IntoDiagnostic;
use r3bl_core::{create_global_singleton, ok};
use std::{
    env,
    fmt::Display,
    fs,
    io::Write as _,
    ops::{AddAssign, Deref},
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
};
use strum_macros::{Display, EnumString};

/// Use this macro to make it more ergonomic to work with [PathBuf]s.
///
/// # Example - create a new path
///
/// ```
/// use tls::path;
/// use std::path::{PathBuf, Path};
///
/// let my_path = path![new: "usr/bin"];
/// assert_eq!(my_path, PathBuf::from("usr/bin"));
///
/// let my_path = path![new: "usr", "bin"];
/// assert_eq!(my_path, PathBuf::from("usr/bin"));
/// ```
///
/// # Example - join to an existing path
///
/// ```
/// use tls::path;
/// use std::path::{PathBuf, Path};
///
/// let root_path = PathBuf::from("/home/user");
/// let my_path = path![with_root: root_path, "Downloads", "rust"];
/// assert_eq!(my_path, PathBuf::from("/home/user/Downloads/rust"));
///
/// let root_path = PathBuf::from("/home/user");
/// let my_path = path![with_root: root_path, "Downloads", "rust"];
/// assert_eq!(my_path, PathBuf::from("/home/user/Downloads/rust"));
/// ```
#[macro_export]
macro_rules! path {
    // Join to an existing root path.
    (with_root: $path:expr, $($x:expr),*) => {{
        let mut it = $path.clone();
        $(
            it = it.join($x);
        )*
        it
    }};

    // Create a new path w/ no pre-existing root.
    (new: $($x:expr),*) => {{
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
/// use tls::paths_exist;
/// use tls::path;
/// use r3bl_test_fixtures::create_temp_dir;
///
/// let temp_dir = create_temp_dir().unwrap();
/// let path_1 = path![with_root: temp_dir, "some_folder"];
/// let path_2 = path![with_root: temp_dir, "another_folder"];
///
/// assert!(!paths_exist!(path_1, path_2));
/// ```
#[macro_export]
macro_rules! paths_exist {
    ($($x:expr),*) => {'block: {
        $(
            if !std::fs::metadata($x).is_ok() {
                break 'block false;
            };
        )*
        true
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

    #[derive(Display, EnumString)]
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

        #[tokio::test]
        async fn test_get_latest_tag_from_github() {
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

pub mod directory_stack {
    use miette::Context;

    use super::*;

    create_global_singleton!(DirStack, GLOBAL_DIR_STACK);

    /// Do not instantiate this struct directly. Use [DirStack::get_mut_singleton]
    /// instead.
    ///
    /// A stack-based directory manager that allows pushing and popping directories to
    /// change the current working directory of the process. It is inspired by the
    /// [`pushd`](https://fishshell.com/docs/current/cmds/pushd.html) and
    /// [`popd`](https://fishshell.com/docs/current/cmds/popd.html) commands in `fish`
    /// shell.
    ///
    /// Even though this code is thread safe, the semantics of manipulating a single
    /// global pwd might not be something that you want to do. Here's an example
    /// demonstrating this for just two threads:
    /// 1. Thread 1 pushes Folder A and it takes 500 ms to complete.
    /// 2. Thread 2 starts with a 10 ms delay, and pushes Folder B and it takes 1000 ms to
    ///    complete.
    /// 3. Since the pwd is set globally for a the process it will change a lot over time,
    ///    and this is probably not what you wanted to happen:
    ///   - 0ms: Folder A
    ///   - 10ms: Folder B
    ///   - 510ms Folder A
    ///   - 1000ms: OG folder
    #[derive(Debug, Clone, Default)]
    pub struct DirStack {
        pub inner: Vec<PathBuf>,
    }

    pub struct DirStackDropHandle;

    impl Drop for DirStackDropHandle {
        fn drop(&mut self) {
            if let Ok(dir_stack) = DirStack::get_mut_singleton() {
                _ = dir_stack.lock().unwrap().try_popd();
            }
        }
    }

    impl DirStack {
        /// Pushes the current directory onto the stack and changes the current working
        /// directory to the specified path.
        ///
        /// Returns the previous directory that was on the stack, and a
        /// [DirStackDropHandle] that will automatically pop the directory from the stack
        /// and change back to it when it goes out of scope.
        pub fn try_pushd(
            &mut self,
            dest_dir: impl AsRef<Path>,
        ) -> miette::Result<(PathBuf, DirStackDropHandle)> {
            // Save the current directory.
            let old_dir = directory_path::try_pwd()?;

            // Assert that dest_dir directory exists.
            directory_path::try_folder_exists(dest_dir.as_ref()).context(format!(
                "Can't pushd into directory '{}' that does not exist",
                directory_path::path_as_string(dest_dir.as_ref())
            ))?;

            // Change cwd for current process.
            env::set_current_dir(dest_dir).into_diagnostic()?;

            // Push the old cwd to the stack.
            self.inner.push(old_dir.clone());

            tracing::debug!("pwd after pushd" = ?directory_path::try_pwd());

            ok!((old_dir, DirStackDropHandle))
        }

        /// Pops the top directory from the stack and changes the current working
        /// directory to that directory if it exists. This is a private function that can
        /// only be invoked by the [DirStackDropHandle] when it goes out of scope.
        ///
        /// Returns the directory that was popped from the stack.
        fn try_popd(&mut self) -> miette::Result<Option<PathBuf>> {
            // Get the previous directory from the stack (if any).
            let maybe_prev_dir = self.inner.pop();

            // Change cwd for current process (if any).
            if let Some(ref prev_dir) = maybe_prev_dir {
                env::set_current_dir(prev_dir.clone()).into_diagnostic()?;
                tracing::debug!("pwd after popd" = ?directory_path::try_pwd());
            }

            ok!(maybe_prev_dir)
        }
    }

    #[cfg(test)]
    mod tests_directory_stack {
        use super::*;
        use directory_create::{try_mkdir, MkdirOptions};
        use directory_path::try_pwd;
        use directory_stack::DirStack;
        use r3bl_test_fixtures::create_temp_dir;

        /// Be careful when manipulating the current working directory in tests using
        /// [env::set_current_dir] or [DirStack::pushd], as it can affect other tests that
        /// run in parallel.
        #[serial_test::serial]
        #[test]
        fn test_pushd_and_auto_popd_on_drop() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            // Use mkdir to create a new directory.
            let tmp_root_dir = root.join("test_pushd_and_auto_popd_on_drop");
            try_mkdir(
                &tmp_root_dir,
                MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
            )
            .unwrap();
            assert!(tmp_root_dir.exists());

            // Save the current directory.
            let og_pwd = try_pwd().unwrap();

            // Push the temporary directory onto the stack and change to it.
            let dir_stack_singleton = DirStack::get_mut_singleton().unwrap();
            let (cwd_before_pushd, dir_stack_drop_handle) = dir_stack_singleton
                .lock()
                .unwrap()
                .try_pushd(tmp_root_dir.clone())
                .unwrap();
            assert_eq!(try_pwd().unwrap(), tmp_root_dir);

            // Drop the DirStackDropHandle to pop the directory from the stack and change back
            // to the original directory.
            drop(dir_stack_drop_handle);
            assert_eq!(try_pwd().unwrap(), og_pwd);
            assert_eq!(cwd_before_pushd, og_pwd);
        }

        /// Be careful when manipulating the current working directory in tests using
        /// [env::set_current_dir] or [DirStack::pushd], as it can affect other tests that
        /// run in parallel.
        #[serial_test::serial]
        #[test]
        fn test_pushd_and_popd() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            // Use mkdir to create a new directory.
            let tmp_root_dir = root.join("test_pushd_and_popd");
            try_mkdir(
                &tmp_root_dir,
                MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
            )
            .unwrap();
            assert!(tmp_root_dir.exists());

            // Save the current directory.
            let og_pwd = try_pwd().unwrap();

            // Push the temporary directory onto the stack and change to it.
            let dir_stack = DirStack::get_mut_singleton().unwrap();
            let (cwd_before_pushd, _dir_stack_drop_handle) = dir_stack
                .lock()
                .unwrap()
                .try_pushd(tmp_root_dir.clone())
                .unwrap();
            assert_eq!(try_pwd().unwrap(), tmp_root_dir);

            // Pop the directory from the stack and change back to the original directory.
            let it = dir_stack.lock().unwrap().try_popd().unwrap();
            assert_eq!(try_pwd().unwrap(), og_pwd);
            assert_eq!(it.clone().unwrap(), cwd_before_pushd);
            assert_eq!(it.unwrap(), og_pwd);

            // Pop stack again.
            let it = dir_stack.lock().unwrap().try_popd().unwrap();
            assert!(it.is_none());
        }
    }
}

pub mod directory_create {
    use super::*;

    pub enum MkdirOptions {
        CreateIntermediateFolders,
        CreateIntermediateFoldersOnlyIfNotExists,
        CreateIntermediateFoldersAndPurgeExisting,
    }

    /// Creates a new directory at the specified path. Depending on the [MkdirOptions] the
    /// folders can be created destructively or non-destructively.
    ///
    /// If any permissions issues occur or the directory can't be created due to
    /// inconsistent [MkdirOptions] then an error is returned.
    pub fn try_mkdir(new_dir: impl AsRef<Path>, mkdir: MkdirOptions) -> miette::Result<()> {
        let new_dir = new_dir.as_ref();
        let new_dir_display = directory_path::path_as_string(new_dir);

        match mkdir {
            MkdirOptions::CreateIntermediateFolders => {
                fs::create_dir_all(new_dir).into_diagnostic()
            }

            MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists => {
                if let Ok(true) = fs::exists(new_dir) {
                    miette::bail!("Directory already exists: '{}'", new_dir_display);
                }
                fs::create_dir_all(new_dir).into_diagnostic()
            }

            MkdirOptions::CreateIntermediateFoldersAndPurgeExisting => {
                match fs::exists(new_dir) {
                    Ok(true) => fs::remove_dir_all(new_dir).into_diagnostic()?,
                    Ok(false) => { /* nothing to do, since directory does not exist */ }
                    Err(err) => miette::bail!(
                        "Error '{:?}' while attempting to check if directory exists: '{}'",
                        err,
                        new_dir_display
                    ),
                }
                fs::create_dir_all(new_dir).into_diagnostic()
            }
        }
    }

    #[cfg(test)]
    mod tests_directory_create {
        use super::*;
        use directory_create::{try_mkdir, MkdirOptions};
        use r3bl_test_fixtures::create_temp_dir;

        #[test]
        fn test_create_clean_new_dir() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            // Create a temporary directory.
            let tmp_root_dir = root.join("test_create_clean_new_dir");
            try_mkdir(
                &tmp_root_dir,
                MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
            )
            .unwrap();

            // Create a new folder inside the temporary directory.
            let new_folder = tmp_root_dir.join("new_folder");
            try_mkdir(
                &new_folder,
                MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
            )
            .unwrap();
            assert!(new_folder.exists());

            // Try & fail to create the same folder again non destructively.
            assert!(try_mkdir(
                &new_folder,
                MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
            )
            .is_err());

            // Create a file inside the new folder.
            let file_path = new_folder.join("test_file.txt");
            fs::write(&file_path, "test").unwrap();
            assert!(file_path.exists());

            // Call `mkdir` again with destructive options and ensure the folder is clean.
            try_mkdir(
                &new_folder,
                MkdirOptions::CreateIntermediateFoldersAndPurgeExisting,
            )
            .unwrap();
            assert!(new_folder.exists());
            assert!(!file_path.exists());
        }
    }
}

/// Note that [PathBuf] is owned and [Path] is a slice into it.
/// - So replace `&`[PathBuf] with a `&`[Path].
/// - More details
///   [here](https://rust-lang.github.io/rust-clippy/master/index.html#ptr_arg).
pub mod directory_path {
    use super::*;

    /// Checks whether the folder exist. If won't provide any errors if there are
    /// permissions issues or the folder is invalid. Use [try_folder_exists] if you want
    /// to handle these errors.
    pub fn folder_exists(folder: impl AsRef<Path>) -> bool {
        fs::metadata(folder).is_ok_and(|metadata| metadata.is_dir())
    }

    /// Checks whether the file exists. If won't provide any errors if there are permissions
    /// issues or the file is invalid. Use [try_file_exists] if you want to handle these
    /// errors.
    pub fn file_exists(file: impl AsRef<Path>) -> bool {
        fs::metadata(file).is_ok_and(|metadata| metadata.is_file())
    }

    /// Checks whether the folder exist. If there are issues with permissions for folder
    /// access or invalid folder it will return an error. Use [folder_exists] if you want
    /// to ignore these errors.
    pub fn try_folder_exists(folder: impl AsRef<Path>) -> miette::Result<bool> {
        ok!(match fs::metadata(folder) {
            Ok(metadata) => metadata.is_dir(),
            Err(err) => miette::bail!(err),
        })
    }

    /// Checks whether the file exist. If there are issues with permissions for file access
    /// or invalid file it will return an error. Use [file_exists] if you want to ignore
    /// these errors.
    pub fn try_file_exists(file: impl AsRef<Path>) -> miette::Result<bool> {
        ok!(match fs::metadata(file) {
            Ok(metadata) => metadata.is_file(),
            Err(err) => miette::bail!(err),
        })
    }

    /// Returns the current working directory of the process as a [PathBuf] (owned). If
    /// there are issues with permissions for folder access or invalid folder it will
    /// return an error.
    ///
    /// - `bash` equivalent: `$(pwd)`
    /// - Eg: `PathBuf("/home/user/some/path")`
    pub fn try_pwd() -> miette::Result<PathBuf> {
        env::current_dir().into_diagnostic()
    }

    // 00: deprecated this (replace with path!)
    /// Joins the provided `sub_path` ([Path] slice) to the current working directory of
    /// the process to get a fully qualified [PathBuf] (owned).
    /// - `bash` equivalent: `"$(pwd)/$sub_path"`
    /// - Eg: `PathBuf("/home/user/some/path/sub_path")`
    pub fn try_fq_path_relative_to_pwd(sub_path: impl AsRef<Path>) -> miette::Result<PathBuf> {
        let pwd = try_pwd()?;
        let sub_path_ref = sub_path.as_ref();
        ok!(pwd.join(sub_path_ref))
    }

    /// Returns the [Path] slice as a string.
    /// - Eg: `"/home/user/some/path"`
    pub fn path_as_string(path: &Path) -> String {
        path.display().to_string()
    }

    #[cfg(test)]
    mod tests_directory_path {
        use super::*;
        use directory_path::try_pwd;
        use r3bl_test_fixtures::create_temp_dir;

        /// Be careful when manipulating the current working directory in tests using
        /// [env::set_current_dir], as it can affect other tests that run in parallel.
        #[serial_test::serial]
        #[test]
        fn test_pwd() {
            // Save the current directory.
            let og_pwd = try_pwd().unwrap();

            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            let new_dir = root.join("test_pwd");
            fs::create_dir_all(&new_dir).unwrap();
            env::set_current_dir(&new_dir).unwrap();

            let pwd = try_pwd().unwrap();
            assert!(pwd.exists());
            assert_eq!(pwd, new_dir);

            // Change back to the original directory.
            env::set_current_dir(&og_pwd).unwrap();
        }

        /// Be careful when manipulating the current working directory in tests
        /// [env::set_current_dir], as it can affect other tests that run in parallel.
        #[serial_test::serial]
        #[test]
        fn test_fq_path_relative_to_pwd() {
            // Save the current directory.
            let og_pwd = try_pwd().unwrap();

            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            let sub_path = "test_fq_path_relative_to_pwd";
            let new_dir = root.join(sub_path);
            fs::create_dir_all(&new_dir).unwrap();

            env::set_current_dir(&root).unwrap();

            println!("Current directory set to: {}", root);
            println!("Current directory is    : {}", try_pwd().unwrap().display());

            let fq_path = directory_path::try_fq_path_relative_to_pwd(sub_path).unwrap();

            println!("Sub directory created at: {}", fq_path.display());
            println!("Sub directory exists    : {}", fq_path.exists());

            assert!(fq_path.exists());

            // Change back to the original directory.
            env::set_current_dir(&og_pwd).unwrap();
        }

        /// Be careful when manipulating the current working directory in tests
        /// [env::set_current_dir], as it can affect other tests that run in parallel.
        #[serial_test::serial]
        #[test]
        fn test_path_as_string() {
            // Save the current directory.
            let og_pwd = try_pwd().unwrap();

            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            env::set_current_dir(&root).unwrap();

            let fq_path = directory_path::try_fq_path_relative_to_pwd("some_folder").unwrap();
            let fq_path_str = directory_path::path_as_string(&fq_path);

            assert_eq!(fq_path_str, fq_path.display().to_string());

            // Change back to the original directory.
            env::set_current_dir(&og_pwd).unwrap();
        }

        #[test]
        fn test_file_exists() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            let new_dir = root.join("test_file_exists_dir");
            fs::create_dir_all(&new_dir).unwrap();

            let new_file = new_dir.join("test_file_exists_file.txt");
            fs::write(&new_file, "test").unwrap();

            assert!(directory_path::try_file_exists(&new_file).unwrap());
            assert!(!directory_path::try_file_exists(&new_dir).unwrap());

            fs::remove_dir_all(&new_dir).unwrap();

            // Ensure that an invalid path returns an error.
            assert!(directory_path::try_file_exists(&new_file).is_err()); // This file does not exist.
            assert!(directory_path::try_file_exists(&new_dir).is_err()); // This folder does not exist.
        }

        #[test]
        fn test_folder_exists() {
            // Create the root temp dir.
            let root = create_temp_dir().unwrap();

            let new_dir = root.join("test_folder_exists_dir");
            fs::create_dir_all(&new_dir).unwrap();

            let new_file = new_dir.join("test_folder_exists_file.txt");
            fs::write(&new_file, "test").unwrap();

            assert!(directory_path::try_folder_exists(&new_dir).unwrap());
            assert!(!directory_path::try_folder_exists(&new_file).unwrap());

            fs::remove_dir_all(&new_dir).unwrap();

            // Ensure that an invalid path returns an error.
            assert!(directory_path::try_folder_exists(&new_file).is_err()); // This file does not exist.
            assert!(directory_path::try_folder_exists(&new_dir).is_err()); // This folder does not exist.
        }
    }
}

pub mod environment {
    use super::*;

    #[cfg(target_os = "windows")]
    const OS_SPECIFIC_ENV_PATH_SEPARATOR: &str = ";";
    #[cfg(not(target_os = "windows"))]
    const OS_SPECIFIC_ENV_PATH_SEPARATOR: &str = ":";

    #[derive(Display, EnumString)]
    pub enum EnvKeys {
        #[strum(serialize = "PATH")]
        Path,
    }

    pub fn try_get(key: EnvKeys) -> miette::Result<String> {
        env::var(key.to_string()).into_diagnostic()
    }

    pub fn try_get_path_from_env() -> miette::Result<String> {
        try_get(EnvKeys::Path)
    }

    pub fn try_get_path_prefixed(prefix_path: impl AsRef<Path>) -> miette::Result<String> {
        let path = try_get_path_from_env()?;
        let add_to_path = format!(
            "{}{}{}",
            prefix_path.as_ref().display(),
            OS_SPECIFIC_ENV_PATH_SEPARATOR,
            path
        );
        tracing::debug!("my_path" = %format!("{:.50}{}", add_to_path, "...<clip>".red()));
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
    /// - The `file` must exist and be a file (not a folder).
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
