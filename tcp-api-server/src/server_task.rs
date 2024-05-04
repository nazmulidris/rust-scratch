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
    byte_io, get_from_bucket, insert_into_bucket, iterate_bucket, load_or_create_bucket_from_store,
    load_or_create_store, protocol::ServerMessage, remove_from_bucket, CLIArg, ClientMessage,
    KVBucket, MessageKey, MessageValue, MyServerMessage, CHANNEL_SIZE, CLIENT_ID_TRACING_FIELD,
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
use tracing::{debug, error, info, instrument};

/// 0. who: the client_id of the author.
/// 1. what: the actual message.
pub(super) type InterClientMessage = (String, MessageValue);

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
        broadcast::channel::<InterClientMessage>(CHANNEL_SIZE);

    // Shutdown cancellation token, to cooperatively & gracefully end all awaiting running
    // tasks. Calling `abort()` isn't reliable. Neither is dropping the task. More info:
    // https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/
    let shutdown_token = CancellationToken::new();

    // Set up Ctrl-C handler.
    setup_ctrlc_handler(shutdown_token.clone()).await?;

    // Create the kv store.
    let store = load_or_create_store(None)?;

    info!("Listening for new connections");

    // Server infinite loop - accept connections.
    loop {
        let shutdown_token_clone = shutdown_token.clone();
        let store_clone = store.clone();
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
                        store_clone.clone(),
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
    use crate::{MyClientMessage, MyServerMessage};

    use super::*;

    /// This has an infinite loop, so you might want to call it in a spawn block.
    #[instrument(name = "handle_client_task:main_loop", skip_all, fields(client_id))]
    pub async fn event_loop(
        client_id: &str,
        client_tcp_stream: TcpStream,
        sender_inter_client_broadcast_channel: Sender<InterClientMessage>,
        shutdown_token: CancellationToken,
        store: Store,
    ) -> miette::Result<()> {
        tracing::Span::current().record(CLIENT_ID_TRACING_FIELD, client_id);
        debug!("Handling client task");

        // Get the receiver for the inter client channel.
        let mut receiver_inter_client_broadcast_channel =
            sender_inter_client_broadcast_channel.subscribe();

        // Get reader and writer from TCP stream.
        let (read_half, write_half) = client_tcp_stream.into_split();
        let mut buf_reader = BufReader::new(read_half);
        let mut buf_writer = BufWriter::new(write_half);

        // Send the client ID.
        byte_io::try_write(&mut buf_writer, &{
            let server_message = MyServerMessage::SetClientId(client_id.to_string());
            debug!(?server_message, "Sent to client");
            server_message
        })
        .await?;

        info!("Entering infinite loop to handle client messages");

        // Infinite server loop.
        loop {
            tokio::select! {
                // Branch 1: Read from client.
                result = byte_io::try_read::<_, MyClientMessage>(&mut buf_reader) => {
                    let client_message = result?;
                    if handle_client_message(
                        client_message,
                        client_id,
                        &store,
                        &mut buf_writer,
                        sender_inter_client_broadcast_channel.clone()
                    ).await.is_err() {
                        break;
                    }
                }

                // Branch 2: Read from broadcast channel.
                result = receiver_inter_client_broadcast_channel.recv() => {
                    match result {
                        Ok(payload) => {
                            let payload_buffer = generate_server_message::try_handle_broadcast(
                                client_id,
                                payload
                            ).await?;
                            if let Some(ref payload) = payload_buffer {
                                byte_io::try_write(&mut buf_writer, payload).await?;
                            }
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
                    byte_io::try_write(&mut buf_writer, &MyServerMessage::Exit).await?;
                    break;
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(client_id))]
    pub async fn handle_client_message<Writer: AsyncWrite + Unpin>(
        client_message: MyClientMessage,
        client_id: &str,
        store: &Store,
        buf_writer: &mut BufWriter<Writer>,
        sender_inter_client_broadcast_channel: Sender<InterClientMessage>,
    ) -> miette::Result<()> {
        info!(?client_message, "Received message from client");
        let bucket = load_or_create_bucket_from_store::<MessageKey, MessageValue>(store, None)?;

        match client_message {
            ClientMessage::BroadcastToOthers(payload) => {
                byte_io::try_write(
                    buf_writer,
                    &generate_server_message::try_broadcast_to_others(
                        client_id,
                        sender_inter_client_broadcast_channel,
                        payload,
                    )?,
                )
                .await?;
            }
            ClientMessage::Size => {
                byte_io::try_write(
                    buf_writer,
                    &generate_server_message::try_get_size_of_bucket(&bucket)?,
                )
                .await?;
            }
            ClientMessage::Clear => {
                byte_io::try_write(
                    buf_writer,
                    &generate_server_message::try_clear_bucket(&bucket)?,
                )
                .await?;
            }
            ClientMessage::Get(key) => {
                byte_io::try_write(
                    buf_writer,
                    &generate_server_message::try_get_from_bucket(&bucket, key)?,
                )
                .await?;
            }
            ClientMessage::Remove(key) => {
                byte_io::try_write(
                    buf_writer,
                    &generate_server_message::try_remove_from_bucket(&bucket, key)?,
                )
                .await?;
            }
            ClientMessage::Insert(key, value) => {
                byte_io::try_write(
                    buf_writer,
                    &generate_server_message::try_insert_into_bucket(&bucket, key, value)?,
                )
                .await?;
            }
            ClientMessage::GetAll => {
                byte_io::try_write(
                    buf_writer,
                    &generate_server_message::try_get_all_items_from_bucket(&bucket)?,
                )
                .await?;
            }
            ClientMessage::Exit => {
                info!("Exiting due to client request");
                return Err(miette!("Client requested exit"));
            }
        }

        Ok(())
    }
}

mod generate_server_message {
    use super::*;

    #[instrument(skip(sender_inter_client_broadcast_channel), fields(client_id))]
    pub(super) fn try_broadcast_to_others<'a>(
        client_id: &str,
        sender_inter_client_broadcast_channel: Sender<InterClientMessage>,
        payload: MessageValue,
    ) -> miette::Result<MyServerMessage> {
        sender_inter_client_broadcast_channel
            .send((client_id.to_string(), payload))
            .into_diagnostic()?;
        Ok(ServerMessage::BroadcastToOthersAck({
            let count = sender_inter_client_broadcast_channel.receiver_count();
            match count {
                0 => 0,
                // Subtract the current client from the count.
                _ => count - 1,
            }
        }))
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_get_size_of_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
    ) -> miette::Result<MyServerMessage> {
        Ok(ServerMessage::Size(bucket.len()))
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_clear_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
    ) -> miette::Result<MyServerMessage> {
        let clear_status_flag = match bucket.clear() {
            Ok(_) => true,
            Err(error) => {
                error!(%error, "Problem clearing bucket");
                false
            }
        };
        Ok(ServerMessage::Clear(clear_status_flag))
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_get_from_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
        key: MessageKey,
    ) -> miette::Result<MyServerMessage> {
        let maybe_value = match get_from_bucket(bucket, key) {
            Ok(value) => value,
            Err(error) => {
                error!(%error, "Problem getting from bucket");
                None
            }
        };
        Ok(ServerMessage::Get(maybe_value))
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_get_all_items_from_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
    ) -> miette::Result<MyServerMessage> {
        let mut item_vec: Vec<(MessageKey, MessageValue)> = vec![];
        iterate_bucket(bucket, |key: MessageKey, value: MessageValue| {
            item_vec.push((key, value));
        });
        Ok(ServerMessage::GetAll(item_vec))
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_remove_from_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
        key: MessageKey,
    ) -> miette::Result<MyServerMessage> {
        let remove_status_flag = match remove_from_bucket(bucket, key) {
            Ok(value) => value.is_some(),
            Err(error) => {
                error!(%error, "Problem removing from bucket");
                false
            }
        };
        Ok(ServerMessage::Remove(remove_status_flag))
    }

    #[instrument(skip(bucket), fields(client_id))]
    pub(super) fn try_insert_into_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
        key: MessageKey,
        value: MessageValue,
    ) -> miette::Result<MyServerMessage> {
        let insert_status_flag = match insert_into_bucket(bucket, key, value) {
            Ok(_) => true,
            Err(error) => {
                error!(%error, "Problem inserting into bucket");
                false
            }
        };
        Ok(ServerMessage::Insert(insert_status_flag))
    }

    /// Filter out the client_id that sent the message.
    #[instrument(skip_all, fields(client_id))]
    pub async fn try_handle_broadcast(
        client_id: &str,
        payload: InterClientMessage,
    ) -> miette::Result<Option<MyServerMessage>> {
        // Filter out the client_id that sent the message.
        let (sender_client_id, payload) = payload;
        if sender_client_id == client_id {
            return Ok(None);
        }
        // Send the payload to all other clients.
        Ok(Some(ServerMessage::HandleBroadcast(payload)))
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
pub mod test_handle_client_message {
    use crate::{
        handle_client_task::handle_client_message, insert_into_bucket,
        load_or_create_bucket_from_store, load_or_create_store,
        server_task::generate_server_message, test_fixtures::MockTcpStreamWrite, Buffer,
        ClientMessage, Data, InterClientMessage, ServerMessage, CHANNEL_SIZE,
    };
    use miette::IntoDiagnostic;
    use tempfile::tempdir;
    use tokio::{io::BufWriter, sync::broadcast};

    /// More info: <https://tokio.rs/tokio/topics/testing>
    #[tokio::test]
    async fn test_try_get_all_items_from_bucket() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;
        let bucket = load_or_create_bucket_from_store::<crate::MessageKey, crate::MessageValue>(
            &store, None,
        )?;
        let key = "foo";
        let data = &Data::default();
        insert_into_bucket(&bucket, key.to_string(), data.clone())?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let expected_payload_bytes = {
            let item_vec = vec![(key.to_string(), data.clone())];
            let server_message = ServerMessage::GetAll(item_vec);
            bincode::serialize(&server_message).into_diagnostic()?
        };

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
            broadcast::channel::<InterClientMessage>(CHANNEL_SIZE).0,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = expected_payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(expected_payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    #[tokio::test]
    async fn test_try_insert_into_bucket() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let expected_payload_bytes = {
            let server_message = ServerMessage::<String, Data>::Insert(true);
            bincode::serialize(&server_message).into_diagnostic()?
        };

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
            broadcast::channel::<InterClientMessage>(CHANNEL_SIZE).0,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = expected_payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(expected_payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    #[tokio::test]
    async fn test_try_remove_from_bucket() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;
        let bucket = load_or_create_bucket_from_store::<crate::MessageKey, crate::MessageValue>(
            &store, None,
        )?;
        insert_into_bucket(&bucket, "foo".to_string(), Data::default())?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let expected_payload_bytes = {
            let server_message = ServerMessage::<String, Data>::Remove(true);
            bincode::serialize(&server_message).into_diagnostic()?
        };

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
            broadcast::channel::<InterClientMessage>(CHANNEL_SIZE).0,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = expected_payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(expected_payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    #[tokio::test]
    async fn test_try_get_from_bucket() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;
        let bucket = load_or_create_bucket_from_store::<crate::MessageKey, crate::MessageValue>(
            &store, None,
        )?;

        // Insert some data into the bucket.
        let key = "foo";
        let data = &Data::default();
        insert_into_bucket(&bucket, key.to_string(), data.clone())?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let expected_payload_bytes = {
            let it = Some(data.clone());
            let server_message = ServerMessage::<String, Data>::Get(it);
            bincode::serialize(&server_message).into_diagnostic()?
        };

        // Create a mock writer (for the write half of the TcpStream).
        let writer = MockTcpStreamWrite {
            expected_write: Vec::new(),
        };
        let mut buf_writer = BufWriter::new(writer);

        // Prepare the actual payload, with length-prefix from [byte_io::write]. This will
        // be accumulated in the buf_writer.
        handle_client_message(
            ClientMessage::Get("foo".to_string()),
            "test_client_id",
            &store,
            &mut buf_writer,
            broadcast::channel::<InterClientMessage>(CHANNEL_SIZE).0,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = expected_payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(expected_payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    #[tokio::test]
    async fn test_try_clear_bucket() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;
        let bucket = load_or_create_bucket_from_store::<crate::MessageKey, crate::MessageValue>(
            &store, None,
        )?;
        insert_into_bucket(&bucket, "foo".to_string(), Data::default())?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let expected_payload_bytes = {
            let server_message = ServerMessage::<String, Data>::Clear(true);
            bincode::serialize(&server_message).into_diagnostic()?
        };

        // Create a mock writer (for the write half of the TcpStream).
        let writer = MockTcpStreamWrite {
            expected_write: Vec::new(),
        };
        let mut buf_writer = BufWriter::new(writer);

        // Prepare the actual payload, with length-prefix from [byte_io::write]. This will
        // be accumulated in the buf_writer.
        handle_client_message(
            ClientMessage::Clear,
            "test_client_id",
            &store,
            &mut buf_writer,
            broadcast::channel::<InterClientMessage>(CHANNEL_SIZE).0,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = expected_payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(expected_payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    #[tokio::test]
    async fn test_try_get_size_of_bucket() -> miette::Result<()> {
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;
        let bucket = load_or_create_bucket_from_store::<crate::MessageKey, crate::MessageValue>(
            &store, None,
        )?;
        insert_into_bucket(&bucket, "foo".to_string(), Data::default())?;

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let expected_payload_bytes = {
            let server_message = ServerMessage::<String, Data>::Size(1);
            bincode::serialize(&server_message).into_diagnostic()?
        };

        // Create a mock writer (for the write half of the TcpStream).
        let writer = MockTcpStreamWrite {
            expected_write: Vec::new(),
        };
        let mut buf_writer = BufWriter::new(writer);

        // Prepare the actual payload, with length-prefix from [byte_io::write]. This will
        // be accumulated in the buf_writer.
        handle_client_message(
            ClientMessage::Size,
            "test_client_id",
            &store,
            &mut buf_writer,
            broadcast::channel::<InterClientMessage>(CHANNEL_SIZE).0,
        )
        .await?;

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = expected_payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(expected_payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    #[tokio::test]
    async fn test_try_broadcast_to_others() -> miette::Result<()> {
        // Store.
        let dir = tempdir().expect("Failed to create temp dir");
        let store = load_or_create_store(Some(&dir.path().to_string_lossy().to_string()))?;

        // Channel.
        let (sender, mut receiver_1) = broadcast::channel::<InterClientMessage>(CHANNEL_SIZE);
        let mut receiver_2 = sender.subscribe();
        let expected_count = 1; // There are 2 receivers, but the sender is not counted.

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let expected_payload_bytes = {
            let server_message =
                ServerMessage::<String, Data>::BroadcastToOthersAck(expected_count);
            bincode::serialize(&server_message).into_diagnostic()?
        };

        // Create a mock writer (for the write half of the TcpStream).
        let writer = MockTcpStreamWrite {
            expected_write: Vec::new(),
        };
        let mut buf_writer = BufWriter::new(writer);

        let id = "test_client_id";
        let data = Data::default();

        // Prepare the actual payload, with length-prefix from [byte_io::write]. This will
        // be accumulated in the buf_writer.
        handle_client_message(
            ClientMessage::BroadcastToOthers(data),
            id,
            &store,
            &mut buf_writer,
            sender.clone(),
        )
        .await?;

        // Assert the message was sent to the channel.
        {
            let (sent_id, sent_data) = receiver_1.try_recv().into_diagnostic()?;
            assert_eq!(sent_id, "test_client_id");
            assert_eq!(sent_data, Data::default());

            let (sent_id, sent_data) = receiver_2.try_recv().into_diagnostic()?;
            assert_eq!(sent_id, "test_client_id");
            assert_eq!(sent_data, Data::default());
        }

        // println!("actual bytes  : {:?}", buf_writer.get_ref().expected_write);

        // Assert the actual bytes w/ the expected bytes.
        let mut result_vec: Buffer = vec![];
        let length_prefix = expected_payload_bytes.len() as u64;
        let length_prefix_bytes = length_prefix.to_be_bytes();
        result_vec.extend_from_slice(length_prefix_bytes.as_ref());
        result_vec.extend(expected_payload_bytes);

        assert_eq!(buf_writer.get_ref().expected_write, result_vec);

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_broadcast_channel_between_clients() -> miette::Result<()> {
        let self_id = "self_id";
        let other_id = "other_id";
        let payload = &Data::default();

        // Get the bytes that are expected to be sent to the client (not including the
        // length prefix).
        let expected_payload_bytes = {
            let server_message = ServerMessage::<String, Data>::HandleBroadcast(payload.clone());
            bincode::serialize(&server_message).into_diagnostic()?
        };

        // Prepare the actual payload.
        let actual_payload = generate_server_message::try_handle_broadcast(
            self_id,
            (other_id.to_string(), payload.clone()),
        )
        .await?;

        let actual_payload_bytes = actual_payload
            .map(|payload| bincode::serialize(&payload).into_diagnostic())
            .unwrap()
            .unwrap();

        // println!("actual bytes  : {:?}", actual_payload_bytes);

        // Assert the actual bytes w/ the expected bytes.
        assert_eq!(actual_payload_bytes, expected_payload_bytes);

        Ok(())
    }
}
