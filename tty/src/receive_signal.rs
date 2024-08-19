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

//! # Example of how to receive Linux (POSIX, Unix) signals in a process using Tokio
//!
//! - [tokio::signal::ctrl_c] is a utility function that creates a future that completes
//!   when `ctrl-c` is pressed. There is **NO** need to write a signal stream for this
//!   like so:
//!   ```rust
//!   let signal = tokio::signal::unix::SignalKind::interrupt();
//!   let mut stream = tokio::signal::unix::signal(signal)
//!       .into_diagnostic()?;
//!   loop {
//!       tokio::select! {
//!           _ = stream.recv() => {
//!               println!("\nSIGINT received");
//!               break;
//!           }
//!       }
//!   }
//!   ```
//!
//! - [tokio::signal::unix::signal] is a lower level function that you can use to create a
//!   stream of signals of a given type (e.g., [tokio::signal::unix::SignalKind]). Some
//!   examples are:
//!   - [tokio::signal::unix::SignalKind::hangup]
//!   - [tokio::signal::unix::SignalKind::interrupt]
//!   - [tokio::signal::unix::SignalKind::pipe]
//!
//!  - There are limitations to what [tokio::signal::unix::SignalKind::from_raw] can do.
//!    - For example you can't just pass in `SIGSTOP` ie `19` and expect it to work. This
//!      is an [OS
//!      limitation](https://docs.rs/signal-hook/latest/signal_hook/#limitations) for both
//!      `SIGKILL` or `SIGSTOP`.
//!    - Here's a list of POSIX signals that are
//!      [`FORBIDDEN`](https://docs.rs/signal-hook/latest/signal_hook/low_level/fn.register.html#panics)
//!      from the `signal_hook` crate.
//!    - You can just pass the signal number directly to
//!      [tokio::signal::unix::SignalKind::from_raw].
//!    - However, if you're doing more sophisticated things you might need to use the
//!      [signal-hook](https://github.com/vorner/signal-hook) crate (which not only
//!      supports sending and receiving signals, but also has async adapters for `tokio`).
//!
//! Here are relevant docs:
//! - [tokio::signal](https://docs.rs/tokio/latest/tokio/signal/index.html)
//! - [tokio::signal::unix::signal](https://docs.rs/tokio/latest/tokio/signal/unix/fn.signal.html)
//! - [tokio::signal::unix::SignalKind](https://docs.rs/tokio/latest/tokio/signal/unix/struct.SignalKind.html)
//!
//! # Run the binary
//!
//! ```text
//! ┌───────────────────────────────────────┐
//! │ > cargo run --bin send_receive_signal │
//! └───────────────────────────────────────┘
//! ```
//!
//! # Sending signals to the process
//!
//! To get a list of all the signals that you can send to a process, you can run the
//! following command:
//!
//! ```shell
//! kill -L
//! ```
//!
//! To send Ctrl+C, aka, `SIGINT`, aka [tokio::signal::unix::SignalKind::interrupt]) to
//! the process, you can run the following command:
//!
//! ```shell
//! kill -2 <PID>
//! kill -INT <PID>
//! ```
//!
//! To send `SIGWINCH`, aka [tokio::signal::unix::SignalKind::window_change] to the
//! process, simply change the terminal window size of the terminal that the process is
//! running in. Or run the following command:
//!
//! ```shell
//! kill -28 <PID>
//! kill -WINCH <PID>
//! ```

use miette::IntoDiagnostic;
use r3bl_rs_utils_core::ok;
use tokio::signal::unix;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let signal = unix::SignalKind::window_change();
    let mut stream = unix::signal(signal).into_diagnostic()?;

    let mut tick_interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

    let sleep_future = tokio::time::sleep(tokio::time::Duration::from_secs(5));
    tokio::pin!(sleep_future);

    let pid = std::process::id();
    println!("PID: {}", pid);

    // Copy child PID to clipboard.
    use cli_clipboard::ClipboardProvider as _; // Import `ClipboardProvider` trait.
    let mut ctx = cli_clipboard::ClipboardContext::new()
        .map_err(|e| miette::miette!("could not create clipboard context: {}", e))?;
    ctx.set_contents(pid.to_string().to_owned())
        .map_err(|e| miette::miette!("could not set clipboard contents: {}", e))?;
    ctx.get_contents()
        .map_err(|e| miette::miette!("could not get clipboard contents: {}", e))?;

    loop {
        tokio::select! {
            // Respond to window change signal.
            _ = stream.recv() => {
                println!("\nSIGWINCH received");
                break;
            }

            // Sleep for 5 seconds & terminate the program if running.
            _ = &mut sleep_future => {
                println!("\nSlept for 5 seconds");
                break;
            }

            // Run at each tick interval.
            _ = tick_interval.tick() => {
                println!("Tick");
            }

            // Respond to ctrl-c signal.
            _ = tokio::signal::ctrl_c() => {
                println!("\nCtrl-C received");
                break;
            }
        }
    }

    ok!()
}
