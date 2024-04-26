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

use crate::{byte_io, load_or_create_store, MessageKey};
use crate::{protocol, CLIArg, MessageValue, CHANNEL_SIZE, CLIENT_ID_TRACING_FIELD};
use crossterm::style::Stylize;
use kv::Store;
use miette::{miette, IntoDiagnostic};
use r3bl_rs_utils_core::friendly_random_id;
use tokio::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};

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
    let (sender_inter_client_broadcast_channel, _) =
        broadcast::channel::<MessageValue>(CHANNEL_SIZE);

    // Shutdown cancellation token, to cooperatively & gracefully end all awaiting running
    // tasks. Calling `abort()` isn't reliable. Neither is dropping the task. More info:
    // https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/
    let shutdown_token = CancellationToken::new();

    // Set up Ctrl-C handler.
    setup_ctrlc_handler(shutdown_token.clone()).await?;

    info!("Listening for new connections");

    // Server infinite loop - accept connections.
    loop {
        let shutdown_token_clone = shutdown_token.clone();
        tokio::select! {
            // Branch 1: Accept incoming connections ("blocking").
            result /* Result<(TcpStream, SocketAddr), Error> */ = listener.accept() => {
                let (client_tcp_stream, _) = result.into_diagnostic()?;

                // Clone the broadcast channel senders.
                let sender_inter_client_broadcast_channel = sender_inter_client_broadcast_channel.clone();

                // Start task to handle a connection.
                tokio::spawn(async move {
                    let client_id = friendly_random_id::generate_friendly_random_id();
                    match handle_client_task::enter_loop(
                        &client_id,
                        client_tcp_stream,
                        sender_inter_client_broadcast_channel,
                        shutdown_token_clone.clone(),
                    )
                    .await
                    {
                        Err(error) => error!(client_id, %error, "Problem handling client task"),
                        Ok(_) => info!(client_id, "Successfully ended client task"),
                    }
                });

            }

            // Branch 2: Monitor shutdown cancellation token.
            _ = shutdown_token.cancelled() => {
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
    use super::*;

    #[instrument(skip_all, fields(client_id))]
    pub async fn handle_client_message(
        client_message: protocol::ClientMessage<MessageKey, MessageValue>,
        _client_id: &str,
        _store: &Store,
    ) -> miette::Result<()> {
        info!(?client_message, "Received message from client");

        match client_message {
            protocol::ClientMessage::Exit => {
                info!("Exiting due to client request");
                Err(miette!("Client requested exit"))
            }
            _ => {
                // TODO: do something meaningful w/ this payload & _store
                todo!()
            }
        }
    }

    // TODO: do something meaningful w/ this payload and probably generate a response
    #[instrument(skip_all, fields(client_id))]
    pub async fn handle_broadcast_channel_between_clients_payload(payload: MessageValue) {
        info!(
            "Received payload from broadcast channel (for payloads between clients): {:?}",
            payload
        );
    }

    /// This has an infinite loop, so you might want to call it in a spawn block.
    #[instrument(name = "handle_client_task:main_loop", skip_all, fields(client_id))]
    pub async fn enter_loop(
        client_id: &str,
        client_tcp_stream: TcpStream,
        sender_inter_client_broadcast_channel: Sender<MessageValue>,
        shutdown_token: CancellationToken,
    ) -> miette::Result<()> {
        tracing::Span::current().record(CLIENT_ID_TRACING_FIELD, client_id);
        info!("Handling client task");

        // Get the receiver for the inter client channel.
        let mut receiver_inter_client_broadcast_channel =
            sender_inter_client_broadcast_channel.subscribe();

        // Get reader and writer from TCP stream.
        let (read_half, write_half) = client_tcp_stream.into_split();
        let mut buf_reader = BufReader::new(read_half);
        let mut buf_writer = BufWriter::new(write_half);

        // Send the client ID.
        let server_message =
            protocol::ServerMessage::<MessageKey, MessageValue>::SetClientId(client_id.to_string());
        let payload_buffer = bincode::serialize(&server_message).into_diagnostic()?;
        byte_io::write(&mut buf_writer, payload_buffer).await?;
        info!(?server_message, "Sent to client");

        info!("Entering infinite loop to handle client messages");

        // Infinite server loop.
        loop {
            let store = load_or_create_store(None)?;

            tokio::select! {
                // Branch 1: Read from client.
                result = byte_io::read(&mut buf_reader) => {
                    let payload_buffer = result?;
                    let client_message: protocol::ClientMessage<MessageKey, MessageValue> =
                        bincode::deserialize(&payload_buffer).into_diagnostic()?;
                    if handle_client_message(client_message, client_id, &store).await.is_err() {
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

                // Branch 3: Monitor shutdown cancellation token.
                _ = shutdown_token.cancelled() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        Ok(())
    }
}

#[instrument(skip_all)]
pub async fn setup_ctrlc_handler(shutdown_token: CancellationToken) -> miette::Result<()> {
    ctrlc::set_handler(move || {
        info!(
            "{}",
            "Ctrl-C event detected. Gracefully shutting down..."
                .yellow()
                .bold()
        );
        shutdown_token.cancel();
    })
    .into_diagnostic()
}
