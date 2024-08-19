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

//! # Example of programmatically piping the output of one process into another
//!
//! In this example, we will will orchestrate two processes and make a pipe between them
//! programmatically (we are used to doing this using `|` in shells). We will replicate
//! the following functionality in this program: `echo hello world | tr a-z A-Z`.
//!
//! 1. Spawn the `echo` command, with arg `hello world` and get its `stdout`.
//! 2. Then we will provide this `stdout` to the `stdin` of the `tr` command, with arg
//!    `a-z A-Z` and spawn it.
//! 3. Finally we join the `echo` and `tr` child processes and wait for them both to
//!    complete.
//!
//! # Run the binary
//!
//! ```text
//! ┌────────────────────────────────────────┐
//! │ > cargo run --bin async_command_exec_4 │
//! └────────────────────────────────────────┘
//! ```

use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use r3bl_rs_utils_core::ok;
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

type EchoResult = (tokio::process::ChildStdout, tokio::task::JoinHandle<()>);
type TrResult = (tokio::process::ChildStdout, tokio::task::JoinHandle<()>);

const INPUT: &str = "hello world";

#[tokio::main]
async fn main() -> miette::Result<()> {
    // Spawn the `echo` command & get its `stdout`.
    let (child_stdout_echo, join_handle_echo): EchoResult = spawn_child_echo_and_get_stdout()?;

    // Spawn the `tr` command & provide the `stdout` of `echo` to its `stdin`.
    let (child_stdout_tr, join_handle_tr): TrResult =
        spawn_child_tr_and_provide_stdin(child_stdout_echo)?;

    // Wait for both child processes to complete.
    _ = tokio::try_join!(join_handle_echo, join_handle_tr);

    // Read the output of the `tr` command from `child_stdout_tr`.
    let output = {
        let mut buf = vec![];
        tokio::io::BufReader::new(child_stdout_tr)
            .read_to_end(&mut buf)
            .await
            .into_diagnostic()?;
        buf
    };

    // Make assertions.
    let expected_output = format!("{INPUT}\n").to_uppercase();
    assert_eq!(expected_output, String::from_utf8_lossy(&output));

    // Print the output of the `tr` command.
    println!(
        "{}: {}",
        "output".blue(),
        format!("{}", String::from_utf8_lossy(&output)).green()
    );

    ok!()
}

/// 🚀 Spawn `echo` command & get its `stdout`. We will pipe this into the `stdin` of
/// `tr`.
///
/// Return a tuple of:
/// 1. `stdout` of `echo`: [tokio::process::ChildStdout].
/// 2. [tokio::task::JoinHandle] of `echo` [tokio::process::Child] process, spawned by the
///    [tokio::process::Command] that starts `echo`.
fn spawn_child_echo_and_get_stdout() -> miette::Result<EchoResult> {
    // Spawn the child process for `echo`.
    let mut child_echo = Command::new("echo")
        .arg(INPUT)
        .stdout(Stdio::piped())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .into_diagnostic()?;

    // Take the `stdout` of the child process.
    let child_stdout = child_echo.stdout.take().ok_or(miette::miette!(
        "Failed to capture stdout of `echo` child process"
    ))?;

    // Ensure the child process is spawned in the runtime, so it can make progress on its
    // own while we await any output.
    let join_handle = tokio::spawn(async move {
        _ = child_echo.wait().await;
    });

    // Return the `stdout` of `echo` and the `JoinHandle` of the `echo` child process.
    Ok((child_stdout, join_handle))
}

/// 🚀 Spawn `tr` command & pass the given [tokio::process::ChildStdout] to its `stdin`.
///
/// Return a tuple of:
/// 1. `stdout` of `tr`: [tokio::process::ChildStdout].
/// 2. [tokio::task::JoinHandle] of `tr` [tokio::process::Child] process, spawned by the
///    [tokio::process::Command] that starts `tr`.
fn spawn_child_tr_and_provide_stdin(
    stdout_from_other_child: tokio::process::ChildStdout,
) -> miette::Result<TrResult> {
    // Convert `stdout_from_other_child`: tokio::process::ChildStdout into
    // tokio::process::ChildStdin, so it can be provided to the `stdin` of the `tr`
    // command.
    let stdout_from_other_child: std::process::Stdio =
        stdout_from_other_child.try_into().into_diagnostic()?;

    // Spawn child process.
    let mut child_tr = Command::new("tr")
        .arg("a-z")
        .arg("A-Z")
        .stdin(stdout_from_other_child)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .into_diagnostic()?;

    // Take the `stdout` of the child process.
    let child_stdout = child_tr.stdout.take().ok_or(miette::miette!(
        "Failed to capture stdout of `tr` child process"
    ))?;

    // Ensure the child process is spawned in the runtime, so it can make progress on its
    // own while we await any output.
    let join_handle = tokio::spawn(async move {
        _ = child_tr.wait().await;
    });

    Ok((child_stdout, join_handle))
}
