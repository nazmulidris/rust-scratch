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
use r3bl_tui::{
    network_io::{byte_io, handshake, protocol_types::Buffer},
    rla_println, rla_println_prefixed,
};
use r3bl_tui::{fg_light_yellow_green, fg_pink, fg_lizard_green, fg_frozen_blue, fg_white};

use crate::{CLIArg, MessageValue, MyClientMessage, MyServerMessage};
use miette::{Context, IntoDiagnostic};
use r3bl_tui::{generate_friendly_random_id, ok, SharedWriter, SpinnerTemplate, StdMutex};
use r3bl_tui::{ReadlineAsync, ReadlineEvent, Spinner, SpinnerStyle};
use std::{
    io::{stderr, Write},
    ops::ControlFlow,
    sync::Arc,
    time::Duration,
};
use strum::IntoEnumIterator;
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::broadcast::{self},
};
use tracing::{error, info, instrument, Span};

// Constants.
const DELAY_MS: u64 = 75;
const DELAY_UNIT: Duration = Duration::from_millis(DELAY_MS);
const ARTIFICIAL_UI_DELAY: Duration = Duration::from_millis(DELAY_MS * 10);

#[instrument(skip_all)]
pub async fn client_entry_point(
    cli_args: CLIArg,
    mut readline_async: ReadlineAsync,
) -> miette::Result<()> {
    // Connect to the server.
    let address = format!("{}:{}", cli_args.address, cli_args.port);

    let message_trying_to_connect = format!("Trying to connect to server on {}", &address);

    // Start spinner, pause terminal.
    readline_async.pause().await;

    let maybe_spinner = Spinner::try_start(
        message_trying_to_connect.clone(),
        DELAY_UNIT,
        SpinnerStyle {
            template: SpinnerTemplate::Braille,
            ..Default::default()
        },
        Arc::new(StdMutex::new(stderr())),
        readline_async.clone_shared_writer(),
    )
    .await?;

    // Artificial delay to see the spinner spin.
    tokio::time::sleep(ARTIFICIAL_UI_DELAY).await;

    let result = TcpStream::connect(&address)
        .await
        .into_diagnostic()
        .wrap_err(format!(
            "Couldn't connect to the server at {address}. Are you sure it is up?"
        ));

    // Stop progress bar, resume terminal.
    if let Some(mut spinner) = maybe_spinner {
        let _ = spinner.stop("Connected to server").await;
    }
    readline_async.resume().await;

    info!("{}", message_trying_to_connect);
    let tcp_stream = result?;
    info!("Connected to server on {}", &address);

    // Get reader and writer from TCP stream.
    let (mut read_half, mut write_half) = tcp_stream.into_split();

    // Ensure that you are connecting to the correct server.
    handshake::try_connect_or_timeout(&mut read_half, &mut write_half).await?;

    // Reserve a space for the client_id. This is set for this entire client task.
    let safe_client_id = Arc::new(StdMutex::new(DEFAULT_CLIENT_ID.to_string()));

    // Use broadcast channel for shutting down the server, to cooperatively & gracefully
    // end all awaiting running tasks.
    // Use this in favor of:
    // 1. `abort()` - behavior is undefined / inconsistent.
    // 2. Dropping the task is unreliable.
    // 3. `CancellationToken` from `tokio_util` crate - does not work the way that
    //    broadcast channel or other channels do. It doesn't block when `is_cancelled()`
    //    is called, and creates a strange behavior in `tokio::select!` blocks, causing
    //    the loop to be run repeatedly.
    // More info: <https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/>
    let (shutdown_sender, _) = broadcast::channel::<()>(1);

    // SPAWN TASK: Listen to server messages.
    // Handle messages from the server in a separate task. This will ensure that both the
    // spawned tasks don't block each other (eg: if either of them sleeps).
    tokio::spawn(monitor_tcp_conn_task::event_loop(
        read_half,
        readline_async.clone_shared_writer(),
        shutdown_sender.clone(),
        safe_client_id.clone(),
    ));

    // DON'T SPAWN TASK: User input event infinite loop.
    let _ = monitor_user_input::event_loop(
        write_half,
        readline_async,
        shutdown_sender.clone(),
        safe_client_id,
    )
    .await;

    ok!()
}

pub mod monitor_user_input {
    use super::*;

    mod spinner_support {
        use r3bl_tui::SpinnerTemplate;

        use super::*;

