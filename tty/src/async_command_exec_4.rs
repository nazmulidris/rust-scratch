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

use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use r3bl_rs_utils_core::ok;
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

/// In this example, we will will orchestrate two processes and make a pipe between them
/// programmatically (we are used to doing this using `|` in shells). We will replicate
/// the following functionality in this program: `echo hello world | tr a-z A-Z`.
///
/// 1. Spawn the `echo` command, with arg `hello world` and get its `stdout`.
/// 2. Then we will provide this `stdout` to the `stdin` of the `tr` command, with arg
///    `a-z A-Z` and spawn it.
/// 3. Finally we join the `echo` and `tr` child processes and wait for them both to
///    complete.
///
/// # Run the binary
/// 
/// ```text
/// ┌────────────────────────────────────────┐
/// │ > cargo run --bin async_command_exec_4 │
/// └────────────────────────────────────────┘
/// ```

#[tokio::main]
async fn main() -> miette::Result<()> {
    // Spawn `echo` command & get its `stdout`.
    let (child_stdout_echo, join_handle_echo) = spawn_child_echo_and_get_stdout()?;

    // Spawn `tr` command & pass the `stdout
    let (child_stdout_tr, join_handle_tr) = spawn_child_tr_and_pipe_to_stdin(child_stdout_echo)?;

    // Wait for both child processes to complete.
    let (_, _) = tokio::join!(join_handle_echo, join_handle_tr);

    // Read the output of the `tr` command. And make assertions.
    let mut output = vec![];
    tokio::io::BufReader::new(child_stdout_tr)
        .read_to_end(&mut output)
        .await
        .into_diagnostic()?;
    assert_eq!(output, b"HELLO WORLD!\n");

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
fn spawn_child_echo_and_get_stdout(
) -> miette::Result<(tokio::process::ChildStdout, tokio::task::JoinHandle<()>)> {
    // Spawn child process.
    let mut echo_child = Command::new("echo")
        .arg("hello world!")
        .stdout(Stdio::piped())
        .spawn()
        .into_diagnostic()?;

    // Get the `stdout` of the child process.
    let child_stdout = echo_child
        .stdout
        .take()
        .ok_or(miette::miette!("no stdout"))?;

    // Ensure the child process is spawned in the runtime, so it can make progress on its
    // own while we await any output.
    let join_handle = tokio::task::spawn(async move {
        _ = echo_child.wait().await;
    });

    Ok((child_stdout, join_handle))
}

/// Spawn `tr` command & pass the given [tokio::process::ChildStdout] to its `stdin`.
///
/// Return a tuple of:
/// 1. `stdout` of `tr`: [tokio::process::ChildStdout].
/// 2. [tokio::task::JoinHandle] of `tr` [tokio::process::Child] process, spawned by the
///    [tokio::process::Command] that starts `tr`.
fn spawn_child_tr_and_pipe_to_stdin(
    stdout_from_other_child: tokio::process::ChildStdout,
) -> miette::Result<(tokio::process::ChildStdout, tokio::task::JoinHandle<()>)> {
    // Note the type conversion here, from a `tokio::process::ChildStdout` to a
    // `std::process::Stdio`.
    let stdout_from_other_child: std::process::Stdio =
        stdout_from_other_child.try_into().into_diagnostic()?;

    // Spawn child process.
    let mut tr_child = Command::new("tr")
        .arg("a-z")
        .arg("A-Z")
        .stdin(stdout_from_other_child) // Pipe stdout into this child's stdin.
        .stdout(Stdio::piped())
        .spawn()
        .into_diagnostic()?;

    // Get the `stdout` of the child process.
    let child_stdout = tr_child.stdout.take().ok_or(miette::miette!("no stdout"))?;

    // Ensure the child process is spawned in the runtime, so it can make progress on its
    // own while we await any output.
    let join_handle = tokio::task::spawn(async move {
        _ = tr_child.wait().await;
    });

    Ok((child_stdout, join_handle))
}
