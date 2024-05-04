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

use crate::{byte_io, Buffer, CLIArg, MessageValue, CLIENT_ID_TRACING_FIELD};
use crossterm::style::Stylize;
use miette::{Context, IntoDiagnostic};
use r3bl_rs_utils_core::generate_friendly_random_id;
use r3bl_terminal_async::{
    ReadlineEvent, SharedWriter, Spinner, SpinnerStyle, StdMutex, TerminalAsync,
};
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
};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use tracing::{error, info, instrument};

// Constants.
const DELAY_MS: u64 = 75;
const DELAY_UNIT: Duration = Duration::from_millis(DELAY_MS);
const ARTIFICIAL_UI_DELAY: Duration = Duration::from_millis(DELAY_MS * 10);

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

    // Reserve a space for the client_id. This is set for this entire client task.
    let client_id = Arc::new(StdMutex::new(DEFAULT_CLIENT_ID.to_string()));

    // Shutdown cancellation token, to cooperatively & gracefully end all awaiting running
    // tasks. Calling `abort()` isn't reliable. Neither is dropping the task. More info:
    // https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/
    let shutdown_token = CancellationToken::new();

    // SPAWN TASK: Listen to server messages.
    // Handle messages from the server in a separate task. This will ensure that both the
    // spawned tasks don't block each other (eg: if either of them sleeps).
    tokio::spawn(monitor_tcp_conn_task::event_loop(
        read_half,
        terminal_async.clone_shared_writer(),
        shutdown_token.clone(),
        client_id.clone(),
    ));

    // DON'T SPAWN TASK: User input event infinite loop.
    let _ =
        monitor_user_input::event_loop(write_half, terminal_async, shutdown_token, client_id).await;

    Ok(())
}

pub mod monitor_user_input {
    use crate::MyClientMessage;

    use super::*;

    mod spinner_support {
        use super::*;

