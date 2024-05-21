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
//!
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
//!
//! To run this server, run `cargo run --bin socketserver`.
//!
//! To test this server, you can use the `netcat` command-line tool. Run `netcat localhost 8080`
//! in a separate terminal window, and then type some text. You should see the server echo back
//! the text that you typed.
//!
//! # References
//!
//! - <https://tokio.rs/blog/2021-07-tokio-uring>
//! - <https://docs.rs/tokio-uring/latest/tokio_uring/net/struct.TcpStream.html>
//! - <https://tokio.rs/blog/2021-07-tokio-uring>
use miette::IntoDiagnostic;
use std::net::SocketAddr;
use tokio::task::AbortHandle;
use tokio_uring::{
    buf::IoBuf,
    net::{TcpListener, TcpStream},
};
use tokio_util::sync::CancellationToken;

/// Run `netcat localhost:8080` to test this server (once you run this main function).
fn main() -> miette::Result<()> {
    let cancellation_token = CancellationToken::new();

    // Can't use #[tokio::main] for `main()`, so we have to use the
    // `tokio::runtime::Builder` API. However, we have to launch this in a separate
    // thread, because we don't want it to collide with the `tokio_uring::start()` call.
    let cancellation_token_clone = cancellation_token.clone();
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .into_diagnostic()
            .unwrap()
            .block_on(async_main(cancellation_token_clone))
    });

    let cancellation_token_clone = cancellation_token.clone();
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C!");
        cancellation_token_clone.cancel();
    })
    .into_diagnostic()?;

    tokio_uring::start(start_server(cancellation_token))?;

    Ok(())
}

async fn async_main(cancellation_token: CancellationToken) {
    println!("async_main - start");

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(1_000));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                println!("async_main - tick");
            }
            _ = cancellation_token.cancelled() => {
                println!("async_main - cancelled");
                break;
            }
        }
    }

    println!("async_main - end");
}

async fn start_server(cancellation_token: CancellationToken) -> miette::Result<()> {
    let tcp_listener = {
        let addr: SocketAddr = "0.0.0.0:8080".parse().into_diagnostic()?;
        TcpListener::bind(addr).into_diagnostic()?
    };

    println!("server - started");

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

    println!("server - stopped");
    Ok(())
}

/// This is an echo server implementation.
async fn handle_connection(stream: TcpStream) -> miette::Result<()> {
    println!("handle_connection - start");

    let mut total_bytes_read = 0;
    let mut buf = vec![0u8; 10];

    loop {
        // Read from the stream.
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
    }

    println!(
        "handle_connection - end, total_bytes_read: {}",
        total_bytes_read
    );
    Ok(())
}
