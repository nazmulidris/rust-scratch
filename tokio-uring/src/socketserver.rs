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

//! # tokio-uring socket server
//! This is a simple TCP server that listens on port 8080 and echoes back any data that it
//! receives.
//!
//! - It uses the `tokio-uring` crate to handle the I/O operations asynchronously.
//! - It uses the `tokio_util` crate to handle the `CancellationToken` type. Along with
//!   the `tokio` crate, for `tokio::select!` macro.
//! - It also uses the `miette` crate for error handling and the `ctrlc` crate to handle
//!   Ctrl+C signals.
//!
//! # Usage
//! To run this server, run `cargo run --bin socketserver`.
//!
//! To test this server, you can use the `netcat` command-line tool. Run `netcat localhost 8080`
//! in a separate terminal window, and then type some text. You should see the server echo back
//! the text that you typed.
//!
//! # References
//! - <https://tokio.rs/blog/2021-07-tokio-uring>
//! - <https://docs.rs/tokio-uring/latest/tokio_uring/net/struct.TcpStream.html>
//! - <https://tokio.rs/blog/2021-07-tokio-uring>
use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use r3bl_terminal_async::port_availability;
use std::net::SocketAddr;
use tokio::task::AbortHandle;
use tokio_uring::{
    buf::IoBuf,
    net::{TcpListener, TcpStream},
};
use tokio_util::sync::CancellationToken;

/// Run `netcat localhost:8080` to test this server (once you run this main function).
fn main() -> miette::Result<()> {
    // Register tracing subscriber.
    tracing_subscriber::fmt()
        .without_time()
        .compact()
        .with_target(false)
        .with_line_number(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    let cancellation_token = CancellationToken::new();

    // Can't use #[tokio::main] for `main()`, so we have to use the
    // `tokio::runtime::Builder` API. However, we have to launch this in a separate
    // thread, because we don't want it to collide with the `tokio_uring::start()` call.
    let cancellation_token_clone = cancellation_token.clone();
    std::thread::spawn(move || {
        // If you use `Builder::new_current_thread()`, the runtime will use the single /
        // current thread scheduler. `Builder::new_multi_thread()` will use a thread pool.
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .build()
            .into_diagnostic()
            .unwrap()
            .block_on(async_main(cancellation_token_clone))
    });

    let cancellation_token_clone = cancellation_token.clone();
    ctrlc::set_handler(move || {
        tracing::info!("Received Ctrl+C!");
        cancellation_token_clone.cancel();
    })
    .into_diagnostic()?;

    tokio_uring::start(start_server(cancellation_token))?;

    Ok(())
}

async fn async_main(cancellation_token: CancellationToken) {
    tracing::info!("{}", "async_main - start".to_string().magenta().bold());

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(2_500));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                tracing::info!("{}", "async_main - tick".to_string().magenta().bold());

                // Notice in the output, that these tasks are NOT spawned in the same
                // order repeatedly. They are run in parallel on different threads. And
                // these are scheduled in a non-deterministic order.
                let task_1 = tokio::spawn(async {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    tracing::info!("async_main - tick {} - spawn", "#1".to_string().on_green().black().bold());
                });
                let task_2 = tokio::spawn(async {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    tracing::info!("async_main - tick {} - spawn", "#2".to_string().on_red().black().bold());
                });
                let task_3 = tokio::spawn(async {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    tracing::info!("async_main - tick {} - spawn", "#3".to_string().on_blue().black().bold());
                });
                let _ = tokio::join!(task_1, task_2, task_3);
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("async_main - cancelled");
                break;
            }
        }
    }

    tracing::info!("{}", "async_main - end".to_string().magenta().bold());
}

async fn start_server(cancellation_token: CancellationToken) -> miette::Result<()> {
    let tcp_listener = {
        let addr: SocketAddr = "0.0.0.0:8080".parse().into_diagnostic()?;
        // You can bind to the same address repeatedly, and it won't return an error!
        // Might have to check to see whether the port is open or not before binding to
        // it!
        match port_availability::check(addr).await? {
            port_availability::Status::Free => {
                tracing::info!("Port {} is available", addr.port());
            }
            port_availability::Status::Occupied => {
                tracing::info!("Port {} is NOT available, can't bind to it", addr.port());
                return Err(miette::miette!(
                    "Port {} is NOT available, can't bind to it",
                    addr.port()
                ));
            }
        }
        TcpListener::bind(addr).into_diagnostic()?
    };

    tracing::info!("{}", "server - started".to_string().red().bold());

    let mut abort_handles: Vec<AbortHandle> = vec![];

    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                abort_handles.iter().for_each(|handle| handle.abort());
                break;
            }
            it = tcp_listener.accept() => {
                let (tcp_stream, _addr) = it.into_diagnostic()?;
                let join_handle = tokio_uring::spawn(handle_connection(tcp_stream));
                abort_handles.push(join_handle.abort_handle());
            }
        }
    }

    tracing::info!("{}", "server - stopped".to_string().red().bold());
    Ok(())
}

/// This is an echo server implementation.
async fn handle_connection(stream: TcpStream) -> miette::Result<()> {
    tracing::info!("handle_connection - start");

    let mut total_bytes_read = 0;
    let mut buf = vec![0u8; 10];

    loop {
        // Read from the stream.
        // Read some data, the buffer is passed by ownership and submitted to the kernel.
        // When the operation completes, we get the buffer back.
        let (result_num_bytes_read, return_buf) = stream.read(buf).await;
        buf = return_buf;
        let num_bytes_read = result_num_bytes_read.into_diagnostic()?;

        // Check for EOF.
        if num_bytes_read == 0 {
            break;
        }

        // Write to the stream.
        let (result_num_bytes_written, slice) = stream.write_all(buf.slice(..num_bytes_read)).await;
        result_num_bytes_written.into_diagnostic()?; // Make sure no errors.

        // Update the buffer.
        buf = slice.into_inner();
        total_bytes_read += num_bytes_read;

        tracing::info!(
            "{}: {}",
            "handle_connection - num_bytes_read".to_string().red(),
            num_bytes_read
        );
    }

    tracing::info!(
        "handle_connection - end, total_bytes_read: {}",
        total_bytes_read
    );
    Ok(())
}
