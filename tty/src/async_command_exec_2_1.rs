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

use r3bl_rs_utils_core::ok;

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
#[tokio::main]
async fn main() -> miette::Result<()> {
    ok!()
}