        pub async fn create(
            message: String,
            shared_writer: SharedWriter,
        ) -> miette::Result<Option<Spinner>> {
            let result_maybe_spinner = Spinner::try_start(
                message,
                DELAY_UNIT,
                SpinnerStyle {
                    template: SpinnerTemplate::Block,
                    ..Default::default()
                },
                Arc::new(StdMutex::new(stderr())),
                shared_writer.clone(),
            )
            .await;

            // Artificial delay to see the spinner spin.
            tokio::time::sleep(ARTIFICIAL_UI_DELAY).await;

            result_maybe_spinner
        }

        pub async fn stop(
            message: String,
            result_maybe_spinner: miette::Result<Option<Spinner>>,
        ) -> miette::Result<()> {
            if let Ok(Some(mut spinner)) = result_maybe_spinner {
                let _ = spinner.stop(&message).await;
            }
            Ok(())
        }
    }

    /// - This has an infinite loop, so you might want to spawn a task before calling it.
    ///   And it has a blocking call, so you can't exit it preemptively.
    /// - Inject the `safe_client_id` into the call chain for tracing. All other async
    ///   functions which are instrumented will also have this field embedded in their log
    ///   output.
    #[instrument(name = "monitor_user_input:event_loop", skip_all, fields(client_id))]
    pub async fn event_loop(
        write_half: OwnedWriteHalf,
        mut readline_async: ReadlineAsync,
        shutdown_sender: broadcast::Sender<()>,
        safe_client_id: Arc<StdMutex<String>>,
    ) -> miette::Result<()> {
        info!("Entering loop");

        let items = {
            let mut fmt_items = vec![];
            for item in MyClientMessage::iter() {
                let item = item.to_string();
                readline_async.readline.add_history_entry(item.to_lowercase());
                fmt_items.push(fg_lizard_green(item).bold().to_string());
            }
            fmt_items.join(", ")
        };

        rla_println!(
            readline_async,
            "{}, eg: {}, etc.",
            fg_light_yellow_green("Enter a message").bold(),
            items
        );

        let mut buf_writer = BufWriter::new(write_half);
        let mut shutdown_receiver = shutdown_sender.subscribe();

        // Used to record the value of `client_id` only once.
        let mut self_client_id: Option<String> = None;

        loop {
            tokio::select! {
                // Poll shutdown cancellation token.
                _ = shutdown_receiver.recv() => {
                    break;
                }

                // Poll user input.
                result_readline_event = readline_async.read_line() => {
                    let readline_event = result_readline_event?;
                    match readline_event {
                        ReadlineEvent::Line(input) => {
                            // Parse the input to MyClientMessage enum.
                            let result_parse = MyClientMessage::try_parse_input(&input);

                            // Set the client_id for this client task, only once.
                            if self_client_id.is_none() {
                                let it = safe_client_id.lock().unwrap().clone();
                                if it != DEFAULT_CLIENT_ID {
                                    Span::current().record(CLIENT_ID_FIELD, it.clone());
                                    self_client_id = Some(it);
                                }
                            }

                            match result_parse {
                                Ok((client_message, rest)) => {
                                    let result_send = send_client_message(
                                        client_message,
                                        rest,
                                        &mut buf_writer,
                                        shutdown_sender.clone(),
                                        readline_async.clone_shared_writer(),
                                        safe_client_id.clone(),
                                    )
                                    .await;
                                    match result_send {
                                        ControlFlow::Break(_) => break,
                                        ControlFlow::Continue(_) => continue,
                                    }
                                }
                                Err(_) => {
                                    rla_println_prefixed!(readline_async, "Unknown command: {input}");
                                }
                            }
                        }
                        ReadlineEvent::Eof | ReadlineEvent::Interrupted => {
                            shutdown_sender.send(()).ok();
                            // Ignore the result of the write operation, since the client is exiting.
                            let _ = byte_io::try_write(
                                &mut buf_writer,
                                &MyClientMessage::Exit,
                            ).await;

                            // Delay to allow messages to be printed to display output.
                            break;
                        }
                        ReadlineEvent::Resized => {}
                    }
                }
            }
        }

        info!("Exiting loop");

        readline_async.flush().await;

        readline_async
            .exit(Some("Goodbye! 👋"))
            .await
            .into_diagnostic()?;

        Ok(())
    }

