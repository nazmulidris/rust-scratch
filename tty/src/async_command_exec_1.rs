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

use std::process::Stdio;

use crossterm::style::Stylize;
use miette::IntoDiagnostic;

/// This example uses the
/// [`tokio::process::Command`](https://docs.rs/tokio/latest/tokio/process/index.html)
/// struct to execute a command asynchronously.
///
/// It illustrates the following scenarios:
/// 1. Run a command and wait for it to complete. Do not capture the output or provide the
///    input.
/// 2. Run a command and capture the output. Do not provide the input.
///
/// In both cases, the pattern is the same:
/// 1. Create a [tokio::process::Command].
/// 2. Configure it with the desired `stdin` and `stdout`.
/// 3. Spawn the command. Note this doesn't make any progress until you call
///    `wait().await` or `wait_with_output().await`.
/// 4. Wait for the command to complete with or without output capture.
///
/// # Run the binary
/// 
/// ```text
/// ┌────────────────────────────────────────┐
/// │ > cargo run --bin async_command_exec_1 │
/// └────────────────────────────────────────┘
/// ```

#[tokio::main]
async fn main() -> miette::Result<()> {
    run_command_no_capture().await?;
    run_command_capture_output().await?;
    Ok(())
}

macro_rules! command {
    ($cmd:expr, $args:expr) => {
        tokio::process::Command::new($cmd).args($args)
    };
}

// - Run `echo hello world` and wait for it to complete.
// - Do not capture the output or provide the input.
async fn run_command_no_capture() -> miette::Result<()> {
    println!("{}", "run_command_no_capture".blue());

    // Without redirection, the output of the command will be inherited from the process
    // that starts the command. So if this is running in a terminal, the output will be
    // printed to the terminal.
    //
    // Even though `spawn()` is called this child / command doesn't make any progress
    // until you call `wait().await`.
    let mut child = command!("echo", &["hello", "world"])
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .into_diagnostic()?;

    // Wait for the command to complete. Don't capture the output, it will go to `stdout`
    // of the process running this program.
    let exit_status = child.wait().await.into_diagnostic()?;
    assert!(exit_status.success());

    // Print the exit status of the command.
    println!("exit status: {}", exit_status);

    Ok(())
}

// - Run `echo hello world` and wait for it to complete.
// - Capture its output and do not provide the input.
async fn run_command_capture_output() -> miette::Result<()> {
    println!("{}", "run_command_capture_output".blue());

    // Redirect the output of the command to a pipe `Stdio::piped()`.
    //
    // Even though `spawn()` is called this child / command doesn't make any progress
    // until you call `wait_with_out().await`.
    let child = command!("echo", &["hello", "world"])
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .into_diagnostic()?;

    // Wait for the command to complete and capture the output.
    // - Calling `wait()` consumes the child process, so we can't call `output.stdout` on
    //   it after this.
    // - That's why we use `wait_with_output()`, which actually returns a different type
    //   than `wait()`; this is also a great use of type state pattern.
    let output = child.wait_with_output().await.into_diagnostic()?;

    assert!(output.status.success());
    assert_eq!(output.stdout, b"hello world\n");

    Ok(())
}
