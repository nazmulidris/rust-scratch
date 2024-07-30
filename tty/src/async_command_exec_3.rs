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
use r3bl_rs_utils_core::ok;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout},
    task::JoinHandle,
};

/// This example is similar to `async_command_exec_2.rs`, but it both:
/// 1. Provides data to the `cat` command via `stdin`.
/// 2. Captures the output of `cat` via `stdout`. So there is no need to pipe input from
/// the shell into this program.
///
/// You can just run `cargo run --bin async_command_exec_2_1` and it will print the
/// output.
///
/// Look at code example of [tokio::process::Child::wait] to see how to write / send data
/// to the `stdin` of a command.
///
/// Here's a diagram depicting what happens:
///
/// ```text
/// Terminal emulator running fish shell
/// ┌──────────────────────────────────────────┐
/// │ > cargo run --bin async_command_exec_2_1 │
/// └───▲──────────────────────────────────────┘
///     ├────► create async Command for `cat`
///     │      with stdout = `Stdio::piped()` to
///     │      capture the output of `cmd`
///     │      back into this program
///     │
///     ├────► set stdin = `Stdio::piped()` to provide
///     │      input to the `cat` command asynchronously
///     │
///     ├────► `cmd.spawn()` then sets up the `cat` process
///     │      to run with the given stdin & stdout and
///     │      returns a `Child` struct
///     │
///     ├────► 🚀 instead of waiting "normally", we must use
///     │      `tokio::spawn` to call `child.wait().await`
///     │      on the child so it can make progress while
///     │      we wait for its output below (in the current task)
///     │
///     ├────► 🚀 also use `tokio::spawn` to call `child.stdin.write_all()`
///     │      to provide input to the `cat` command
///     │
///     └────► in our current task, we can now access `stdout`
///            WHILE the child task is making progress above
/// ```
///
/// # Kill child processes
///
/// Note that similar to the behavior to the standard library, and unlike the futures
/// paradigm of dropping-implies-cancellation, a spawned process will, by default,
/// continue to execute even after the Child handle has been dropped.
/// [Docs](https://docs.rs/tokio/latest/tokio/process/index.html#caveats). To change this
/// behavior you can use [tokio::process::Command::kill_on_drop] which isn't really
/// recommended.
///
/// Instead, to kill a child process, you can do the following:
/// - [tokio::process::Child::kill] - This forces the child process to exit.
/// - [tokio::process::Child::wait] - This waits for the child process to cleanly exit.
#[tokio::main]
async fn main() -> miette::Result<()> {
    // Create a child process that runs `cat`.
    // 1. Send the output of `cat` back to this child process.
    // 2. Send the input to `cat` from this child process.
    // 3. This child / command does not make progress until `wait().await` is called.
    let mut child = tokio::process::Command::new("cat")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .into_diagnostic()?;

    // These are the bytes that will be sent to the `stdin` of the child process.
    let input = &["hello", "nadia!"];

    // Get the stdout & stdin of the child process. Do this before the next step, because
    // the `child` struct is moved into the closure.
    let (stdout, stdin): (ChildStdout, ChildStdin) = {
        let Some(stdout) = child.stdout.take() else {
            return Err(miette::miette!("Child process did not have a stdout"));
        };
        let Some(stdin) = child.stdin.take() else {
            return Err(miette::miette!("Child process did not have a stdin"));
        };
        (stdout, stdin)
    };

    let child_task_join_handle = spawn_child_process(child);

    let provide_input_task_join_handle = spawn_provide_input(stdin, input).await?;

    // As long as there is a line to be read from the child process, print it to the
    // terminal.
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut output = String::new();
    while let Some(line) = stdout_reader.next_line().await.into_diagnostic()? {
        output.push_str(&line);
        println!("{}", format!("child stdout ❯ {}", line).blue());
    }

    // Wait for the child task to complete.
    child_task_join_handle.await.into_diagnostic()?;
    provide_input_task_join_handle.await.into_diagnostic()?;

    // Make assertions.
    assert_eq!(output, input.join(""));

    ok!()
}

/// 🚀 Provide input to the child process.
async fn spawn_provide_input(
    mut stdin: ChildStdin,
    input: &[&str],
) -> miette::Result<JoinHandle<()>> {
    // Convert the input to a vector of strings, since this will be moved into the
    // `tokio::spawn` block next.
    let input = input.iter().map(|s| s.to_string()).collect::<Vec<String>>();

    let handle = tokio::spawn(async move {
        _ = stdin.write_all(input.join("\n").as_bytes()).await;

        // Dropping the handle signals EOF to the child process.
        drop(stdin);

        println!(
            "{}: \"{}\"",
            "Finished providing input + EOF to child process ❯ stdin".green(),
            input.join("\\n")
        );
    });
    Ok(handle)
}

/// 🚀 Ensure the child process is spawned in the runtime, so it can make progress on its
/// own while we await any output.
fn spawn_child_process(mut child: Child) -> JoinHandle<()> {
    tokio::spawn(async move {
        let status = child.wait().await;
        println!(
            "{}",
            format!("Child process exited with status: {:?}", status).red()
        );
    })
}