    /// Please refer to the [crate::ClientMessage] enum in the [crate::protocol] module for the list of
    /// commands.
    #[instrument(skip_all, fields(?client_message, %rest))]
    pub async fn send_client_message(
        client_message: MyClientMessage,
        rest: String,
        buf_writer: &mut BufWriter<OwnedWriteHalf>,
        shutdown_sender: broadcast::Sender<()>,
        mut shared_writer: SharedWriter,
        safe_client_id: Arc<StdMutex<String>>,
    ) -> ControlFlow<()> {
        info!("Sending client message");

        // Default control flow. Set to break if there is an error.
        let mut control_flow = ControlFlow::Continue(());

        match client_message {
            MyClientMessage::BroadcastToOthers(_) => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Send the broadcast message to the server.
                byte_io::try_write(buf_writer, &{
                    let value = MessageValue {
                        description: format!("from: '{}'", safe_client_id.lock().unwrap().clone()),
                        ..Default::default()
                    };
                    MyClientMessage::BroadcastToOthers(value)
                })
                .await
                .map_err(|_| {
                    control_flow = ControlFlow::Break(());
                })
                .ok();

                // Stop spinner.
                spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await
                    .ok();
            }
            MyClientMessage::Size => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Send the size message to the server.
                byte_io::try_write(buf_writer, &MyClientMessage::Size)
                    .await
                    .map_err(|_| {
                        control_flow = ControlFlow::Break(());
                    })
                    .ok();

                // Stop spinner.
                spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await
                    .ok();
            }
            MyClientMessage::Clear => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Send the clear message to the server.
                byte_io::try_write(buf_writer, &MyClientMessage::Clear)
                    .await
                    .map_err(|_| {
                        control_flow = ControlFlow::Break(());
                    })
                    .ok();

                // Stop spinner.
                spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await
                    .ok();
            }
            MyClientMessage::Get(_) => {
                // No key provided.
                if rest.is_empty() {
                    let msg = format!(
                        "Please provide a key to get, eg: {} {}",
                        fg_lizard_green("get"),
                        fg_light_yellow_green("<key>").bold()
                    );
                    writeln!(shared_writer, "{}", msg).ok();
                    return control_flow;
                }

                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Send the get message to the server.
                byte_io::try_write(buf_writer, &MyClientMessage::Get(rest))
                    .await
                    .map_err(|_| {
                        control_flow = ControlFlow::Break(());
                    })
                    .ok();

                // Stop spinner.
                spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await
                    .ok();
            }
            MyClientMessage::Remove(_) => {
                // No key provided.
                if rest.is_empty() {
                    let msg = format!(
                        "Please provide a key to remove, eg: {} {}",
                        fg_lizard_green("remove"),
                        fg_light_yellow_green("<key>").bold()
                    );
                    let _ = writeln!(shared_writer, "{}", msg);
                    return control_flow;
                }

                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Send the remove message to the server.
                byte_io::try_write(buf_writer, &MyClientMessage::Remove(rest))
                    .await
                    .map_err(|_| {
                        control_flow = ControlFlow::Break(());
                    })
                    .ok();

                // Stop spinner.
                spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await
                    .ok();
            }
            MyClientMessage::Insert(_, _) => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Send the insert message to the server.
                byte_io::try_write(buf_writer, &{
                    let key = generate_friendly_random_id().to_string();
                    let value = MessageValue {
                        id: rand::random(),
                        description: format!("from: '{}'", safe_client_id.lock().unwrap().clone()),
                        data: Buffer::from("data"),
                    };
                    MyClientMessage::Insert(key, value)
                })
                .await
                .map_err(|_| {
                    control_flow = ControlFlow::Break(());
                })
                .ok();

                // Stop spinner.
                spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await
                    .ok();
            }
            MyClientMessage::GetAll => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Send the getall message to the server.
                byte_io::try_write(buf_writer, &MyClientMessage::GetAll)
                    .await
                    .map_err(|_| {
                        control_flow = ControlFlow::Break(());
                    })
                    .ok();

                // Stop spinner.
                spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await
                    .ok();
            }
            MyClientMessage::Exit => {
                // Break out of the loop.
                control_flow = ControlFlow::Break(());

                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Ignore the result of the write operation, since the client is exiting.
                byte_io::try_write(buf_writer, &MyClientMessage::Exit)
                    .await
                    .ok();

                // Send the shutdown signal to all tasks.
                shutdown_sender.send(()).ok();

                // Stop spinner.
                spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await
                    .ok();
            }
        };

        control_flow
    }
}

