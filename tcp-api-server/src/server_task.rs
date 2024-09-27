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
    byte_io, protocol::ServerMessage, CLIArg, ClientMessage, MessageKey, MessageValue,
    MyClientMessage, MyServerMessage, CHANNEL_SIZE,
};
use crossterm::style::Stylize;
use kv::Store;
use miette::{miette, IntoDiagnostic};
use r3bl_rs_utils_core::{
    friendly_random_id, get_from_bucket, insert_into_bucket, iterate_bucket,
    load_or_create_bucket_from_store, load_or_create_store, remove_from_bucket, KVBucket,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::{
    io::{AsyncWrite, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self},
};
use tracing::{debug, error, info, instrument};

/// 0. who: the client_id of the author.
/// 1. what: the actual message.
pub(super) type InterClientMessage = (String, MessageValue);

#[instrument(skip_all)]
pub async fn server_main_event_loop(cli_args: CLIArg) -> miette::Result<()> {
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

    // Keep track of the number of connected clients.
    let safe_connected_client_count = Arc::new(AtomicUsize::new(0));

    // Use broadcast channel for shutting down the server, to cooperatively & gracefully
    // end all awaiting running tasks.
    // Use this in favor of:
    // 1. `abort()` - behavior is undefined / inconsistent.
    // 2. Dropping the task is not reliable.
    // 3. `CancellationToken` from `tokio_util` crate - does not work the way that
    //    broadcast channel or other channels do. It doesn't block when `is_cancelled()`
    //    is called, and creates a strange behavior in `tokio::select!` blocks, causing
    //    the loop to be run repeatedly.
    // More info: <https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/>
    let (shutdown_sender, mut shutdown_receiver) = broadcast::channel::<()>(1);

    // Create the kv store.
    let store = load_or_create_store(None)?;

    info!("Listening for new connections");

    // Server main event loop - accept connections.
    loop {
        tokio::select! {
            // Branch 1: Accept incoming connections (this is not blocking and doesn't tie
            // up a thread 🎉).
            result /*: Result<(TcpStream, SocketAddr), Error> */ = listener.accept() => {
                let (client_tcp_stream, _) = result.into_diagnostic()?;

                // Clone all the things to move into tokio::spawn.
                let store_clone = store.clone();
                let sender_inter_client_broadcast_channel_clone =
                    sender_inter_client_broadcast_channel.clone();
                let safe_connected_client_count_clone = safe_connected_client_count.clone();
                let shutdown_sender_clone = shutdown_sender.clone();
                let shutdown_receiver_clone = shutdown_sender_clone.subscribe();

                // Start task to handle a connection. Note that there might be n of these
                // tasks spawned where n is the number of connected clients.
                tokio::spawn(async move {
                    // Increment the connected client count.
                    safe_connected_client_count_clone.fetch_add(1, Ordering::SeqCst);

                    let client_id = friendly_random_id::generate_friendly_random_id();
                    let result_handle_client_task = handle_client_task::event_loop(
                        &client_id,
                        client_tcp_stream,
                        sender_inter_client_broadcast_channel_clone,
                        shutdown_sender_clone.clone(),
                        store_clone.clone(),
                        safe_connected_client_count_clone.clone(),
                    ).await;

                    // Decrement the connected client count.
                    safe_connected_client_count_clone.fetch_sub(1, Ordering::SeqCst);

                    // If Ctrl-C signal is received and there are no connected clients,
                    // then exit.
                    let ctrl_c_signal_received = !shutdown_receiver_clone.is_empty();
                    let no_clients_are_connected = safe_connected_client_count_clone.load(Ordering::SeqCst) == 0;
                    if ctrl_c_signal_received && no_clients_are_connected {
                        info!(
                            "{}",
                            "Send signal to shutdown channel, connected clients: 0" /* .to_string().yellow() */
                        );
                        shutdown_sender_clone.send(()).ok();
                    }

                    // Log the result of the client task.
                    match result_handle_client_task
                    {
                        Err(error) => error!(client_id, %error, ?safe_connected_client_count_clone, "Problem handling client task"),
                        Ok(_) => info!(client_id, ?safe_connected_client_count_clone, "Successfully ended client task"),
                    }

                });
            }

            // Branch 2: Monitor shutdown broadcast channel.
            _ = shutdown_receiver.recv() => {
                if safe_connected_client_count.load(Ordering::SeqCst) == 0 {
                    info!(
                        "{}",
                        "Received signal in shutdown channel - No connected clients, exiting main loop" /* .to_string().dark_red() */
                    );
                    break;
                } else {
                    info!(
                        "{}",
                        "Received signal in shutdown channel - Waiting for connected clients to reach 0" /*.to_string().dark_yellow() */
                    );
                }
            }

            // Branch 3: Monitor Ctrl-C.
            _ = tokio::signal::ctrl_c() => {
                println!(
                    "{}",
                    "Ctrl-C event detected. Gracefully shutting down..."
                        .yellow()
                        .bold()
                );
                info!("{}", "Ctrl-C event detected. Gracefully shutting down...");
                shutdown_sender.send(()).ok();
            }
        }
    }

    println!("Goodbye! 👋");

    Ok(())
}

pub mod handle_client_task {
    use super::*;

    /// This has an infinite loop, so you might want to call it in a spawn block. Server
    /// shutdown policy - this function can't affect the main event loop. It only affects
    /// the client task (it is responsible for sending an Exit message to it's connected
    /// client) when Ctrl+C is detected.
    #[instrument(name = "handle_client_task:event_loop", skip_all, fields(%client_id))]
    pub async fn event_loop(
        client_id: &str,
        client_tcp_stream: TcpStream,
        sender_inter_client_broadcast_channel: broadcast::Sender<InterClientMessage>,
        shutdown_sender: broadcast::Sender<()>,
        store: Store,
        safe_connected_client_count: Arc<AtomicUsize>,
    ) -> miette::Result<()> {
        info!(
            "Handling client connection, connected client count: {}",
            safe_connected_client_count.load(Ordering::SeqCst)
        );

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

        let mut shutdown_receiver = shutdown_sender.subscribe();

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

                // Branch 3: Monitor Ctrl-C shutdown broadcast channel. Note that this
                // code runs n-times where n is the number of connected clients (each
                // client is spawned a green thread / tokio task.
                _ = shutdown_receiver.recv() => {
                    info!("Received Ctrl-C signal");

                    // Send Exit message to client (don't do anything if it fails).
                    let _ = byte_io::try_write(&mut buf_writer, &MyServerMessage::Exit).await;
                    info!("Sent Exit server message to client");

                    break;
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(?client_message))]
    pub async fn handle_client_message<Writer: AsyncWrite + Unpin>(
        client_message: MyClientMessage,
        client_id: &str,
        store: &Store,
        buf_writer: &mut BufWriter<Writer>,
        sender_inter_client_broadcast_channel: broadcast::Sender<InterClientMessage>,
    ) -> miette::Result<()> {
        info!("Handling client message");

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

    #[instrument(skip_all, fields(?payload))]
    pub(super) fn try_broadcast_to_others<'a>(
        client_id: &str,
        sender_inter_client_broadcast_channel: broadcast::Sender<InterClientMessage>,
        payload: MessageValue,
    ) -> miette::Result<MyServerMessage> {
        info!("Broadcasting to others");
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

    #[instrument(skip_all, fields(bucket_len = bucket.len()))]
    pub(super) fn try_get_size_of_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
    ) -> miette::Result<MyServerMessage> {
        info!("Getting size of bucket");
        Ok(ServerMessage::Size(bucket.len()))
    }

    #[instrument(skip_all)]
    pub(super) fn try_clear_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
    ) -> miette::Result<MyServerMessage> {
        info!("Clearing bucket");
        let clear_status_flag = match bucket.clear() {
            Ok(_) => true,
            Err(error) => {
                error!(%error, "Problem clearing bucket");
                false
            }
        };
        Ok(ServerMessage::Clear(clear_status_flag))
    }

    #[instrument(skip_all, fields(?key))]
    pub(super) fn try_get_from_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
        key: MessageKey,
    ) -> miette::Result<MyServerMessage> {
        info!("Getting from bucket");
        let maybe_value = match get_from_bucket(bucket, key) {
            Ok(value) => value,
            Err(error) => {
                error!(%error, "Problem getting from bucket");
                None
            }
        };
        Ok(ServerMessage::Get(maybe_value))
    }

    #[instrument(skip_all)]
    pub(super) fn try_get_all_items_from_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
    ) -> miette::Result<MyServerMessage> {
        info!("Getting all items from bucket");
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

    #[instrument(skip_all, fields(?key, ?value))]
    pub(super) fn try_insert_into_bucket<'a>(
        bucket: &KVBucket<'a, MessageKey, MessageValue>,
        key: MessageKey,
        value: MessageValue,
    ) -> miette::Result<MyServerMessage> {
        info!("Inserting into bucket");
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
    #[instrument(skip_all, fields(?payload))]
    pub async fn try_handle_broadcast(
        client_id: &str,
        payload: InterClientMessage,
    ) -> miette::Result<Option<MyServerMessage>> {
        // Filter out the client_id that sent the message.
        let (sender_client_id, payload) = payload;
        if sender_client_id == client_id {
            info!("Ignoring broadcast from self");
            return Ok(None);
        }
        // Send the payload to all other clients.
        info!("Handling broadcast");
        Ok(Some(ServerMessage::HandleBroadcast(payload)))
    }
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
        handle_client_task::handle_client_message, server_task::generate_server_message,
        test_fixtures::MockTcpStreamWrite, Buffer, ClientMessage, Data, InterClientMessage,
        ServerMessage, CHANNEL_SIZE,
    };
    use miette::IntoDiagnostic;
    use r3bl_rs_utils_core::{
        insert_into_bucket, load_or_create_bucket_from_store, load_or_create_store,
    };
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
