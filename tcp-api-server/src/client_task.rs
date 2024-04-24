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

use crate::{byte_io, MessageKey, MessageValue};
use crate::{protocol, CLIArg, ClientMessage, CLIENT_ID_TRACING_FIELD};
use crossterm::style::Stylize;
use miette::{Context, IntoDiagnostic};
use r3bl_terminal_async::{
    ReadlineEvent, SharedWriter, Spinner, SpinnerStyle, StdMutex, TerminalAsync,
};
use std::{
    io::{stderr, Write},
    ops::ControlFlow,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tracing::{error, info, instrument};

const DELAY_MS: u64 = 100;
const DELAY_UNIT: Duration = Duration::from_millis(DELAY_MS);
const ARTIFICIAL_UI_DELAY: Duration = Duration::from_millis(DELAY_MS * 12);

/// The `client_id` field is added to the span, so that it can be used in the logs by
/// functions called by this one. See also: [crate::CLIENT_ID_TRACING_FIELD].
#[instrument(skip_all, fields(client_id))]
pub async fn client_main(
    cli_args: CLIArg,
    mut terminal_async: TerminalAsync,
) -> miette::Result<()> {
    // Connect to the server.
    let address = format!("{}:{}", cli_args.address, cli_args.port);

    let message_trying_to_connect = format!("Trying to connect to server on {}", &address);

    // Start spinner, pause terminal.
    terminal_async.pause().await;

    let maybe_spinner = Spinner::try_start(
        message_trying_to_connect.clone(),
        DELAY_UNIT,
        SpinnerStyle {
            template: r3bl_terminal_async::SpinnerTemplate::Braille,
            ..Default::default()
        },
        Arc::new(StdMutex::new(stderr())),
        terminal_async.clone_shared_writer(),
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
    terminal_async.resume().await;

    info!("{}", message_trying_to_connect);
    let tcp_stream = result?;
    info!("Connected to server on {}", &address);

    // Get reader and writer from TCP stream.
    let (read_half, write_half) = tcp_stream.into_split();

    // Broadcast shutdown channel to end all tasks (cooperatively).
    let (shutdown_sender, _) = tokio::sync::broadcast::channel::<bool>(1);

    // SPAWN TASK: Listen to server messages.
    // Handle messages from the server in a separate task. This will ensure that both the
    // spawned tasks don't block each other (eg: if either of them sleeps).
    tokio::spawn(monitor_tcp_conn_task::enter_loop(
        read_half,
        terminal_async.clone_shared_writer(),
        shutdown_sender.clone(),
    ));

    // DON'T SPAWN TASK: User input event infinite loop.
    let _ = monitor_user_input::enter_loop(write_half, terminal_async, shutdown_sender).await;

    Ok(())
}

pub mod monitor_user_input {
    use super::*;

    /// This has an infinite loop, so you might want to spawn a task before calling it.
    /// And it has a blocking call, so you can't exit it preemptively.
    #[instrument(name = "user_input:main_loop", skip_all, fields(client_id))]
    pub async fn enter_loop(
        write_half: OwnedWriteHalf,
        mut terminal_async: TerminalAsync,
        shutdown_sender: tokio::sync::broadcast::Sender<bool>,
    ) -> miette::Result<()> {
        // Artificial delay to let all the other tasks settle.
        tokio::time::sleep(DELAY_UNIT).await;

        let welcome_message = format!(
            "{}, eg: {}, etc.",
            "Enter a message".yellow().bold(),
            "exit, info, getall, clear".green().bold()
        );

        // 00: implement info which simply collects the `ClientMessage.iter()` (just like in the test)
        terminal_async.println(welcome_message).await;

        let mut buf_writer = BufWriter::new(write_half);

        let mut shutdown_receiver = shutdown_sender.subscribe();

        loop {
            tokio::select! {
                // Poll shutdown channel.
                _ = shutdown_receiver.recv() => {
                    break;
                }

                // Poll user input.
                result_readline_event = terminal_async.get_readline_event() => {
                    let readline_event = result_readline_event?;
                    match readline_event {
                        ReadlineEvent::Line(input) => {
                            // Parse the input into a ClientMessage.
                            let result_user_input_message =
                                protocol::ClientMessage::<MessageKey, MessageValue>::from_str(&input);
                                // Same as: input.parse::<protocol::ClientMessage>();
                            match result_user_input_message {
                                Ok(user_input_message) => {
                                    terminal_async.pause().await;
                                    let control_flow = handle_user_input(
                                        user_input_message,
                                        &mut buf_writer,
                                        shutdown_sender.clone(),
                                        terminal_async.clone_shared_writer(),
                                    )
                                    .await;
                                    terminal_async.resume().await;
                                    if let ControlFlow::Break(_) = control_flow {
                                        break;
                                    }
                                }
                                Err(_) => {
                                    terminal_async
                                        .println_prefixed(format!(
                                            "Unknown command: {}",
                                            input.red().bold()
                                        ))
                                        .await;
                                }
                            }
                        }
                        ReadlineEvent::Eof => {
                            // 00: do something meaningful w/ the EOF event
                            let _ = shutdown_sender.send(true);
                            break;
                        }
                        ReadlineEvent::Interrupted => {
                            // 00: do something meaningful w/ the Interrupted event
                            let _ = shutdown_sender.send(true);
                            break;
                        }
                    }
                }
            }
        }

        info!("Exiting loop");

        terminal_async.flush().await;

        Ok(())
    }

    /// Please refer to the [ClientMessage] enum in the [protocol] module for the list of
    /// commands.
    #[instrument(skip_all, fields(client_message))]
    pub async fn handle_user_input(
        client_message: ClientMessage<MessageKey, MessageValue>,
        buf_writer: &mut BufWriter<OwnedWriteHalf>,
        shutdown_sender: tokio::sync::broadcast::Sender<bool>,
        mut shared_writer: SharedWriter,
    ) -> ControlFlow<()> {
        match client_message {
            ClientMessage::Exit => {
                let result_maybe_spinner = Spinner::try_start(
                    "Sending exit message".to_string(),
                    DELAY_UNIT,
                    SpinnerStyle {
                        template: r3bl_terminal_async::SpinnerTemplate::Block,
                        ..Default::default()
                    },
                    Arc::new(StdMutex::new(stderr())),
                    shared_writer.clone(),
                )
                .await;

                // Artificial delay to see the spinner spin.
                tokio::time::sleep(ARTIFICIAL_UI_DELAY).await;

                // Ignore the result of the write operation, since the client is exiting.
                if let Ok(payload_buffer) =
                    bincode::serialize(&protocol::ClientMessage::<MessageKey, MessageValue>::Exit)
                {
                    let _ = byte_io::write(buf_writer, payload_buffer).await;
                }

                // Stop progress bar.
                if let Ok(Some(mut spinner)) = result_maybe_spinner {
                    let _ = spinner.stop("Sent exit message").await;
                }

                // Send the shutdown signal to all tasks.
                let _ = shutdown_sender.send(true);

                return ControlFlow::Break(());
            }
            _ => {
                // 00: do something meaningful w/ the other client messages
                let msg = format!("TODO! implement: {:?}", client_message);
                let _ = writeln!(shared_writer, "{}", msg);
            }
        }

        ControlFlow::Continue(())
    }
}

const DEFAULT_CLIENT_ID: &str = "none";

pub mod monitor_tcp_conn_task {
    use super::*;

    /// This has an infinite loop, so you might want to call it in a spawn block.
    #[instrument(name = "monitor_tcp_conn_task:main_loop", fields(client_id), skip_all)]
    pub async fn enter_loop(
        buf_reader: OwnedReadHalf,
        mut shared_writer: SharedWriter,
        shutdown_sender: tokio::sync::broadcast::Sender<bool>,
    ) -> miette::Result<()> {
        let mut buf_reader = BufReader::new(buf_reader);

        // This will be set by the server when the client connects.
        let mut client_id = DEFAULT_CLIENT_ID.to_string();

        info!("Entering loop");

        let mut shutdown_receiver = shutdown_sender.subscribe();

        // 00: listen for data from the server, handle it, monitor shutdown channel.
        loop {
            tokio::select! {
                result_payload_buffer = byte_io::read(&mut buf_reader) => {
                    match result_payload_buffer {
                        Ok(payload_buffer) => {
                            let server_message =
                                bincode::deserialize::<protocol::ServerMessage<MessageKey, MessageValue>>(&payload_buffer)
                                    .into_diagnostic()?;
                            handle_server_message(server_message, &mut client_id, shared_writer.clone())
                                .await?;
                        }
                        Err(error) => {
                            let _ = writeln!(
                                shared_writer,
                                "Error reading from server for client task w/ 'client_id': {}",
                                client_id.to_string().yellow().bold(),
                            );
                            error!(?error);
                            let _ = shutdown_sender.send(true);
                            break;
                        }
                    }

                }
                _ = shutdown_receiver.recv() => {
                    break;
                }
            }
        }

        info!("Exiting loop");

        Ok(())
    }

    #[instrument(skip_all, fields(server_message, client_id))]
    async fn handle_server_message(
        server_message: protocol::ServerMessage<MessageKey, MessageValue>,
        client_id: &mut String,
        mut shared_writer: SharedWriter,
    ) -> miette::Result<()> {
        // Display the message to the console.
        let msg = format!(
            "[{}]: {}: {}",
            client_id.to_string().yellow().bold(),
            "Received message from server".green().bold(),
            format!("{:?}", server_message).magenta().bold(),
        );
        let _ = writeln!(shared_writer, "{}", msg);

        // Process the message.
        info!(?server_message, "Start");
        match server_message {
            protocol::ServerMessage::SetClientId(ref id) => {
                client_id.clone_from(id);
                tracing::Span::current().record(CLIENT_ID_TRACING_FIELD, &client_id);
            }
            _ => {
                // 00: do something meaningful with the server message (eg: set the client_id)
                todo!()
            }
        };

        info!(?server_message, "End");

        Ok(())
    }
}
