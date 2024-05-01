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
    byte_io,
    protocol::{self, ClientMessage, ServerMessage},
    Buffer, CLIArg, MessageKey, MessageValue, CLIENT_ID_TRACING_FIELD,
};
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
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tokio_util::sync::CancellationToken;
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
    use strum::IntoEnumIterator;

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
            for item in ClientMessage::<String, String>::iter() {
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
                            let result = ClientMessage::<MessageKey, MessageValue>::parse_input(&input);
                            match result {
                                Ok((client_message, rest)) => {
                                    terminal_async.pause().await;
                                    let control_flow = send_client_message(
                                        client_message,
                                        rest,
                                        &mut buf_writer,
                                        shutdown_token_clone.clone(),
                                        terminal_async.clone_shared_writer(),
                                        client_id.clone(),
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
                        ReadlineEvent::Eof | ReadlineEvent::Interrupted => {
                            shutdown_token.cancel();
                            // Ignore the result of the write operation, since the client is exiting.
                            if let Ok(payload_buffer) =
                                bincode::serialize(&ClientMessage::<MessageKey, MessageValue>::Exit)
                            {
                                let _ = byte_io::write(&mut buf_writer, payload_buffer).await;
                            }
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
        client_message: ClientMessage<MessageKey, MessageValue>,
        rest: String,
        buf_writer: &mut BufWriter<OwnedWriteHalf>,
        shutdown_token: CancellationToken,
        mut shared_writer: SharedWriter,
        client_id: Arc<StdMutex<String>>,
    ) -> ControlFlow<()> {
        // Default control flow. Set to break if there is an error.
        let mut control_flow = ControlFlow::Continue(());

        // BOOKM: 1) send_client_message
        match client_message {
            ClientMessage::Clear => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Ignore the result of the write operation, since the client is exiting.
                if let Ok(payload_buffer) =
                    bincode::serialize(&ClientMessage::<MessageKey, MessageValue>::Clear)
                {
                    if byte_io::write(buf_writer, payload_buffer).await.is_err() {
                        control_flow = ControlFlow::Break(());
                    }
                }

                // Stop spinner.
                let _ = spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await;
            }
            ClientMessage::Get(_) => {
                // No key provided.
                if rest.is_empty() {
                    let msg = format!(
                        "Please provide a key to get, eg: {} {}",
                        "get".green(),
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

                // Get the key from `rest` param.
                if let Ok(payload_buffer) =
                    bincode::serialize(&ClientMessage::<MessageKey, MessageValue>::Get(rest))
                {
                    if byte_io::write(buf_writer, payload_buffer).await.is_err() {
                        control_flow = ControlFlow::Break(());
                    }
                }

                // Stop spinner.
                let _ = spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await;
            }
            ClientMessage::Remove(_) => {
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

                // Get the key from `rest` param.
                if let Ok(payload_buffer) =
                    bincode::serialize(&ClientMessage::<MessageKey, MessageValue>::Remove(rest))
                {
                    if byte_io::write(buf_writer, payload_buffer).await.is_err() {
                        control_flow = ControlFlow::Break(());
                    }
                }

                // Stop spinner.
                let _ = spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await;
            }
            ClientMessage::Insert(_, _) => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                let key = generate_friendly_random_id();
                let value = MessageValue {
                    id: rand::random(),
                    description: format!("from: '{}'", client_id.lock().unwrap().clone()),
                    data: Buffer::from("data"),
                };

                if let Ok(payload_buffer) =
                    bincode::serialize(&ClientMessage::<MessageKey, MessageValue>::Insert(
                        key, value,
                    ))
                {
                    if byte_io::write(buf_writer, payload_buffer).await.is_err() {
                        control_flow = ControlFlow::Break(());
                    }
                }

                // Stop spinner.
                let _ = spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await;
            }
            ClientMessage::GetAll => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                if let Ok(payload_buffer) =
                    bincode::serialize(&ClientMessage::<MessageKey, MessageValue>::GetAll)
                {
                    if byte_io::write(buf_writer, payload_buffer).await.is_err() {
                        control_flow = ControlFlow::Break(());
                    }
                }

                // Stop spinner.
                let _ = spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await;
            }
            ClientMessage::Exit => {
                // Start spinner.
                let spinner = spinner_support::create(
                    format!("Sending {} message", client_message),
                    shared_writer,
                )
                .await;

                // Ignore the result of the write operation, since the client is exiting.
                if let Ok(payload_buffer) =
                    bincode::serialize(&ClientMessage::<MessageKey, MessageValue>::Exit)
                {
                    let _ = byte_io::write(buf_writer, payload_buffer).await;
                }
                // Send the shutdown signal to all tasks.
                shutdown_token.cancel();
                control_flow = ControlFlow::Break(());

                // Stop spinner.
                let _ = spinner_support::stop(format!("Sent {} message", client_message), spinner)
                    .await;
            }
            // CLEANUP: remove this when all the cases above are handled
            _ => {
                let msg = format!("TODO! implement: {:?}", client_message);
                let _ = writeln!(shared_writer, "{}", msg);
            }
        };

        control_flow
    }
}

const DEFAULT_CLIENT_ID: &str = "none";

pub mod monitor_tcp_conn_task {
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
                result_payload_buffer = byte_io::read(&mut buf_reader) => {
                    match result_payload_buffer {
                        Ok(payload_buffer) => {
                            let server_message =
                                bincode::deserialize::<protocol::ServerMessage<MessageKey, MessageValue>>(&payload_buffer)
                                    .into_diagnostic()?;
                            handle_server_message(server_message, client_id.clone(), shared_writer.clone(), shutdown_token.clone())
                                .await?;
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
        server_message: ServerMessage<MessageKey, MessageValue>,
        client_id: Arc<StdMutex<String>>,
        mut shared_writer: SharedWriter,
        shutdown_token: CancellationToken,
    ) -> miette::Result<()> {
        info!(?server_message, "Start");

        // BOOKM: 3) handle_server_message
        match server_message {
            ServerMessage::Clear(success_flag) => {
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
            ServerMessage::Get(ref data) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received get message from server".green().bold(),
                    format!("{:?}", data).magenta().bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            ServerMessage::Remove(success_flag) => {
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
            ServerMessage::Insert(success_flag) => {
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
            ServerMessage::GetAll(ref data) => {
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received getall message from server".green().bold(),
                    format!("{:?}", data).magenta().bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
            ServerMessage::Exit => {
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
            ServerMessage::SetClientId(ref id) => {
                *client_id.lock().unwrap() = id.to_string();
                tracing::Span::current().record(CLIENT_ID_TRACING_FIELD, id);
            }
            // CLEANUP: remove when all other cases are handled
            _ => {
                // Display the message to the console.
                let msg = format!(
                    "[{}]: {}: {}",
                    client_id.lock().unwrap().to_string().yellow().bold(),
                    "Received message from server".green().bold(),
                    format!("{:?}", server_message).magenta().bold(),
                );
                let _ = writeln!(shared_writer, "{}", msg);
            }
        };

        info!(?server_message, "End");

        Ok(())
    }
}
