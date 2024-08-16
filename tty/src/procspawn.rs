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

//! # Example using `procspawn` to spawn processes
//!
//! The [`procspawn`](https://docs.rs/procspawn/latest/procspawn/) crate provides the
//! ability to spawn processes with a function similar to `thread::spawn`.
//!
//! - Unlike `thread::spawn` data cannot be passed by the use of closures.
//! - Instead if must be explicitly passed as serializable object (specifically it must be
//!   `serde` serializable). Internally, the data is serialized using
//!   [`bincode`](https://docs.rs/procspawn/latest/procspawn/#bincode-limitations).
//! - The return value from the spawned closure also must be serializable and can then be
//!   retrieved from the returned join handle.
//! - If the spawned function causes a panic it will also be serialized across the process
//!   boundaries.
//!
//! # Examples
//!
//! Great [examples](https://github.com/mitsuhiko/procspawn/tree/master/examples) from the
//! official docs.
//!
//! # Run the binary
//!
//! ```text
//! ┌─────────────────────────────┐
//! │ > cargo run --bin procspawn │
//! └─────────────────────────────┘
//! ```

use cli_clipboard::ClipboardProvider;
use std::io::Read;
use std::process::Stdio;

fn main() {
    // A spawned process will execute every line of code up to this point.
    procspawn::init();

    let pid = std::process::id();
    println!("PID (parent): {}", pid);

    // This is passed across process boundaries, using serde.
    let arg: Vec<i64> = vec![1, 2, 3, 4];
    let mut join_handle = get_builder().spawn(arg, run_on_another_process);

    // Read the stdout of the child process into `buf`.
    let mut buf = String::new();
    join_handle
        .stdout()
        .unwrap()
        .read_to_string(&mut buf)
        .unwrap();
    let (sum, pid_child) = join_handle.join().unwrap();

    // Make assertions.
    assert_eq!(buf, "Received data [1, 2, 3, 4]\n");
    assert_eq!(sum, 10);
    println!("Sum: {} from PID (child): {}", sum, pid_child);
}

/// Create a new builder with stdout piped.
fn get_builder() -> procspawn::Builder {
    let mut it = procspawn::Builder::new();
    it.stdout(Stdio::piped());
    it.stderr(Stdio::null());
    it
}

/// This function will be executed in a separate process.
fn run_on_another_process(param: Vec<i64>) -> (i64, u32) {
    println!("Received data {:?}", &param);

    // This won't get printed to the terminal. In fact all the stderr output of the
    // child process will be discarded. This includes the error from `mesa` driver for
    // `wayland` on Linux (via the `cli-clipboard` crate, via the `wl-clipboard-rs`
    // crate).
    let pid = std::process::id();
    eprintln!("PID (child ): {}", pid);

    // Copy child pid to the clipboard.
    let mut ctx = cli_clipboard::ClipboardContext::new().unwrap();
    ctx.set_contents(pid.to_string().to_owned()).unwrap();
    ctx.get_contents().unwrap();

    (param.into_iter().sum::<i64>(), pid)
}
