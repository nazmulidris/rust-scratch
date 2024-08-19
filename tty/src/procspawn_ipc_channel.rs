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

//! [ipc_channel::ipc::channel] is used to send messages across processes via IPC. These
//! messages must be serializable.
//!
//! 1. The parent process sends messages to the child process. This happens over an
//!    ipc_channel sender.
//! 2. The child process receives messages from the parent process. This happens over an
//!    ipc_channel receiver. The receiver is passed across process boundaries from the
//!    parent to the child process.
//!
//! # Run the binary
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │ > cargo run --bin procspawn_ipc_channel │
//! └─────────────────────────────────────────┘
//! ```

use miette::IntoDiagnostic;
use r3bl_rs_utils_core::ok;

type Message = String;

const MSG_1: &str = "Hello";
const MSG_2: &str = "World";
const END_MSG: &str = "END";
const SHUTDOWN_MSG: &str = "SHUTDOWN";

fn main() -> miette::Result<()> {
    // A spawned process will execute every line of code up to this point.
    procspawn::init();

    // Create a channel to send messages across processes.
    let (sender, receiver) = ipc_channel::ipc::channel::<Message>().into_diagnostic()?;

    // Spawn a child process that will receive messages from the parent process.
    let mut join_handle = configure_builder().spawn(
        /* arg from parent process */ receiver,
        /* param to child process; closure runs in child process */
        run_in_child_process,
    );

    parent_send_messages(sender)?;

    // Read the stdout, until EOF, of the child process into `buf`.
    let mut buf = String::new();
    use std::io::Read as _; // Import `Read` trait for `read_to_string`.
    let Some(stdout) = join_handle.stdout() else {
        miette::bail!("Failed to get stdout");
    };
    let bytes_read = stdout.read_to_string(&mut buf).into_diagnostic()?;
    println!(
        "Output from child process: {:?}, bytes_read: {}",
        buf, bytes_read
    );

    // Make assertions.
    assert_eq!(buf, format!("{MSG_1}\n{MSG_2}\n{END_MSG}\n"));

    // Wait for the child process to exit and get its return value.
    join_handle.join().into_diagnostic()?;

    ok!()
}

fn parent_send_messages(sender: ipc_channel::ipc::IpcSender<Message>) -> miette::Result<()> {
    sender.send(MSG_1.to_string()).into_diagnostic()?;
    sender.send(MSG_2.to_string()).into_diagnostic()?;
    sender.send(SHUTDOWN_MSG.to_string()).into_diagnostic()?;
    ok!()
}

/// This function will be executed in the child process. It gets [Message]s from the
/// parent process and processes them.
fn run_in_child_process(receiver: ipc_channel::ipc::IpcReceiver<Message>) {
    while let Ok(msg) = receiver.recv() {
        if msg == SHUTDOWN_MSG {
            break;
        }
        // Print the message to stdout.
        println!("{}", msg);
    }

    // Print `END_MSG` to stdout.
    println!("{END_MSG}");
}

/// Create a new builder with stdout piped and stderr muted.
fn configure_builder() -> procspawn::Builder {
    let mut it = procspawn::Builder::new();
    it.stdout(std::process::Stdio::piped());
    it.stderr(std::process::Stdio::null());
    it
}
