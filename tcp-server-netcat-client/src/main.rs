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

use r3bl_rs_utils_core::friendly_random_id;
use r3bl_tui::ColorWheel;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};

pub type IOResult<T> = std::io::Result<T>;

// Overview:
// Create TcpListener and accept socket connection
// Create a tcp stream
// Get reader / writer from stream
// Loop:
//   - read incoming from reader
//   - process(incoming) => outgoing
//   - broadcast incoming to others connected clients
#[tokio::main]
pub async fn main() -> IOResult<()> {
    femme::start();

    let addr = "127.0.0.1:3000";

    // Create TCP listener.
    let tcp_listener = TcpListener::bind(addr).await?;
    log::info!("Server is ready to accept connections on {}", addr);

    // Accept incoming socket connections.
    loop {
        let (tcp_stream, _socket_addr) = tcp_listener.accept().await?;

        tokio::spawn(async move {
            let result = handle_socket_connection_from_client_task(tcp_stream).await;
            match result {
                Ok(_) => {
                    log::info!("handle_socket_connection_from_client_task() terminated gracefully")
                }
                Err(error) => log::error!(
                    "handle_socket_connection_from_client_task() encountered error: {}",
                    error
                ),
            }
        });
    }
}

async fn handle_socket_connection_from_client_task(mut tcp_stream: TcpStream) -> IOResult<()> {
    log::info!("Handle socket connection from client");
    let id = friendly_random_id::generate_friendly_random_id();

    // Set up buf reader and writer.
    let (reader, writer) = tcp_stream.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let mut incoming = String::new();

    loop {
        // Reader -> incoming.
        let num_bytes_read = reader.read_line(&mut incoming).await?;

        log::info!(
            "[{}]: incoming: {}, size: {}",
            id,
            incoming.trim(),
            num_bytes_read
        );

        // EOF check.
        if num_bytes_read == 0 {
            break;
        }

        // Process incoming -> outgoing.
        let outgoing = process(&incoming);

        // outgoing -> Writer.
        writer.write(outgoing.as_bytes()).await?;
        writer.flush().await?;

        log::info!(
            "[{}]: outgoing: {}, size: {}",
            id,
            outgoing.trim(),
            num_bytes_read
        );

        incoming.clear();
    }

    Ok(())
}

fn process(incoming: &str) -> String {
    // Remove new line from incoming.
    let incoming_trimmed = format!("{}", incoming.trim());
    // Colorize it.
    let outgoing = ColorWheel::lolcat_into_string(&incoming_trimmed);
    // Add new line back to outgoing.
    format!("{}\n", outgoing)
}
