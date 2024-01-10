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

use std::net::SocketAddr;

use r3bl_rs_utils_core::friendly_random_id;
use r3bl_tui::ColorWheel;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{tcp::WriteHalf, TcpListener, TcpStream},
    sync::broadcast::{self, error::RecvError, Sender},
};

pub type IOResult<T> = std::io::Result<T>;

#[derive(Debug, Clone)]
pub struct MsgType {
    pub socket_addr: SocketAddr,
    pub payload: String,
    pub from_id: String,
}

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
    let addr = "127.0.0.1:3000";

    // Start logging.
    femme::start();

    // Create TCP listener.
    let tcp_listener = TcpListener::bind(addr).await?;
    log::info!("Server is ready to accept connections on {}", addr);

    // Create channel shared among all clients that connect to the server loop.
    let (tx, _) = broadcast::channel::<MsgType>(10);

    // Server loop.
    loop {
        // Accept incoming socket connections.
        let (tcp_stream, socket_addr) = tcp_listener.accept().await?;

        let tx = tx.clone();
        tokio::spawn(async move {
            let result = handle_client_task(tcp_stream, tx, socket_addr).await;
            match result {
                Ok(_) => {
                    log::info!("handle_client_task() terminated gracefully")
                }
                Err(error) => log::error!("handle_client_task() encountered error: {}", error),
            }
        });
    }
}

async fn handle_client_task(
    mut tcp_stream: TcpStream,
    tx: Sender<MsgType>,
    socket_addr: SocketAddr,
) -> IOResult<()> {
    log::info!("Handle socket connection from client");

    let id = friendly_random_id::generate_friendly_random_id();
    let mut rx = tx.subscribe();

    // Set up buf reader and writer.
    let (reader, writer) = tcp_stream.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    // Send welcome message to client w/ ids.
    let welcome_msg_for_client =
        ColorWheel::lolcat_into_string(&format!("addr: {}, id: {}\n", socket_addr, id));
    writer.write(welcome_msg_for_client.as_bytes()).await?;
    writer.flush().await?;

    let mut incoming = String::new();

    loop {
        let tx = tx.clone();
        tokio::select! {
            // Read from broadcast channel.
            result = rx.recv() => {
                read_from_broadcast_channel(result, socket_addr, &mut writer, &id).await?;
            }

            // Read from socket.
            network_read_result = reader.read_line(&mut incoming) => {
                let num_bytes_read: usize = network_read_result?;
                // EOF check.
                if num_bytes_read == 0 {
                    break;
                }
                handle_socket_read(num_bytes_read, &id, &incoming, &mut writer, tx, socket_addr).await?;
                incoming.clear();
            }
        }
    }

    Ok(())
}

async fn read_from_broadcast_channel(
    result: Result<MsgType, RecvError>,
    socket_addr: SocketAddr,
    writer: &mut BufWriter<WriteHalf<'_>>,
    id: &str,
) -> IOResult<()> {
    match result {
        Ok(it) => {
            let msg: MsgType = it;
            log::info!("[{}]: channel: {:?}", id, msg);
            if msg.socket_addr != socket_addr {
                writer.write(msg.payload.as_bytes()).await?;
                writer.flush().await?;
            }
        }
        Err(error) => {
            log::error!("{:?}", error);
        }
    }

    Ok(())
}

async fn handle_socket_read(
    num_bytes_read: usize,
    id: &str,
    incoming: &str,
    writer: &mut BufWriter<WriteHalf<'_>>,
    tx: Sender<MsgType>,
    socket_addr: SocketAddr,
) -> IOResult<()> {
    log::info!(
        "[{}]: incoming: {}, size: {}",
        id,
        incoming.trim(),
        num_bytes_read
    );

    // Process incoming -> outgoing.
    let outgoing = process(&incoming);

    // outgoing -> Writer.
    writer.write(outgoing.as_bytes()).await?;
    writer.flush().await?;

    // Broadcast outgoing to the channel.
    let _ = tx.send(MsgType {
        socket_addr,
        payload: incoming.to_string(),
        from_id: id.to_string(),
    });

    log::info!(
        "[{}]: outgoing: {}, size: {}",
        id,
        outgoing.trim(),
        num_bytes_read
    );

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
