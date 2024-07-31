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
use tokio::io::{AsyncBufReadExt, BufReader};

/// This example uses the
/// [`tokio::process::Command`](https://docs.rs/tokio/latest/tokio/process/index.html)
/// struct to execute a command asynchronously, and then pipes the output of this command,
/// back to itself. Then prints the output one line at a time.
///
/// - To run this program, pipe some input (from the shell) into this program.
///   ```fish
///   echo -e "hello world\nfoo\nbar\n" | cargo run --bin async_command_exec_2
///   ```
/// - This process will then run `cat` and capture the output from `cat`.
/// - It will then print the output from `cat` one line at time to the terminal.
///
/// # Flow of what happens
///
/// ```text
/// Terminal emulator running fish shell
/// ┌──────────────────────────────────────────────────────────────────┐
/// │ > echo -e "foo\nbar\nbaz" | cargo run --bin async_command_exec_2 │
/// └───────▲─────────────────────▲────────────────────────────────────┘
///         │                     │        Pipeline above runs
///         │                     │        in parallel
///    external                 external
///    process                  process
///    command (fork & exec)    command (fork & exec)
///                               │
///                               ├────► create async Command for `cat`
///                               │      with stdout = `Stdio::piped()` to
///                               │      capture the output of `cmd`
///                               │      back into this program
///                               │
///                               ├────► the stdin for this Command is
///                               │      inherited from the current process
///                               │      which is provided by the terminal
///                               │      and `pipe`
///                               │
///                               ├────► `cmd.spawn()` then sets up the `cat`
///                               │      process to run with the given stdin
///                               │      & stdout and returns a `Child` struct
///                               │
///                               ├────► 🚀 instead of waiting "normally", we
///                               │      must use `tokio::spawn` to call
///                               │      `child.wait().await` on the child so
///                               │      it can make progress while we wait for
///                               │      its output below (in the current task)
///                               │
///                               └────► in our current task, we can now access
///                                      `stdout` WHILE the child task is
///                                      making progress above
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
    // - Send the output of `cat` back to this child process.
    // - This child / command does not make progress until `wait().await` is called.
    let mut child = tokio::process::Command::new("cat")
        .stdout(Stdio::piped())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .into_diagnostic()?;

    // Get the stdout of the child process. Do this before the next step, because the
    // `child` struct is moved into the closure.
    let Some(stdout) = child.stdout.take() else {
        return Err(miette::miette!("Child process did not have a stdout"));
    };

    // 🚀 Ensure the child process is spawned in the runtime, so it can make progress on its
    // own while we await any output.
    let child_task_join_handle = tokio::spawn(async move {
        let status = child.wait().await;
        println!(
            "{}",
            format!("Child process exited with status: {:?}", status).red()
        );
    });

    // As long as there is a line to be read from the child process, print it to the
    // terminal.
    let mut stdout_reader = BufReader::new(stdout).lines();
    while let Some(line) = stdout_reader.next_line().await.into_diagnostic()? {
        println!("{}", format!("❯ {}", line).blue());
    }

    // Wait for the child task to complete.
    child_task_join_handle.await.into_diagnostic()?;

    ok!()
}
