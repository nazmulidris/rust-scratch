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

use futures::stream::StreamExt;
use miette::IntoDiagnostic;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

/// This example demonstrates how to send and receive Linux (POSIX, Unix) signals in a
/// process. It uses the following crates to make this happen:
/// - [signal-hook](https://docs.rs/signal-hook/)
/// - [signal-hook-tokio](https://docs.rs/signal-hook-tokio/latest/signal_hook_tokio/)
///
/// # Signal handler registration limitations (to receive signals)
///
/// POSIX allows signal handlers to be overridden in a process. This is a powerful feature
/// that can be used to implement a wide variety of functionality.
/// - However, there are
///   [limitations](https://docs.rs/signal-hook/latest/signal_hook/#limitations) around
///   overriding signal handlers in a process. For example, POSIX compliant operating
///   systems will not allow you to override the
///   [`SIGKILL`](https://docs.rs/signal-hook/latest/signal_hook/consts/signal/constant.SIGKILL.html)
///   or
///   [`SIGSTOP`](https://docs.rs/signal-hook/latest/signal_hook/consts/signal/constant.SIGSTOP.html)
///   signals.
/// - Here's a full list of
///   [`FORBIDDEN`](https://docs.rs/signal-hook/latest/signal_hook/low_level/fn.register.html#panics)
///   signals that will `panic` the `register` function, if used.
///
/// # Dependencies needed
///
/// The following needs to be added to the `Cargo.toml` file for this to work:
/// ```toml
/// signal-hook = { version = "0.3.17" }
/// signal-hook-tokio = { version = "0.3.1", features = ["futures-v0_3"] }
/// futures = "0.3.30"
/// ```
#[tokio::main]
async fn main() -> miette::Result<()> {
    // Create a broadcast channel to send signals to the signal handler.
    let (sender_shutdown_channel, _) = tokio::sync::broadcast::channel::<()>(1);

    // Register to listen to the following signals.
    let signals_stream: Signals =
        Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT]).into_diagnostic()?;
    let signals_handle = signals_stream.handle();
    let signals_task_join_handle = tokio::spawn(handle_signals_task(
        signals_stream,
        sender_shutdown_channel.clone(),
    ));

    // Execute your main program logic.
    run_main(sender_shutdown_channel.clone()).await;

    // Terminate the signal stream.
    signals_handle.close();
    signals_task_join_handle.await.into_diagnostic()?;

    Ok(())
}

async fn handle_signals_task(
    mut signals_stream: Signals,
    sender_shutdown_channel: tokio::sync::broadcast::Sender<()>,
) {
    while let Some(signal) = signals_stream.next().await {
        match signal {
            SIGHUP | SIGTERM | SIGINT | SIGQUIT => {
                _ = sender_shutdown_channel.send(());
            }
            _ => unreachable!(),
        }
    }
}

async fn run_main(sender_shutdown_channel: tokio::sync::broadcast::Sender<()>) {
    // End program after 5 seconds.
    let sleep_future = tokio::time::sleep(std::time::Duration::from_secs(5));
    tokio::pin!(sleep_future);

    // Tick every 500ms.
    let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(500));

    // Listen for shutdown signal.
    let mut receiver_shutdown_channel = sender_shutdown_channel.subscribe();

    // Spawn a task to send a signal in 1 second.
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        _ = signal_hook::low_level::raise(signal_hook::consts::signal::SIGTERM);
        println!("Sent SIGTERM signal");
    });

    loop {
        tokio::select! {
            _ = &mut sleep_future => {
                println!("Final tick");
                break;
            }
            _ = receiver_shutdown_channel.recv() => {
                println!("Received shutdown signal");
                break;
            }
            _ = tick_interval.tick() => {
                println!("Tick");
            }
        }
    }
}
