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

//! # Example using `procspawn` to spawn processes
//!
//! The [`procspawn`](https://docs.rs/procspawn/latest/procspawn/) crate provides the
//! ability to spawn processes with a function similar to `thread::spawn`.
//!
//! - Unlike `thread::spawn` data cannot be passed by the use of closures.
//! - Instead if must be explicitly passed as serializable object (specifically it must be
//!   `serde` serializable). Internally, the data is serialized using
//!   [`bincode`](https://docs.rs/procspawn/latest/procspawn/#bincode-limitations).
//! - The return value from the spawned closure also must be serializable and can then be
//!   retrieved from the returned join handle.
//! - If the spawned function causes a panic it will also be serialized across the process
//!   boundaries.
//!
//! # Examples
//!
//! Great [examples](https://github.com/mitsuhiko/procspawn/tree/master/examples) from the
//! official docs.
//!
//! # Run the binary
//!
//! ```text
//! ┌─────────────────────────────┐
//! │ > cargo run --bin procspawn │
//! └─────────────────────────────┘
//! ```

use miette::IntoDiagnostic;
use r3bl_rs_utils_core::ok;

fn main() -> miette::Result<()> {
    // A spawned process will execute every line of code up to this point.
    procspawn::init();

    let pid_parent = std::process::id();

    let args: Vec<i64> = vec![1, 2, 3, 4];
    let (sum, pid_child, pid_child_from_clip) = configure_builder()
        .spawn(args, run_in_child_process)
        .join()
        .into_diagnostic()?
        .into_diagnostic()?;

    println!("Parent PID: {}", pid_parent);
    println!(
        "Child PID: {}, sum: {}, pid from clip: {}",
        pid_child, sum, pid_child_from_clip
    );

    assert_eq!(sum, 10);
    assert_eq!(pid_child, pid_child_from_clip);

    ok!()
}

// Create a new builder with stderr & stdout that's null.
fn configure_builder() -> procspawn::Builder {
    let mut it = procspawn::Builder::new();
    it.stderr(std::process::Stdio::null()); // Suppress stderr.
    it.stdout(std::process::Stdio::null()); // Suppress stdout.
    it
}

// This function will be executed in a child process.
fn run_in_child_process(
    /* serde */ param: Vec<i64>,
) -> std::result::Result<
    /* serde - Ok variant */
    (
        /* sum */ i64,
        /* pid */ String,
        /* pid from clip */ String,
    ),
    /* serde - Err variant */
    ClipboardError,
> {
    let pid_child = std::process::id();
    let sum = param.iter().sum();

    // Copy child pid to the clipboard.
    use cli_clipboard::ClipboardProvider as _; // Import `ClipboardProvider` trait.
    let mut ctx =
        cli_clipboard::ClipboardContext::new().map_err(|_| ClipboardError::ContextUnavailable)?;
    ctx.set_contents(pid_child.to_string().to_owned())
        .map_err(|_| ClipboardError::SetContents)?;
    let pid_child_from_clip = ctx
        .get_contents()
        .map_err(|_| ClipboardError::GetContents)?;

    Ok((sum, pid_child.to_string(), pid_child_from_clip))
}

#[derive(Debug, serde::Deserialize, serde::Serialize, thiserror::Error)]
pub enum ClipboardError {
    #[error("clipboard context unavailable")]
    ContextUnavailable,

    #[error("could not get clipboard contents")]
    GetContents,

    #[error("could not set clipboard contents")]
    SetContents,
}
