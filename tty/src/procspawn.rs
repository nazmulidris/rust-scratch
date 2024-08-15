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

use std::io::Read;
use std::process::Stdio;

/// The [`procspawn`](https://docs.rs/procspawn/latest/procspawn/) crate provides the
/// ability to spawn processes with a function similar to `thread::spawn`.
///
/// - Unlike `thread::spawn` data cannot be passed by the use of closures.
/// - Instead if must be explicitly passed as serializable object (specifically it must be
///   `serde` serializable).
/// - The return value from the spawned closure also must be serializable and can then be
///   retrieved from the returned join handle.
/// - If the spawned function causes a panic it will also be serialized across the process
///   boundaries.
///
/// # Examples
///
/// Great [examples](https://github.com/mitsuhiko/procspawn/tree/master/examples) from the
/// official docs.
///
/// # Run the binary
///
/// ```text
/// ┌─────────────────────────────┐
/// │ > cargo run --bin procspawn │
/// └─────────────────────────────┘
/// ```
fn main() {
    // A spawned process will execute every line of code up to this point.
    procspawn::init();

    // Create a new builder with stdout piped.
    let mut builder = {
        let mut it = procspawn::Builder::new();
        it.stdout(Stdio::piped());
        it
    };

    // This is passed across process boundaries, using serde.
    let arg = vec![1, 2, 3, 4];
    let mut join_handle = builder.spawn(arg, |param| {
        println!("Received data {:?}", &param);
        param.into_iter().sum::<i64>()
    });

    // Read the stdout of the child process into `buf`.
    let mut buf = String::new();
    join_handle
        .stdout()
        .unwrap()
        .read_to_string(&mut buf)
        .unwrap();
    let output = join_handle.join().unwrap();

    // Make assertions.
    assert_eq!(buf, "Received data [1, 2, 3, 4]\n");
    assert_eq!(output, 10);
}
