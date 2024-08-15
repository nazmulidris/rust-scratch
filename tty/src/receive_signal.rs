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

use crossterm::style::Stylize;
use miette::IntoDiagnostic;

/// This example demonstrates how to receive Linux (POSIX, Unix) signals in a process
/// using Tokio.
///
/// - [tokio::signal::ctrl_c] is a utility function that creates a future that completes
///   when `ctrl-c` is pressed. There is no need to write a signal stream for this like
///   so:
///   ```rust
///   let mut stream_sigterm =
///       tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
///       .into_diagnostic()?;
///   loop {
///       tokio::select! {
///           _ = stream_sigterm.recv() => {
///               println!("\nSIGTERM received");
///               break;
///           }
///       }
///   }
///   ```
///
/// - [tokio::signal::unix::signal] is a lower level function that you can use to create a
///   stream of signals of a given type (e.g., [tokio::signal::unix::SignalKind]). Some
///   examples are:
///   - [tokio::signal::unix::SignalKind::hangup]
///   - [tokio::signal::unix::SignalKind::interrupt]
///   - [tokio::signal::unix::SignalKind::pipe]
///
///  - There are limitations to what [tokio::signal::unix::SignalKind::from_raw] can do.
///    - For example you can't just pass in `SIGSTOP` ie `19` and expect it to work. This
///      is an [OS
///      limitation](https://docs.rs/signal-hook/latest/signal_hook/#limitations) for both
///      `SIGKILL` or `SIGSTOP`.
///    - Here's a list of POSIX signals that are
///      [`FORBIDDEN`](https://docs.rs/signal-hook/latest/signal_hook/low_level/fn.register.html#panics)
///      from the `signal_hook` crate.
///    - You can just pass the signal number directly to
///      [tokio::signal::unix::SignalKind::from_raw].
///    - However, if you're doing more sophisticated things you might need to use the
///      [signal-hook](https://github.com/vorner/signal-hook) crate (which not only
///      supports sending and receiving signals, but also has async adapters for `tokio`).
///
/// Here are relevant docs:
/// - [tokio::signal](https://docs.rs/tokio/latest/tokio/signal/index.html)
/// - [tokio::signal::unix::signal](https://docs.rs/tokio/latest/tokio/signal/unix/fn.signal.html)
/// - [tokio::signal::unix::SignalKind](https://docs.rs/tokio/latest/tokio/signal/unix/struct.SignalKind.html)
///
/// # Run the binary
/// 
/// ```text
/// ┌───────────────────────────────────────┐
/// │ > cargo run --bin send_receive_signal │
/// └───────────────────────────────────────┘
/// ```
#[tokio::main]
async fn main() -> miette::Result<()> {
    println!("{}",
        "Press Ctrl-C to exit the program, and resize the terminal to send SIGWINCH and end the program".blue().bold()
    );

    // Tick every 500ms.
    let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(500));

    // Sleep for 5 seconds (ends program).
    let sleep_future = tokio::time::sleep(std::time::Duration::from_secs(5));
    tokio::pin!(sleep_future);

    // Infinite stream of `SIGWINCH` signals (when terminal size changes).
    let mut stream_sigwinch =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::window_change())
            .into_diagnostic()?;

    loop {
        tokio::select! {
            // Wait for Ctrl-C to be pressed. This is equivalent to `SIGTERM`.
            _ = tokio::signal::ctrl_c() => {
                println!("\nCtrl-C pressed");
                break;
            }
            // This branch will run when the terminal is resized.
            _ = stream_sigwinch.recv() => {
                println!("\nSIGWINCH received");
                break;
            }
            // This branch will run every 500ms.
            _ = tick_interval.tick() => {
                println!("Tick");
            }
            // This branch will end the program after 5 seconds.
            _ = &mut sleep_future => {
                println!("Final tick");
                break;
            }
        }
    }

    Ok(())
}
