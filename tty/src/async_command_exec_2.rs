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

//! # Example of piping input to `cat` process programmatically
//!
//! This example uses the
//! [`tokio::process::Command`](https://docs.rs/tokio/latest/tokio/process/index.html)
//! struct to execute a command asynchronously, and then pipes the output of this command,
//! back to itself. Then prints the output one line at a time.
//!
//! - To run this program, pipe some input (from the shell) into this program.
//!   ```fish
//!   echo -e "hello world\nfoo\nbar\n" | cargo run --bin async_command_exec_2
//!   ```
//! - This process will then run `cat` and capture the output from `cat`.
//! - It will then print the output from `cat` one line at time to the terminal.
//!
//! # Flow of what happens
//!
//! ```text
//! Terminal emulator running fish/bash shell
//! ┌──────────────────────────────────────────────────────────────────┐
//! │ > echo -e "foo\nbar\nbaz" | cargo run --bin async_command_exec_2 │
//! └───────▲─────────────────────▲────────────────────────────────────┘
//!         │                     │        Pipeline above runs
//!         │                     │        in parallel
//!    external                 external
//!    process                  process
//!    command (fork & exec)    command (fork & exec)
//!                               │
//!                               ├────► create async Command for `cat`
//!                               │      with stdout = `Stdio::piped()` to
//!                               │      capture the output of `cmd`
//!                               │      back into this program
//!                               │
//!                               ├────► the stdin for this Command is
//!                               │      inherited from the current process
//!                               │      which is provided by the terminal
//!                               │      and `pipe`
//!                               │
//!                               ├────► `cmd.spawn()` then sets up the `cat`
//!                               │      process to run with the given stdin
//!                               │      & stdout and returns a `Child` struct
//!                               │
//!                               ├────► 🚀 instead of waiting "normally", we
//!                               │      must use `tokio::spawn` to call
//!                               │      `child.wait().await` on the child so
//!                               │      it can make progress while we wait for
//!                               │      its output below (in the current task)
//!                               │
//!                               └────► in our current task, we can now access
//!                                      `stdout` WHILE the child task is
//!                                      making progress above
//! ```
//!
//! # Kill child processes
//!
//! Note that similar to the behavior to the standard library, and unlike the futures
//! paradigm of dropping-implies-cancellation, a spawned process will, by default,
//! continue to execute even after the [tokio::process::Child] handle has been dropped.
//! More info in the
//! [docs](https://docs.rs/tokio/latest/tokio/process/index.html#caveats). To change this
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
use tokio::io::{AsyncBufReadExt, BufReader};

/// This variant requires the use of `tokio::spawn` to wait for the child process to
/// complete.
#[tokio::main]
async fn main() -> miette::Result<()> {
    // Create a child process that runs `cat`.
    // - Send the output of `cat` back to this child process.
    // - This child / command does not make progress until `wait().await` is called.
    let mut child = tokio::process::Command::new("cat")
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .into_diagnostic()?;

    // Get the stdout of the child process. Do this before the next step, because the
    // `child` struct is moved into the closure.
    let Some(child_stdout) = child.stdout.take() else {
        miette::bail!("Failed to capture stdout of child process");
    };

    // 🚀 Ensure the child process is spawned in the runtime, so it can make progress on its
    // own while we await any output.
    let child_task_join_handle = tokio::spawn(async move {
        let result_exit_status = child.wait().await;
        println!(
            "{}",
            format!("Child process exited with status: {:?}", result_exit_status).green()
        );
    });

    // As long as there is a line to be read from the child process, print it to the
    // terminal.
    let mut child_stdout_reader = BufReader::new(child_stdout).lines();
    while let Some(line) = child_stdout_reader.next_line().await.into_diagnostic()? {
        println!("{}", format!("❯ {}", line).cyan());
    }

    // Wait for the child task to complete.
    child_task_join_handle.await.into_diagnostic()?;

    ok!()
}

/// This is a simpler version of the `main` function above. It doesn't need to use
/// `tokio::spawn` to wait for the child process to complete.
async fn main_simpler() -> miette::Result<()> {
    // Create a child process that runs `cat`.
    // - Send the output of `cat` back to this child process.
    // - This child / command does not make progress until `wait().await` is called.
    let mut child = tokio::process::Command::new("cat")
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .into_diagnostic()?;

    // Get the stdout of the child process. Do this before the next step, because the
    // `child` struct is moved into the closure.
    let Some(child_stdout) = child.stdout.take() else {
        miette::bail!("Failed to capture stdout of child process");
    };

    // As long as there is a line to be read from the child process, print it to the
    // terminal.
    let mut child_stdout_reader = BufReader::new(child_stdout).lines();
    while let Some(line) = child_stdout_reader.next_line().await.into_diagnostic()? {
        println!("{}", format!("❯ {}", line).cyan());
    }

    // Simultaneously waits for the child to exit and collect all remaining output on the
    // stdout/stderr handles, returning an Output instance.
    let output = child.wait_with_output().await.into_diagnostic()?;
    println!(
        "{}",
        format!("Child process exited with status: {:?}", output.status).green()
    );

    ok!()
}

/// The nature of this function is different to the 2 above. For eg, if you run this
/// function in a terminal, you have to terminate the input using `Ctrl-D` (EOF) if you
/// want to see anything displayed in the terminal output. In the two variants above,
/// output is captured in an "interactive" manner, as it comes in from the stdin.
async fn main_non_interactive() -> miette::Result<()> {
    // Create a child process that runs `cat`.
    // - Send the output of `cat` back to this child process.
    // - This child / command does not make progress until `wait().await` is called.
    let child = tokio::process::Command::new("cat")
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .into_diagnostic()?;

    // Simultaneously waits for the child to exit and collect all remaining output on the
    // stdout/stderr handles, returning an Output instance.
    let output = child.wait_with_output().await.into_diagnostic()?;
    println!(
        "{}",
        format!("Child process exited with status: {:?}", output.status).green()
    );

    // Print the output.stdout to terminal.
    println!("{}", String::from_utf8_lossy(&output.stdout));

    ok!()
}
