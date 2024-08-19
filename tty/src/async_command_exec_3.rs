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

//! # Example of programmatically providing input into `stdin` and getting output from `stdout` of a process
//!
//! This example is similar to `async_command_exec_2.rs`, except that there is no need to
//! pipe input from the shell into this program. It does the following:
//! 1. Programmatically provides data to the `cat` command via `stdin`.
//! 2. Programmatically captures the output of `cat` via `stdout`.
//!
//! You can run `cargo run --bin async_command_exec_3` and it will print the output.
//!
//! > Note: Look at code example of [tokio::process::Child::wait] to see how to write /
//! > send data to the `stdin` of a command.
//!
//! # Flow of what happens
//!
//! ```text
//! Terminal emulator running fish/bash shell
//! ┌────────────────────────────────────────┐
//! │ > cargo run --bin async_command_exec_3 │
//! └───▲────────────────────────────────────┘
//!     ├────► create async Command for `cat`
//!     │      with stdout = `Stdio::piped()` to
//!     │      capture the output of `cmd`
//!     │      back into this program
//!     │
//!     ├────► set stdin = `Stdio::piped()` to provide
//!     │      input to the `cat` command asynchronously
//!     │
//!     ├────► `cmd.spawn()` then sets up the `cat` process
//!     │      to run with the given stdin & stdout and
//!     │      returns a `Child` struct
//!     │
//!     ├────► 🚀 instead of waiting "normally", we must use
//!     │      `tokio::spawn` to call `child.wait().await`
//!     │      on the child so it can make progress while
//!     │      we wait for its output below (in the current task)
//!     │
//!     ├────► 🚀 also use `tokio::spawn` to call `child.stdin.write_all()`
//!     │      to provide input to the `cat` command
//!     │
//!     └────► in our current task, we can now access `stdout`
//!            WHILE the child task is making progress above
//! ```
//!
//! # Kill child processes
//!
//! Note that similar to the behavior to the standard library, and unlike the futures
//! paradigm of dropping-implies-cancellation, a spawned process will, by default,
//! continue to execute even after the [tokio::process::Child] handle has been dropped.
//! [Docs](https://docs.rs/tokio/latest/tokio/process/index.html#caveats). To change this
//! behavior you can use [tokio::process::Command::kill_on_drop] which isn't really
//! recommended.
//!
//! Instead, to kill a child process, you can do the following:
//! - [tokio::process::Child::kill] - This forces the child process to exit.
//! - [tokio::process::Child::wait] - This waits for the child process to cleanly exit.

use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use r3bl_rs_utils_core::ok;
use std::process::Stdio;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout},
    task::JoinHandle,
};

#[tokio::main]
async fn main() -> miette::Result<()> {
    // Create a child process that runs `cat`.
    // 1. Send the output of `cat` back to this child process.
    // 2. Send the input to `cat` from this child process.
    // 3. This child / command does not make progress until `wait().await` is called.
    let mut child = tokio::process::Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .into_diagnostic()?;

    // These are the bytes that will be sent to the `stdin` of the child process.
    let input = &["hello", "nadia!"];

    // Get the stdout & stdin of the child process. Do this before the next step, because
    // the `child` struct is moved into the closure.
    let (stdout, stdin): (ChildStdout, ChildStdin) = {
        let Some(stdout) = child.stdout.take() else {
            miette::bail!("Child process did not have a stdout");
        };
        let Some(stdin) = child.stdin.take() else {
            miette::bail!("Child process did not have a stdin");
        };
        (stdout, stdin)
    };

    // Spawn tasks to:
    let join_handle_child_task = spawn_child_process(child);
    let join_handle_provide_input_task = spawn_provide_input(stdin, input);

    // Read the output of the child process, on the current thread.
    _ = read_stdout(stdout).await;

    // Wait for the child process to complete.
    _ = tokio::join!(join_handle_child_task, join_handle_provide_input_task);

    // Make assertions.
    assert_eq!(input.join("\n"), "hello\nnadia!");

    ok!()
}

/// As long as there is a line to be read from the child process, print it to the
/// terminal.
async fn read_stdout(stdout: ChildStdout) -> miette::Result<()> {
    let mut output: Vec<String> = vec![];
    let mut stdout_reader = BufReader::new(stdout).lines();
    while let Some(line) = stdout_reader.next_line().await.into_diagnostic()? {
        output.push(line.clone());
        println!("🧵 read_stdout -> {}", format!("🫲  {}", line).cyan());
    }
    ok!()
}

/// 🚀 Ensure the child process is spawned in the runtime, so it can make progress on its
/// own while we await any output.
fn spawn_child_process(mut child: Child) -> JoinHandle<()> {
    tokio::spawn(async move {
        let result_exit_status = child.wait().await;
        println!(
            "{}",
            format!(
                "🚀 spawn_child_process -> Child process exited with status: {:?}",
                result_exit_status
            )
            .green()
        );
    })
}

/// 🚀 Provide input to the child process.
fn spawn_provide_input(mut stdin: ChildStdin, input: &[&str]) -> JoinHandle<()> {
    let input = input
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    tokio::spawn(async move {
        // Write the input to the `stdin` of the child process.
        _ = stdin.write_all(input.as_bytes()).await;

        // Drop the handle to signal EOF to the child process.
        drop(stdin);

        println!(
            "{}: {}",
            "🚀 spawn_provide_input -> Finished providing input + EOF to child process 🫱  stdin"
                .green(),
            format!("{:?}", input).blue()
        );
    })
}
