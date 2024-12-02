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
    env, fs,
    io::Write as _,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
};
use strum_macros::{Display, EnumString};

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
    use super::*;

    pub mod constants {
        pub const TAG_NAME: &str = "tag_name";
        pub const VERSION_PREFIX: &str = "v";
    }

    pub mod urls {
        pub const REPO_LATEST_RELEASE: &str =
            "https://api.github.com/repos/{org}/{repo}/releases/latest";
    }

    pub async fn get_latest_release_tag_from_github(
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
}

#[cfg(test)]
mod tests_github_api {
    use super::*;
    use github_api::get_latest_release_tag_from_github;

    #[tokio::test]
    async fn test_get_latest_tag_from_github() {
        let org = "cloudflare";
        let repo = "cfssl";
        let tag = get_latest_release_tag_from_github(org, repo).await.unwrap();
        assert!(!tag.is_empty());
        println!("Latest tag: {}", tag.magenta());
    }
}

pub mod directory_stack {
    use super::*;

    create_global_singleton!(Stack, GLOBAL_DIR_STACK);

    /// Do not instantiate this struct directly. Use [Stack::get_mut_singleton] instead.
    ///
    /// A stack-based directory manager that allows pushing and popping directories to change
    /// the current working directory of the process. It is inspired by the
    /// [`pushd`](https://fishshell.com/docs/current/cmds/pushd.html) and
    /// [`popd`](https://fishshell.com/docs/current/cmds/popd.html) commands in `fish` shell.
    #[derive(Debug, Clone, Default)]
    pub struct Stack {
        pub inner: Vec<PathBuf>,
    }

    impl Stack {
        /// Pushes the current directory onto the stack and changes the current working
        /// directory to the specified path.
        ///
        /// Returns the previous directory that was on the stack.
        pub fn pushd(&mut self, new_dir: PathBuf) -> miette::Result<PathBuf> {
            // Save the current directory.
            let old_dir = path::pwd()?;

            // Change cwd for current process.
            env::set_current_dir(new_dir).into_diagnostic()?;

            // Push the old cwd to the stack.
            self.inner.push(old_dir.clone());

            ok!(old_dir)
        }

        /// Pops the top directory from the stack and changes the current working directory to
        /// that directory if it exists.
        ///
        /// Returns the directory that was popped from the stack.
        pub fn popd(&mut self) -> miette::Result<Option<PathBuf>> {
            // Get the previous directory from the stack (if any).
            let maybe_prev_dir = self.inner.pop();

            // Change cwd for current process (if any).
            if let Some(ref prev_dir) = maybe_prev_dir {
                env::set_current_dir(prev_dir.clone()).into_diagnostic()?;
            }

            ok!(maybe_prev_dir)
        }
    }
}

#[cfg(test)]
mod tests_directory_stack {
    use super::*;
    use directory::{mkdir, MkdirOptions};
    use directory_stack::Stack;
    use path::pwd;

    #[test]
    fn test_pushd_and_popd() {
        let dir_stack = Stack::get_mut_singleton().unwrap();

        // Create a temporary directory.
        let tmp_root_dir = env::temp_dir().join("test_pushd_and_pop");
        mkdir(
            &tmp_root_dir,
            MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
        )
        .unwrap();
        assert!(tmp_root_dir.exists());

        // Save the current directory.
        let og_cwd = pwd().unwrap();

        // Push the temporary directory onto the stack and change to it.
        let cwd_before_pushd = dir_stack
            .lock()
            .unwrap()
            .pushd(tmp_root_dir.clone())
            .unwrap();
        assert_eq!(pwd().unwrap(), tmp_root_dir);

        // Pop the directory from the stack and change back to the original directory.
        let it = dir_stack.lock().unwrap().popd().unwrap();
        assert_eq!(pwd().unwrap(), og_cwd);
        assert_eq!(it.clone().unwrap(), cwd_before_pushd);
        assert_eq!(it.unwrap(), og_cwd);

        // Pop stack again.
        let it = dir_stack.lock().unwrap().popd().unwrap();
        assert!(it.is_none());

        // Clean up.
        fs::remove_dir_all(tmp_root_dir).unwrap();
    }
}

pub mod directory {
    use super::*;

    pub enum MkdirOptions {
        CreateIntermediateFolders,
        CreateIntermediateFoldersOnlyIfNotExists,
        CreateIntermediateFoldersAndPurgeExisting,
    }

    pub fn mkdir(new_dir: impl AsRef<Path>, mkdir: MkdirOptions) -> miette::Result<()> {
        let new_dir = new_dir.as_ref();
        let new_dir_display = path::path_as_string(new_dir);

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
}

#[cfg(test)]
mod tests_directory {
    use super::*;
    use directory::{mkdir, MkdirOptions};

