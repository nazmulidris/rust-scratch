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

use crate::{
    byte_io, insert_into_bucket, iterate_bucket, load_or_create_bucket_from_store,
    load_or_create_store,
    protocol::{self, ServerMessage},
    remove_from_bucket, Buffer, CLIArg, ClientMessage, KVBucket, MessageKey, MessageValue,
    CHANNEL_SIZE, CLIENT_ID_TRACING_FIELD,
};
use crossterm::style::Stylize;
use kv::Store;
use miette::{miette, IntoDiagnostic};
use r3bl_rs_utils_core::friendly_random_id;
use tokio::{
    io::{AsyncWrite, BufReader, BufWriter},
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
                    match handle_client_task::event_loop(
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

    // BOOKM: 2) handle_client_message
    #[instrument(skip_all, fields(client_id))]
    pub async fn handle_client_message<Writer: AsyncWrite + Unpin>(
        client_message: ClientMessage<MessageKey, MessageValue>,
        _client_id: &str,
        store: &Store,
        buf_writer: &mut BufWriter<Writer>,
    ) -> miette::Result<()> {
        info!(?client_message, "Received message from client");
        let bucket = load_or_create_bucket_from_store::<MessageKey, MessageValue>(store, None)?;

        match client_message {
            ClientMessage::Exit => {
                info!("Exiting due to client request");
                return Err(miette!("Client requested exit"));
            }
            ClientMessage::GetAll => {
                let payload_buffer = try_get_all_items_from_bucket(&bucket)?;
                byte_io::write(buf_writer, payload_buffer).await?;
            }
            ClientMessage::Insert(key, value) => {
                let payload_buffer = try_insert_into_bucket(&bucket, key, value)?;
                byte_io::write(buf_writer, payload_buffer).await?;
            }
            ClientMessage::Remove(key) => {
                let payload_buffer = try_remove_from_bucket(&bucket, key)?;
                byte_io::write(buf_writer, payload_buffer).await?;
            }
            // TODO: implement the following cases
            ClientMessage::Get(_) => todo!(),
            ClientMessage::Clear => todo!(),
            ClientMessage::Size => todo!(),
            ClientMessage::ContainsKey(_) => todo!(),
            ClientMessage::IsEmpty => todo!(),
            ClientMessage::BroadcastToOthers(_) => todo!(),
        }

        Ok(())
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_get_all_items_from_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
    ) -> miette::Result<Buffer> {
        let mut item_vec: Vec<(MessageKey, MessageValue)> = vec![];
        iterate_bucket(bucket, |key: MessageKey, value: MessageValue| {
            item_vec.push((key, value));
        });
        let server_message = ServerMessage::<MessageKey, MessageValue>::GetAll(item_vec);
        let payload_buffer = bincode::serialize(&server_message).into_diagnostic()?;
        Ok(payload_buffer)
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_insert_into_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
        key: MessageKey,
        value: MessageValue,
    ) -> miette::Result<Buffer> {
        let it = match insert_into_bucket(bucket, key, value) {
            Ok(_) => true,
            Err(error) => {
                error!(%error, "Problem inserting into bucket");
                false
            }
        };
        let server_message = ServerMessage::<MessageKey, MessageValue>::Insert(it);
        let payload_buffer = bincode::serialize(&server_message).into_diagnostic()?;
        Ok(payload_buffer)
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_remove_from_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
        key: MessageKey,
    ) -> miette::Result<Buffer> {
        let it = match remove_from_bucket(bucket, key) {
            Ok(_) => true,
            Err(error) => {
                error!(%error, "Problem removing from bucket");
                false
            }
        };
        let server_message = ServerMessage::<MessageKey, MessageValue>::Remove(it);
        let payload_buffer = bincode::serialize(&server_message).into_diagnostic()?;
        Ok(payload_buffer)
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
    pub async fn event_loop(
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
            ServerMessage::<MessageKey, MessageValue>::SetClientId(client_id.to_string());
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
                    if handle_client_message(client_message, client_id, &store, &mut buf_writer).await.is_err() {
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

                    // Send Exit message to client.
                    let payload_bytes = bincode::serialize(
                        &ServerMessage::<MessageKey, MessageValue>::Exit,
                    ).into_diagnostic()?;
                    byte_io::write(&mut buf_writer, payload_bytes).await?;

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

#[cfg(test)]
pub mod test_fixtures {
    use crate::Buffer;
    use std::io::Result;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::AsyncWrite;

    /// A mock struct for the [tokio::net::TcpStream].
    /// - Alternative to [tokio_test::io::Builder::new()].
    /// - The difference is that [MockTcpStreamWrite] allows access to the expected write
    ///   buffer.
    pub struct MockTcpStreamWrite {
        pub expected_write: Buffer,
    }

    /// Implement the [AsyncWrite] trait for the mock struct.
    impl AsyncWrite for MockTcpStreamWrite {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            self.expected_write.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
            Poll::Ready(Ok(()))
        }
    }
}

#[cfg(test)]
pub mod test_handle_client_task {
    use crate::{
        handle_client_task::{
            handle_client_message, try_get_all_items_from_bucket, try_insert_into_bucket,
            try_remove_from_bucket,
        },
        insert_into_bucket, load_or_create_bucket_from_store, load_or_create_store,
        test_fixtures::MockTcpStreamWrite,
        Buffer, ClientMessage, Data,
    };
    use tempfile::tempdir;
    use tokio::io::BufWriter;

    /// More info: <https://tokio.rs/tokio/topics/testing>
    #[tokio::test]
    async fn test_handle_client_message_get_all() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;
        let bucket = load_or_create_bucket_from_store::<crate::MessageKey, crate::MessageValue>(
            &store, None,
        )?;
        insert_into_bucket(&bucket, "foo".to_string(), Data::default())?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let payload_bytes = try_get_all_items_from_bucket(&bucket)?;

        // Create a mock writer (for the write half of the TcpStream).
        let writer = MockTcpStreamWrite {
            expected_write: Vec::new(),
        };
        let mut buf_writer = BufWriter::new(writer);

        // Prepare the actual payload, with length-prefix from [byte_io::write]. This will
        // be accumulated in the buf_writer.
        handle_client_message(
            ClientMessage::GetAll,
            "test_client_id",
            &store,
            &mut buf_writer,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    // BOOKM: 4) add tests for each handle_client_message variant
    #[tokio::test]
    async fn test_handle_client_message_insert() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;
        let bucket = load_or_create_bucket_from_store::<crate::MessageKey, crate::MessageValue>(
            &store, None,
        )?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let payload_bytes = try_insert_into_bucket(&bucket, "foo".to_string(), Data::default())?;

        // Create a mock writer (for the write half of the TcpStream).
        let writer = MockTcpStreamWrite {
            expected_write: Vec::new(),
        };
        let mut buf_writer = BufWriter::new(writer);

        // Prepare the actual payload, with length-prefix from [byte_io::write]. This will
        // be accumulated in the buf_writer.
        handle_client_message(
            ClientMessage::Insert("foo".to_string(), Data::default()),
            "test_client_id",
            &store,
            &mut buf_writer,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_client_message_remove() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;
        let bucket = load_or_create_bucket_from_store::<crate::MessageKey, crate::MessageValue>(
            &store, None,
        )?;
        insert_into_bucket(&bucket, "foo".to_string(), Data::default())?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let payload_bytes = try_remove_from_bucket(&bucket, "foo".to_string())?;

        // Create a mock writer (for the write half of the TcpStream).
        let writer = MockTcpStreamWrite {
            expected_write: Vec::new(),
        };
        let mut buf_writer = BufWriter::new(writer);

        // Prepare the actual payload, with length-prefix from [byte_io::write]. This will
        // be accumulated in the buf_writer.
        handle_client_message(
            ClientMessage::Remove("foo".to_string()),
            "test_client_id",
            &store,
            &mut buf_writer,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }
}
