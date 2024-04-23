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
use serde::{Deserialize, Serialize};
use tokio::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tracing::{error, info, instrument};

use crate::{protocol, Buffer, CLIArg, CHANNEL_SIZE, CLIENT_ID_TRACING_FIELD};

/// Just a sample value or payload type. Replace this with whatever type you want to use.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub id: f32,
    pub description: String,
    pub data: Buffer,
}

#[instrument(skip_all)]
pub async fn server_main(cli_args: CLIArg) -> miette::Result<()> {
    let address = cli_args.address;
    let port = cli_args.port;

    // Try to start the server.
    info!("Starting server on {}:{}", address, port);
    let listener = TcpListener::bind(format!("{}:{}", address, port))
        .await
        .into_diagnostic()?;

    // Create broadcast channel for sending messages to all clients.
    let (sender_inter_client_broadcast_channel, _) = broadcast::channel::<Data>(CHANNEL_SIZE);

    // Create broadcast channel to handle shutdown.
    let (sender_shutdown_broadcast_channel, mut receiver_shutdown_broadcast_channel) =
        broadcast::channel::<()>(1);

    // Set up Ctrl-C handler.
    setup_ctrlc_handler(sender_shutdown_broadcast_channel.clone()).await?;

    info!("Listening for new connections");

    // Server infinite loop - accept connections.
    loop {
        tokio::select! {
            // Branch 1: Accept incoming connections ("blocking").
            result /* Result<(TcpStream, SocketAddr), Error> */ = listener.accept() => {
                let (client_tcp_stream, _) = result.into_diagnostic()?;

                // Clone the broadcast channel senders.
                let sender_inter_client_broadcast_channel = sender_inter_client_broadcast_channel.clone();
                let sender_shutdown_broadcast_channel = sender_shutdown_broadcast_channel.clone();

                // Start task to handle a connection.
                tokio::spawn(async move {
                    let client_id = friendly_random_id::generate_friendly_random_id();
                    match handle_client_task::main_loop(
                        &client_id,
                        client_tcp_stream,
                        sender_inter_client_broadcast_channel,
                        sender_shutdown_broadcast_channel,
                    )
                    .await
                    {
                        Err(error) => error!(client_id, %error, "Problem handling client task"),
                        Ok(_) => info!(client_id, "Successfully ended client task"),
                    }
                });

            }

            // Branch 2: Monitor shutdown channel.
            _ = receiver_shutdown_broadcast_channel.recv() => {
                info!("Received shutdown signal");
                break;
            }
        }
    }

    Ok(())
}

/// The `client_id` field is added to the span, so that it can be used in the logs by all
/// the functions in this module. See also: [crate::CLIENT_ID_TRACING_FIELD].
pub mod handle_client_task {
    use std::fmt::Debug;

    use crate::byte_io;

    use super::*;

    #[instrument(skip_all, fields(client_id))]
    pub async fn handle_client_message<K: Debug + Default, V: Debug + Default>(
        client_message: protocol::ClientMessage<K, V>,
        _client_id: &str,
    ) -> miette::Result<()> {
        info!(?client_message, "Received message from client");

        // 00: do something meaningful w/ this payload and probably generate a response
        match client_message {
            protocol::ClientMessage::Exit => {
                info!("Exiting due to client request");
                Err(miette!("Client requested exit"))
            }
            _ => {
                // 00: do something meaningful w/ this payload and probably generate a response
                todo!()
            }
        }
    }

    // 00: do something meaningful w/ this payload and probably generate a response
    #[instrument(skip_all, fields(client_id))]
    pub async fn handle_broadcast_channel_between_clients_payload(payload: Data) {
        info!(
            "Received payload from broadcast channel (for payloads between clients): {:?}",
            payload
        );
    }

    /// This has an infinite loop, so you might want to call it in a spawn block.
    #[instrument(name = "handle_client_task:main_loop", skip_all, fields(client_id))]
    pub async fn main_loop(
        client_id: &str,
        client_tcp_stream: TcpStream,
        sender_inter_client_broadcast_channel: Sender<Data>,
        sender_shutdown_broadcast_channel: Sender<()>,
    ) -> miette::Result<()> {
        tracing::Span::current().record(CLIENT_ID_TRACING_FIELD, client_id);
        info!("Handling client task");

        // Get the receiver for the inter client channel.
        let mut receiver_inter_client_broadcast_channel =
            sender_inter_client_broadcast_channel.subscribe();

        // Get the receiver for the shutdown channel.
        let mut receiver_shutdown_broadcast_channel = sender_shutdown_broadcast_channel.subscribe();

        // Get reader and writer from TCP stream.
        let (read_half, write_half) = client_tcp_stream.into_split();
        let mut buf_reader = BufReader::new(read_half);
        let mut buf_writer = BufWriter::new(write_half);

        // Send the client ID.
        let server_message =
            protocol::ServerMessage::<String, Data>::SetClientId(client_id.to_string());
        let payload_buffer = bincode::serialize(&server_message).into_diagnostic()?;
        byte_io::write(&mut buf_writer, payload_buffer).await?;
        info!(?server_message, "Sent to client");

        info!("Entering infinite loop to handle client messages");

        // Infinite server loop.
        loop {
            tokio::select! {
                // Branch 1: Read from client.
                result = byte_io::read(&mut buf_reader) => {
                    let payload_buffer = result?;
                    let client_message: protocol::ClientMessage<String, Data> =
                        bincode::deserialize(&payload_buffer).into_diagnostic()?;
                    if handle_client_message(client_message, client_id).await.is_err() {
                        break;
                    }
                }

                // Branch 2: Read from broadcast channel.
                result = receiver_inter_client_broadcast_channel.recv() => {
                    match result {
                        Ok(payload) => {
                            handle_broadcast_channel_between_clients_payload(payload).await;
                        }
                        Err(error) => {
                            error!("Problem reading from broadcast channel: {:?}", error);
                        }
                    }
                }

                // Branch 3: Monitor shutdown channel.
                _ = receiver_shutdown_broadcast_channel.recv() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        Ok(())
    }
}

#[instrument(skip_all)]
pub async fn setup_ctrlc_handler(
    sender_shutdown_broadcast_channel: Sender<()>,
) -> miette::Result<()> {
    ctrlc::set_handler(move || {
        info!(
            "{}",
            "Ctrl-C event detected. Gracefully shutting down..."
                .yellow()
                .bold()
        );
        let _ = sender_shutdown_broadcast_channel.send(());
    })
    .into_diagnostic()
}