    #[test]
    fn test_create_clean_new_dir() {
        // Create a temporary directory.
        let tmp_root_dir = env::temp_dir().join("test_create_clean_new_dir");
        mkdir(
            &tmp_root_dir,
            MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
        )
        .unwrap();

        // Create a new folder inside the temporary directory.
        let new_folder = tmp_root_dir.join("new_folder");
        mkdir(
            &new_folder,
            MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
        )
        .unwrap();
        assert!(new_folder.exists());

        // Try & fail to create the same folder again non destructively.
        assert!(mkdir(
            &new_folder,
            MkdirOptions::CreateIntermediateFoldersOnlyIfNotExists,
        )
        .is_err());

        // Create a file inside the new folder.
        let file_path = new_folder.join("test_file.txt");
        fs::write(&file_path, "test").unwrap();
        assert!(file_path.exists());

        // Call `mkdir` again with destructive options and ensure the folder is clean.
        mkdir(
            &new_folder,
            MkdirOptions::CreateIntermediateFoldersAndPurgeExisting,
        )
        .unwrap();
        assert!(new_folder.exists());
        assert!(!file_path.exists());

        // Clean up.
        fs::remove_dir_all(tmp_root_dir).unwrap();
    }
}

/// Note that [PathBuf] is owned and [Path] is a slice into it.
/// - So replace `&`[PathBuf] with a `&`[Path].
/// - More details
///   [here](https://rust-lang.github.io/rust-clippy/master/index.html#ptr_arg).
pub mod path {
    use super::*;

    pub fn folder_exists(folder: impl AsRef<Path>) -> miette::Result<bool> {
        ok!(match fs::metadata(folder) {
            Ok(metadata) => metadata.is_dir(),
            Err(err) => miette::bail!(
                "Error '{:?}' while attempting to check if folder exists",
                err
            ),
        })
    }

    pub fn file_exists(file: impl AsRef<Path>) -> miette::Result<bool> {
        ok!(match fs::metadata(file) {
            Ok(metadata) => metadata.is_file(),
            Err(err) => miette::bail!("Error '{:?}' while attempting to check if file exists", err),
        })
    }

    /// Returns the current working directory of the process as a [PathBuf] (owned).
    /// - `bash` equivalent: `$(pwd)`
    /// - Eg: `PathBuf("/home/user/some/path")`
    pub fn pwd() -> miette::Result<PathBuf> {
        env::current_dir().into_diagnostic()
    }

    /// Joins the provided `sub_path` ([Path] slice) to the current working directory of
    /// the process to get a fully qualified [PathBuf] (owned).
    /// - `bash` equivalent: `"$(pwd)/$sub_path"`
    /// - Eg: `PathBuf("/home/user/some/path/sub_path")`
    pub fn fq_path_relative_to_pwd(sub_path: impl AsRef<Path>) -> miette::Result<PathBuf> {
        let pwd = pwd()?;
        let sub_path_ref = sub_path.as_ref();
        ok!(pwd.join(sub_path_ref))
    }

    /// Returns the [Path] slice as a string.
    /// - Eg: `"/home/user/some/path"`
    pub fn path_as_string(path: &Path) -> String {
        path.display().to_string()
    }
}

#[cfg(test)]
mod tests_path {
    use super::*;
    use path::pwd;

    #[test]
    fn test_pwd() {
        let root = env::temp_dir();

        let new_dir = root.join("test_pwd");
        fs::create_dir_all(&new_dir).unwrap();
        env::set_current_dir(&new_dir).unwrap();

        let pwd = pwd().unwrap();
        assert!(pwd.exists());
        assert_eq!(pwd, new_dir);

        fs::remove_dir_all(&new_dir).unwrap();
    }

    #[test]
    fn test_fq_path_relative_to_pwd() {
        let root = env::temp_dir();

        let sub_path = "test_fq_path_relative_to_pwd";
        let new_dir = root.join(sub_path);
        fs::create_dir_all(&new_dir).unwrap();

        env::set_current_dir(&root).unwrap();
        let fq_path = path::fq_path_relative_to_pwd(sub_path).unwrap();
        assert!(fq_path.exists());

        fs::remove_dir_all(&new_dir).unwrap();
    }

    #[test]
    fn test_path_as_string() {
        let root = env::temp_dir();
        env::set_current_dir(&root).unwrap();

        let fq_path = path::fq_path_relative_to_pwd("some_folder").unwrap();
        let fq_path_str = path::path_as_string(&fq_path);

        assert_eq!(fq_path_str, fq_path.display().to_string());
    }

    #[test]
    fn test_file_exists() {
        let root = env::temp_dir();

        let new_dir = root.join("test_file_exists_dir");
        fs::create_dir_all(&new_dir).unwrap();

        let new_file = new_dir.join("test_file_exists_file.txt");
        fs::write(&new_file, "test").unwrap();

        assert!(path::file_exists(&new_file).unwrap());
        assert!(!path::file_exists(&new_dir).unwrap());

        fs::remove_dir_all(&new_dir).unwrap();

        // Ensure that an invalid path returns an error.
        assert!(path::file_exists(&new_file).is_err()); // This file does not exist.
        assert!(path::file_exists(&new_dir).is_err()); // This folder does not exist.
    }

