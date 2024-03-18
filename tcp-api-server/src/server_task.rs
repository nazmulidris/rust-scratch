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

use crate::MyPayload;
use crossterm::style::Stylize;
use miette::miette;
use miette::IntoDiagnostic;
use r3bl_rs_utils_core::friendly_random_id;
use tokio::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tracing::{error, info, instrument};

use crate::{print_output, protocol, CLIArg, CHANNEL_SIZE, CLIENT_ID_TRACING_FIELD};

#[instrument(skip_all)]
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

    info!("Listening for new connections");

    // Server infinite loop - accept connections.
    loop {
        // Accept incoming connections (blocking).
        let (client_tcp_stream, _) = listener.accept().await.into_diagnostic()?;

        // Start task to handle a connection.
        let sender_for_broadcast_channel_between_client_handlers =
            sender_for_broadcast_channel_between_client_handlers.clone();

        tokio::spawn(async move {
            let client_id = friendly_random_id::generate_friendly_random_id();
            match handle_client_task::enter(
                client_tcp_stream,
                sender_for_broadcast_channel_between_client_handlers,
                &client_id,
            )
            .await
            {
                Err(error) => error!(client_id, %error, "Problem handling client task"),
                Ok(_) => info!(client_id, "Successfully ended client task"),
            }
        });
    }
}

/// The `client_id` field is added to the span, so that it can be used in the logs by all
/// the functions in this module. See also: [crate::CLIENT_ID_TRACING_FIELD].
pub mod handle_client_task {
    use super::*;

    #[instrument(skip_all, fields(client_id))]
    pub async fn handle_client_message(
        client_message: protocol::ClientMessage,
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
    pub async fn handle_broadcast_channel_between_clients_payload(payload: MyPayload) {
        info!(
            "Received payload from broadcast channel (for payloads between clients): {:?}",
            payload
        );
    }

    /// This has an infinite loop, so you might want to call it in a spawn block.
    #[instrument(name = "handle_client_task:enter", skip_all, fields(client_id))]
    pub async fn enter(
        client_tcp_stream: TcpStream,
        sender_for_broadcast_channel_between_client_handlers: Sender<protocol::MyPayload>,
        client_id: &str,
    ) -> miette::Result<()> {
        tracing::Span::current().record(CLIENT_ID_TRACING_FIELD, client_id);
        info!("Handling client task");

        // Get sender and receiver ready.
        let mut receiver_for_broadcast_channel_between_client_handlers =
            sender_for_broadcast_channel_between_client_handlers.subscribe();

        // Get reader and writer from TCP stream.
        let (read_half, write_half) = client_tcp_stream.into_split();
        let mut buf_reader = BufReader::new(read_half);
        let mut buf_writer = BufWriter::new(write_half);

        // Send the client ID.
        let server_message = protocol::ServerMessage::SetClientId(client_id.to_string());
        let payload_buffer = bincode::serialize(&server_message).into_diagnostic()?;
        protocol::write_bytes(&mut buf_writer, payload_buffer).await?;
        info!(?server_message, "Sent message to client");

        info!("Entering infinite loop to handle client messages");

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

#[instrument]
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
