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

//! # Example of how to send and receive Linux (POSIX, Unix) signals in a process
//!
//! It uses the following crates to make this happen:
//! - [signal-hook](https://docs.rs/signal-hook/)
//! - [signal-hook-tokio](https://docs.rs/signal-hook-tokio/latest/signal_hook_tokio/)
//!
//! # Signal handler registration limitations (to receive signals)
//!
//! POSIX allows signal handlers to be overridden in a process. This is a powerful feature
//! that can be used to implement a wide variety of functionality.
//! - However, there are
//!   [limitations](https://docs.rs/signal-hook/latest/signal_hook/#limitations) around
//!   overriding signal handlers in a process. For example, POSIX compliant operating
//!   systems will not allow you to override the
//!   [`SIGKILL`](https://docs.rs/signal-hook/latest/signal_hook/consts/signal/constant.SIGKILL.html)
//!   or
//!   [`SIGSTOP`](https://docs.rs/signal-hook/latest/signal_hook/consts/signal/constant.SIGSTOP.html)
//!   signals.
//! - Here's a full list of
//!   [`FORBIDDEN`](https://docs.rs/signal-hook/latest/signal_hook/low_level/fn.register.html#panics)
//!   signals that will `panic` the `register` function, if used.
//!
//! # Dependencies needed
//!
//! The following needs to be added to the `Cargo.toml` file for this to work:
//! ```toml
//! signal-hook = { version = "0.3.17" }
//! signal-hook-tokio = { version = "0.3.1", features = ["futures-v0_3"] }
//! futures = "0.3.30"
//! ```
//!
//! # Run the binary
//!
//! ```text
//! ┌───────────────────────────────────────────┐
//! │ > cargo run --bin send_and_receive_signal │
//! └───────────────────────────────────────────┘
//! ```

use futures::stream::StreamExt as _;
use miette::IntoDiagnostic;
use r3bl_rs_utils_core::ok;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let pid = std::process::id();
    println!("PID: {}", pid);

    // Broadcast channel to shutdown the process.
    let (sender_shutdown_channel, _) = tokio::sync::broadcast::channel::<()>(1);

    // Register signal handlers.
    let signals_stream: Signals =
        Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT]).into_diagnostic()?;
    let signals_handle = signals_stream.handle();
    let join_handle_monitor_signals_task = tokio::spawn(handle_signals_task(
        signals_stream,
        sender_shutdown_channel.clone(),
    ));

    run_main_event_loop(sender_shutdown_channel.clone()).await;

    // Cleanup tasks after shutdown.
    signals_handle.close();
    join_handle_monitor_signals_task.await.into_diagnostic()?;

    ok!()
}

async fn run_main_event_loop(sender_shutdown_channel: tokio::sync::broadcast::Sender<()>) {
    let mut receiver_shutdown_channel = sender_shutdown_channel.subscribe();

    let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(500));

    // Wait for 1 sec & then send SIGTERM signal.
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        _ = signal_hook::low_level::raise(SIGTERM);
        println!("🧨 Sent SIGTERM signal");
    });

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                println!("Tick");
            }
            _ = receiver_shutdown_channel.recv() => {
                println!("Received shutdown signal");
                break;
            }
        }
    }
}

async fn handle_signals_task(
    mut signals_stream: Signals,
    sender_shutdown_channel: tokio::sync::broadcast::Sender<()>,
) {
    while let Some(signal) = signals_stream.next().await {
        match signal {
            SIGHUP | SIGTERM | SIGINT | SIGQUIT => {
                println!("📥 Received signal: {:?}", signal);
                _ = sender_shutdown_channel.send(());
            }
            _ => unreachable!(),
        }
    }
}