    #[test]
    fn test_folder_exists() {
        let root = env::temp_dir();

        let new_dir = root.join("test_folder_exists_dir");
        fs::create_dir_all(&new_dir).unwrap();

        let new_file = new_dir.join("test_folder_exists_file.txt");
        fs::write(&new_file, "test").unwrap();

        assert!(path::folder_exists(&new_dir).unwrap());
        assert!(!path::folder_exists(&new_file).unwrap());

        fs::remove_dir_all(&new_dir).unwrap();

        // Ensure that an invalid path returns an error.
        assert!(path::folder_exists(&new_file).is_err()); // This file does not exist.
        assert!(path::folder_exists(&new_dir).is_err()); // This folder does not exist.
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

    pub fn get(key: EnvKeys) -> miette::Result<String> {
        env::var(key.to_string()).into_diagnostic()
    }

    pub fn get_path() -> miette::Result<String> {
        get(EnvKeys::Path)
    }

    pub fn get_path_prefixed(prefix_path: impl AsRef<Path>) -> miette::Result<String> {
        let path = get_path()?;
        ok!(format!(
            "{}{}{}",
            prefix_path.as_ref().display(),
            OS_SPECIFIC_ENV_PATH_SEPARATOR,
            path
        ))
    }
}

#[cfg(test)]
mod tests_environment {
    use super::*;

    #[test]
    fn test_get_path() {
        let path = environment::get_path().unwrap();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_get_path_prefixed() {
        let prefix_path = "/usr/bin";
        let path = environment::get_path_prefixed(prefix_path).unwrap();
        assert!(!path.is_empty());
        assert!(path.starts_with(prefix_path));
    }
}

pub mod download {
    use super::*;
    use http_client::create_client_with_user_agent;

    pub async fn download_file_overwrite_existing(
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
}

#[cfg(test)]
mod tests_download {
    use super::*;
    use download::download_file_overwrite_existing;

    #[tokio::test]
    async fn test_download_file_overwrite_existing() {
        let root = env::temp_dir();

        let new_dir = root.join("test_download_file_overwrite_existing");
        fs::create_dir_all(&new_dir).unwrap();

        let source_url = "https://developerlife.com";
        let destination_file = new_dir.join("test_download_file_overwrite_existing.html");

        // Download file (no pre-existing file).
        download_file_overwrite_existing(source_url, &destination_file)
            .await
            .unwrap();
        assert!(destination_file.exists());

        let meta_data = destination_file.metadata().unwrap();
        let og_file_size = meta_data.len();

        // Download file again (overwrite existing).
        download_file_overwrite_existing(source_url, &destination_file)
            .await
            .unwrap();
        assert!(destination_file.exists());

        // Ensure that the file sizes are the same.
        let meta_data = destination_file.metadata().unwrap();
        let new_file_size = meta_data.len();
        assert_eq!(og_file_size, new_file_size);

        fs::remove_file(&destination_file).unwrap();

        fs::remove_dir_all(&new_dir).unwrap();
    }
}

pub mod permissions {
    use super::*;

    /// Sets the file at the specified path to be executable by owner, group, and others.
    /// - `bash` equivalent: `chmod +x file`
    /// - Eg: `set_file_executable("some_file.sh")`
    /// - The `file` must exist and be a file (not a folder).
    pub fn set_file_executable(file: impl AsRef<Path>) -> miette::Result<()> {
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
}

#[cfg(test)]
mod tests_permissions {
    use super::*;
    use permissions::set_file_executable;

    #[test]
    fn test_set_file_executable() {
        let root = env::temp_dir();

        let new_dir = root.join("test_set_file_executable");
        fs::create_dir_all(&new_dir).unwrap();

        let new_file = new_dir.join("test_set_file_executable.sh");
        fs::write(&new_file, "echo 'Hello, World!'").unwrap();

        set_file_executable(&new_file).unwrap();

        let metadata = fs::metadata(&new_file).unwrap();
        let lhs = metadata.permissions();

        // Assert that the file has executable permission for owner, group, and others:
        // - The bitwise AND operation (lhs.mode() & 0o777) ensures that only the
        //   permission bits are compared, ignoring other bits that might be present in
        //   the mode.
        // - The assertion checks if the permission bits match 0o755.
        assert_eq!(lhs.mode() & 0o777, 0o755);

        fs::remove_dir_all(&new_dir).unwrap();
    }

    #[test]
    fn test_set_file_executable_on_non_file() {
        let root = env::temp_dir();

        let new_dir = root.join("test_set_file_executable_on_non_file");
        fs::create_dir_all(&new_dir).unwrap();

        let result = set_file_executable(&new_dir);
        assert!(result.is_err());

        fs::remove_dir_all(&new_dir).unwrap();
    }

    #[test]
    fn test_set_file_executable_on_non_existent_file() {
        let root = env::temp_dir();

        let new_dir = root.join("test_set_file_executable_on_non_existent_file");
        fs::create_dir_all(&new_dir).unwrap();

        let non_existent_file = new_dir.join("non_existent_file.sh");
        let result = set_file_executable(&non_existent_file);
        assert!(result.is_err());

        fs::remove_dir_all(&new_dir).unwrap();
    }
}
