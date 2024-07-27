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
/// Here's a diagram depicting what happens:
///
/// ```text
/// Terminal emulator running fish shell
/// ┌─────────────────────────────────────────────────────┐
/// │                                                     │
/// │ > echo "foo" | cargo run --bin async_command_exec_2 │
/// │                                                     │
/// └───────▲─────────────▲───────────────────────────────┘
///         │             │        Pipeline above runs
///         │             │        in parallel
///    internal         external
///    shell            process
///    command          command (fork & exec)
///                       │
///                       ├────► create async Command for `cat`
///                       │      with stdout = `Stdio::piped()` to
///                       │      capture the output of `cmd`
///                       │      back into this program
///                       │
///                       ├────► the stdin for this Command is
///                       │      inherited from the current process
///                       │      which is provided by the terminal
///                       │      and `pipe`
///                       │
///                       ├────► `cmd.spawn()` then runs the `cat`
///                       │      process with the given stdin & stdout
///                       │
///                       ├────► instead of waiting normally, we must use
///                       │      tokio::spawn to call `wait()` on the child
///                       │      so it can make progress while we wait for
///                       │      its output below.
///                       │
///                       └────► in our normal task, we can now access `stdout`
///                              WHILE the child task is making progress above.
/// ```
#[tokio::main]
async fn main() -> miette::Result<()> {
    Ok(())
}