const DEFAULT_CLIENT_ID: &str = "none";
const CLIENT_ID_FIELD: &str = "client_id";

pub mod monitor_tcp_conn_task {
    use super::*;

    /// - This has an infinite loop, so you might want to call it in a spawn block.
    /// - Inject the `safe_client_id` into the call chain for tracing. All other async
    ///   functions which are instrumented will also have this field embedded in their log
    ///   output.
    #[instrument(name = "monitor_tcp_conn_task:event_loop", skip_all, fields(client_id))]
    pub async fn event_loop(
        read_half: OwnedReadHalf,
        mut shared_writer: SharedWriter,
        shutdown_sender: broadcast::Sender<()>,
        safe_client_id: Arc<StdMutex<String>>,
    ) -> miette::Result<()> {
        info!("Entering loop");

        let mut buf_reader = BufReader::new(read_half);
        let mut shutdown_receiver = shutdown_sender.subscribe();
        loop {
            tokio::select! {
                // Poll the TCP stream for data.
                result_payload = byte_io::try_read::<_, MyServerMessage>(&mut buf_reader) => {
                    match result_payload {
                        Ok(server_message) => {
                            if let Some(new_client_id) = handle_server_message(
                                server_message,
                                safe_client_id.clone(),
                                shared_writer.clone(),
                                shutdown_sender.clone()
                            ).await? {
                                Span::current().record(CLIENT_ID_FIELD, &new_client_id);
                            }
                        }
                        Err(error) => {
                            let _ = writeln!(
                                shared_writer,
                                "Error reading from server for client task w/ 'client_id': {}",
                                fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                            );
                            error!(?error);
                            shutdown_sender.send(()).ok();
                            break;
                        }

                    }
                }

                // Poll the shutdown cancellation token.
                _ = shutdown_receiver.recv() => {
                    break;
                }
            }
        }

        info!("Exiting loop");

        Ok(())
    }

    #[instrument(skip_all, fields(?server_message))]
    async fn handle_server_message(
        server_message: MyServerMessage,
        safe_client_id: Arc<StdMutex<String>>,
        mut shared_writer: SharedWriter,
        shutdown_sender: broadcast::Sender<()>,
    ) -> miette::Result<Option<String>> {
        info!("Handling server message");

        match server_message {
            MyServerMessage::Exit => {
                let msg = format!(
                    "[{}]: {}: {}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received exit message from server").bold(),
                    fg_pink("Shutting down client").bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);

                // Cancel the shutdown token to end the client.
                shutdown_sender.send(()).ok();
            }
            MyServerMessage::SetClientId(ref id) => {
                // Save the new client ID.
                *safe_client_id.lock().unwrap() = id.to_string();
                let msg = format!(
                    "[{}]: {}: {}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received setclientid message from server").bg_moonlight_blue().bold(),
                    fg_frozen_blue(&format!("{:?}", id)).bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
                return Ok(Some(id.to_string()));
            }
            MyServerMessage::HandleBroadcast(ref data) => {
                let msg = format!(
                    "[{}]: {}: {:#?}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received broadcast message from server").bold(),
                    data,
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::BroadcastToOthersAck(num_clients) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_white("Received ACK for broadcast message from server").bg_moonlight_blue().bold(),
                    fg_frozen_blue(&format!("Broadcast to {} clients", num_clients)).bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Size(ref data) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received size message from server").bold(),
                    fg_frozen_blue(&format!("{:?}", data)).bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Clear(success_flag) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received clear message from server").bold(),
                    match success_flag {
                        true => fg_lizard_green("✅ Success").bold(),
                        false => fg_pink("❌ Failure").bold(),
                    }
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Get(ref data) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received get message from server").bold(),
                    fg_frozen_blue(&format!("{:?}", data)).bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Remove(success_flag) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received remove message from server").bold(),
                    match success_flag {
                        true => fg_lizard_green("✅ Success").bold(),
                        false => fg_pink("❌ Failure").bold(),
                    }
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Insert(success_flag) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received insert message from server").bold(),
                    match success_flag {
                        true => fg_lizard_green("✅ Success").bold(),
                        false => fg_pink("❌ Failure").bold(),
                    }
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::GetAll(ref data) => {
                let msg = format!(
                    "[{}]: {}: {:#?}",
                    fg_light_yellow_green(safe_client_id.lock().unwrap().as_str()).bold(),
                    fg_lizard_green("Received getall message from server").bold(),
                    data,
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
        };

        Ok(None)
    }
}