        pub async fn create(
            message: String,
            shared_writer: SharedWriter,
        ) -> miette::Result<Option<Spinner>> {
            let result_maybe_spinner = Spinner::try_start(
                message,
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

    /// This has an infinite loop, so you might want to spawn a task before calling it.
    /// And it has a blocking call, so you can't exit it preemptively.
    #[instrument(name = "user_input:main_loop", skip_all, fields(client_id))]
    pub async fn event_loop(
        write_half: OwnedWriteHalf,
        mut terminal_async: TerminalAsync,
        shutdown_token: CancellationToken,
        client_id: Arc<StdMutex<String>>,
    ) -> miette::Result<()> {
        // Artificial delay to let all the other tasks settle.
        tokio::time::sleep(DELAY_UNIT).await;

        let items = {
            let mut items = vec![];
            for item in MyClientMessage::iter() {
                let item = item.to_string().to_lowercase();
                terminal_async.readline.add_history_entry(item.clone());
                items.push(item.green().bold().to_string());
            }
            items.join(", ")
        };
        let welcome_message = format!("{}, eg: {}, etc.", "Enter a message".yellow().bold(), items);

        terminal_async.println(welcome_message).await;

        let mut buf_writer = BufWriter::new(write_half);
        let shutdown_token_clone = shutdown_token.clone();

        loop {
            tokio::select! {
                // Poll shutdown cancellation token.
                _ = shutdown_token.cancelled() => {
                    break;
                }

                // Poll user input.
                result_readline_event = terminal_async.get_readline_event() => {
                    let readline_event = result_readline_event?;
                    match readline_event {
                        ReadlineEvent::Line(input) => {
                            // Parse the input into a ClientMessage.
                            let result_parse = MyClientMessage::try_parse_input(&input);
                            match result_parse {
                                Ok((client_message, rest)) => {
                                    let result_send = send_client_message(
                                        client_message,
                                        rest,
                                        &mut buf_writer,
                                        shutdown_token_clone.clone(),
                                        terminal_async.clone_shared_writer(),
                                        client_id.clone(),
                                    )
                                    .await;
                                    match result_send {
                                        ControlFlow::Break(_) => break,
                                        ControlFlow::Continue(_) => continue,
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
                        ReadlineEvent::Eof | ReadlineEvent::Interrupted => {
                            shutdown_token.cancel();
                            // Ignore the result of the write operation, since the client is exiting.
                            let _ = byte_io::try_write(
                                &mut buf_writer,
                                &MyClientMessage::Exit,
                            ).await;

                            // Delay to allow messages to be printed to display output.
                            tokio::time::sleep(DELAY_UNIT).await;
                            break;
                        }
                        ReadlineEvent::Resized => {}
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
    pub async fn send_client_message(
        client_message: MyClientMessage,
        rest: String,
        buf_writer: &mut BufWriter<OwnedWriteHalf>,
        shutdown_token: CancellationToken,
        mut shared_writer: SharedWriter,
        client_id: Arc<StdMutex<String>>,
    ) -> ControlFlow<()> {
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
                        description: format!("from: '{}'", client_id.lock().unwrap().clone()),
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
                        "get".green(),
                        "<key>".yellow().bold()
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
                        "remove".green(),
                        "<key>".yellow().bold()
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
                    let key = generate_friendly_random_id();
                    let value = MessageValue {
                        id: rand::random(),
                        description: format!("from: '{}'", client_id.lock().unwrap().clone()),
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
                shutdown_token.cancel();

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

pub mod monitor_tcp_conn_task {
    use crate::MyServerMessage;

    use super::*;

    /// This has an infinite loop, so you might want to call it in a spawn block.
    #[instrument(name = "monitor_tcp_conn_task:main_loop", fields(client_id), skip_all)]
    pub async fn event_loop(
        buf_reader: OwnedReadHalf,
        mut shared_writer: SharedWriter,
        shutdown_token: CancellationToken,
        client_id: Arc<StdMutex<String>>,
    ) -> miette::Result<()> {
        let mut buf_reader = BufReader::new(buf_reader);

        info!("Entering loop");

        loop {
            tokio::select! {
                // Poll the TCP stream for data.
                result_payload = byte_io::try_read::<_, MyServerMessage>(&mut buf_reader) => {
                    match result_payload {
                        Ok(server_message) => {
                            handle_server_message(
                                server_message,
                                client_id.clone(),
                                shared_writer.clone(),
                                shutdown_token.clone()
                            ).await?;
                        }
                        Err(error) => {
                            let client_id_str = client_id.lock().unwrap().clone();
                            let _ = writeln!(
                                shared_writer,
                                "Error reading from server for client task w/ 'client_id': {}",
                                client_id_str.yellow().bold(),
                            );
                            error!(?error);
                            shutdown_token.cancel();
                            break;
                        }

                    }
                }

                // Poll the shutdown cancellation token.
                _ = shutdown_token.cancelled() => {
                    break;
                }
            }
        }

        info!("Exiting loop");

        Ok(())
    }

    #[instrument(skip_all, fields(server_message, client_id))]
    async fn handle_server_message(
        server_message: MyServerMessage,
        client_id: Arc<StdMutex<String>>,
        mut shared_writer: SharedWriter,
        shutdown_token: CancellationToken,
    ) -> miette::Result<()> {
        info!(?server_message, "Start");

        match server_message {
            MyServerMessage::HandleBroadcast(ref data) => {
                let msg = format!(
                    "[{}]: {}: {:#?}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received broadcast message from server".green().bold(),
                    data,
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::BroadcastToOthersAck(num_clients) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received ACK for broadcast message from server"
                        .white()
                        .on_dark_grey()
                        .bold(),
                    format!("Broadcast to {} clients", num_clients)
                        .magenta()
                        .bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Size(ref data) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received size message from server".green().bold(),
                    format!("{:?}", data).magenta().bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Clear(success_flag) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received clear message from server".green().bold(),
                    match success_flag {
                        true => "✅ Success".green().bold(),
                        false => "❌ Failure".red().bold(),
                    }
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Get(ref data) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received get message from server".green().bold(),
                    format!("{:?}", data).magenta().bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Remove(success_flag) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received remove message from server".green().bold(),
                    match success_flag {
                        true => "✅ Success".green().bold(),
                        false => "❌ Failure".red().bold(),
                    }
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Insert(success_flag) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received insert message from server".green().bold(),
                    match success_flag {
                        true => "✅ Success".green().bold(),
                        false => "❌ Failure".red().bold(),
                    }
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::GetAll(ref data) => {
                let msg = format!(
                    "[{}]: {}: {:#?}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received getall message from server".green().bold(),
                    data,
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            MyServerMessage::Exit => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received exit message from server".green().bold(),
                    "Shutting down client".red().bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);

                // Cancel the shutdown token to end the client.
                tokio::time::sleep(DELAY_UNIT).await; // Delay to allow the message above to be printed.
                shutdown_token.cancel();
            }
            MyServerMessage::SetClientId(ref id) => {
                *client_id.lock().unwrap() = id.to_string();
                tracing::Span::current().record(CLIENT_ID_TRACING_FIELD, id);
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received setclientid message from server"
                        .on_black()
                        .yellow()
                        .bold(),
                    format!("{:?}", id).magenta().bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
        };

        debug!(?server_message, "End");

        Ok(())
    }
}
