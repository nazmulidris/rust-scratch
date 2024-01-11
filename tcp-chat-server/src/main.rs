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

use log::info;
use r3bl_rs_utils_core::friendly_random_id;
use r3bl_tui::ColorWheel;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};

type IOResult<T> = std::io::Result<T>;

#[derive(Debug, Clone)]
pub struct Message {
    pub sender_id: String,
    pub payload: String,
}

#[tokio::main]
async fn main() -> IOResult<()> {
    let addr = "127.0.0.1:3000";

    // Start logging.
    femme::start();

    // Bind to the addr & listen.
    let listener = TcpListener::bind(addr).await?;

    // Create broadcast channel.
    let (tx, _) = broadcast::channel::<Message>(10);

    // Server loop.
    loop {
        info!("Listening for new connections");

        // Accept incoming connections.
        let (client_tcp_stream, _) = listener.accept().await?;

        // Start task to handle a connection.
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            match handle_client_task(client_tcp_stream, tx_clone).await {
                Ok(_) => info!("Successfully ended client task"),
                Err(error) => info!("Problem handling client task: {:?}", error),
            }
        });

        info!("Released a connection");
    }
}

async fn handle_client_task(mut client_tcp_stream: TcpStream, tx: Sender<Message>) -> IOResult<()> {
    let id = &friendly_random_id::generate_friendly_random_id();
    info!("[{id}] handle_client: start");

    // Get tx and rx ready.
    let mut rx = tx.subscribe();

    // Get reader and writer from tcp stream.
    let (read_half, write_half) = client_tcp_stream.split();
    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);

    // Send id to client.
    let id_msg = format!("my client_id: [{id}]\n");
    let color_id_msg = ColorWheel::lolcat_into_string(&id_msg);
    let _ = writer.write(&color_id_msg.into_bytes()).await?;
    writer.flush().await?;

    // Handle client connection loop.
    loop {
        // Allocate buffer for reading into.
        let mut incoming = String::new();

        tokio::select! {
            // 1. Socket: Read data from client -> process & reply (to client) & broadcast (to channel).
            socket_read_result = reader.read_line(&mut incoming) => {
                let num_bytes_read: usize = socket_read_result?;

                // Check for EOF to break.
                if num_bytes_read == 0 {
                    break;
                }

                // Process incoming -> outgoing.
                let outgoing: String = process(&incoming);

                // Write outgoing to client_tcp_stream.
                let _ = writer.write(outgoing.as_bytes()).await?;
                writer.flush().await?;

                // Broadcast to channel.
                let _ = tx.send(Message{ sender_id: id.to_string(), payload: outgoing });

                info!("[{id}] handle_client: got message from client -> send reply to client & broadcast to channel")
            }
            // 2. Channel: Read from broadcast channel -> send data to the client.
            channel_read_result = rx.recv() => {
                if let Ok(message /* Message */) = channel_read_result {
                    // Ignore messages that are sent by myself.
                    if &message.sender_id != id {
                        let outgoing = format!("From: [{}]: {}", message.sender_id, message.payload);
                        let _ = writer.write(&outgoing.as_bytes()).await?;
                        writer.flush().await?;
                        info!("[{id}] handle_client: got message from channel -> sent to client")
                    }
                }
            }
        }
    }

    info!("[{id}] handle_client: finished");
    return Ok(());
}

fn process(incoming: &str) -> String {
    ColorWheel::lolcat_into_string(incoming.trim()) + "\n"
}
