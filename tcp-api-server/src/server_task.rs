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
use miette::miette;
use miette::IntoDiagnostic;
use r3bl_rs_utils_core::friendly_random_id;
use tokio::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tracing::{error, info};

use crate::{print_output, protocol, CLIArg, CHANNEL_SIZE};

pub async fn start_server(cli_args: CLIArg) -> miette::Result<()> {
    let address = cli_args.address;
    let port = cli_args.port;

    // Try to start the server.
    info!("Starting server on {}:{}", address, port);
    let listener = TcpListener::bind(format!("{}:{}", address, port))
        .await
        .into_diagnostic()?;

    // Set up Ctrl-C handler.
    setup_ctrlc_handler().await?;

    // Create broadcast channel for sending messages to all clients.
    let (sender_for_broadcast_channel_between_client_handlers, _) =
        broadcast::channel::<protocol::MyPayload>(CHANNEL_SIZE);

    // Server infinite loop - accept connections.
    loop {
        info!("Listening for new connections");

        // Accept incoming connections (blocking).
        let (client_tcp_stream, _) = listener.accept().await.into_diagnostic()?;

        // Start task to handle a connection.
        let sender_for_broadcast_channel_between_client_handlers =
            sender_for_broadcast_channel_between_client_handlers.clone();
        tokio::spawn(async move {
            let client_id = friendly_random_id::generate_friendly_random_id();
            info!(
                "[{}]: {}",
                client_id.to_string().yellow().bold(),
                "Handling client task"
            );
            match handle_client_task::enter(
                client_tcp_stream,
                sender_for_broadcast_channel_between_client_handlers,
                &client_id,
            )
            .await
            {
                Ok(_) => info!(
                    "[{}]: {}",
                    client_id.to_string().yellow().bold(),
                    "Successfully ended client task"
                ),
                Err(error) => info!(
                    "[{}]: {} {}",
                    client_id.to_string().yellow().bold(),
                    "Problem handling client task, it ended due to",
                    error.to_string().red().bold()
                ),
            }
        });

        info!("Spawned task to handle the connection");
    }
}

pub mod handle_client_task {
    use crate::MyPayload;

    use super::*;

    pub async fn handle_client_message(
        client_message: protocol::ClientMessage,
        client_id: &String,
    ) -> miette::Result<()> {
        info!(
            "[{}]: Received message from client {}",
            client_id.to_string().yellow().bold(),
            format!("{:?}", client_message).green().bold()
        );

        // 00: do something meaningful w/ this payload and probably generate a response
        match client_message {
            protocol::ClientMessage::Exit => {
                info!(
                    "[{}]: Exiting due to client request",
                    client_id.to_string().yellow().bold(),
                );
                Err(miette!("Client requested exit"))
            }
            _ => {
                // 00: do something meaningful w/ this payload and probably generate a response
                todo!()
            }
        }
    }

    // 00: do something meaningful w/ this payload and probably generate a response
    pub async fn handle_broadcast_channel_between_clients_payload(payload: MyPayload) {
        info!(
            "Received payload from broadcast channel (for payloads between clients): {:?}",
            payload
        );
    }

    /// This has an infinite loop, so you might want to call it in a spawn block.
    pub async fn enter(
        client_tcp_stream: TcpStream,
        sender_for_broadcast_channel_between_client_handlers: Sender<protocol::MyPayload>,
        client_id: &String,
    ) -> miette::Result<()> {
        // Get sender and receiver ready.
        let mut receiver_for_broadcast_channel_between_client_handlers =
            sender_for_broadcast_channel_between_client_handlers.subscribe();

        // Get reader and writer from TCP stream.
        let (read_half, write_half) = client_tcp_stream.into_split();
        let mut buf_reader = BufReader::new(read_half);
        let mut buf_writer = BufWriter::new(write_half);

        // Send the client ID.
        let set_client_id_message = protocol::ServerMessage::SetClientId(client_id.to_string());
        let payload_buffer = bincode::serialize(&set_client_id_message).into_diagnostic()?;
        protocol::write_bytes(&mut buf_writer, payload_buffer).await?;
        info!(
            "[{}]: Sent 'SetClientId' message to client with id: {}",
            client_id.to_string().yellow().bold(),
            format!("{:?}", set_client_id_message).yellow().bold()
        );

        // Infinite server loop.
        loop {
            tokio::select! {
                // Branch 1: Read from client.
                result = protocol::read_bytes(&mut buf_reader) => {
                    let payload_buffer = result?;
                    let client_message: protocol::ClientMessage = bincode::deserialize(&payload_buffer).into_diagnostic()?;
                    if handle_client_message(client_message, client_id).await.is_err() {
                        break;
                    }
                }

                // Branch 2: Read from broadcast channel.
                result = receiver_for_broadcast_channel_between_client_handlers.recv() => {
                    match result {
                        Ok(payload) => {
                            handle_broadcast_channel_between_clients_payload(payload).await;
                        }
                        Err(error) => {
                            error!("Problem reading from broadcast channel: {:?}", error);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub async fn setup_ctrlc_handler() -> miette::Result<()> {
    ctrlc::set_handler(move || {
        print_output(format!(
            "{}",
            "Ctrl-C event detected. Gracefully shutting down..."
                .yellow()
                .bold()
        ));
        std::process::exit(0);
    })
    .into_diagnostic()
}
