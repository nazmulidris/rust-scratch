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
    protocol, CLIArg, ClientMessage, ProgressBarAsync, TerminalAsync, CHANNEL_SIZE,
    CLIENT_ID_TRACING_FIELD,
};
use crossterm::style::Stylize;
use miette::{miette, Context, ErrReport, IntoDiagnostic};
use std::{
    ops::ControlFlow,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc::{self, Sender},
    task::AbortHandle,
};
use tracing::{error, info, instrument};

const DELAY_MS: u64 = 33;
const DELAY_UNIT: Duration = Duration::from_millis(DELAY_MS);
const ARTIFICIAL_UI_DELAY: Duration = Duration::from_millis(DELAY_MS * 15);

#[derive(Debug)]
pub enum Message {
    Exit,
    ExitDueToError(miette::Report),
}

/// The `client_id` field is added to the span, so that it can be used in the logs by
/// functions called by this one. See also: [crate::CLIENT_ID_TRACING_FIELD].
#[instrument(skip_all, fields(client_id))]
pub async fn start_client(cli_args: CLIArg, terminal_async: TerminalAsync) -> miette::Result<()> {
    // Connect to the server.
    let address = format!("{}:{}", cli_args.address, cli_args.port);

    let message_trying_to_connect = format!("Trying to connect to server on {}", &address);

    let mut bar = ProgressBarAsync::try_new_and_start(
        message_trying_to_connect.clone(),
        DELAY_UNIT,
        terminal_async.clone(),
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

    // Stop progress bar.
    bar.finish_with_message("Connected to server").await;

    info!("{}", message_trying_to_connect);
    let tcp_stream = result?;
    info!("Connected to server on {}", &address);

    // Get reader and writer from TCP stream.
    let (read_half, write_half) = tcp_stream.into_split();

    // Create channel to control client lifecycle.
    let (client_lifecycle_control_sender, client_lifecycle_control_receiver) =
        mpsc::channel::<Message>(CHANNEL_SIZE);

    // Hold the task handles in a vector, so that they can be stopped when the user exits.
    let task_abort_handles = Arc::new(Mutex::new(Vec::<AbortHandle>::new()));

    // SPAWN TASK 1: Listen to server messages.
    // Handle messages from the server in a separate task. This will ensure that both the
    // spawned tasks don't block each other (eg: if either of them sleeps).
    task_abort_handles.lock().unwrap().push({
        let future = monitor_tcp_conn_task::main_loop(
            client_lifecycle_control_sender.clone(),
            read_half,
            terminal_async.clone(),
        );
        let join_handle = tokio::spawn(future);
        join_handle.abort_handle()
    });

    // SPAWN TASK 2: Listen to channel messages.
    // Handle messages from the channel in a separate task.
    task_abort_handles.clone().lock().unwrap().push({
        let future = monitor_lifecycle_channel_task::main_loop(
            client_lifecycle_control_receiver,
            task_abort_handles.clone(),
            terminal_async.clone(),
        );
        let join_handle = tokio::spawn(future);
        join_handle.abort_handle()
    });

    // DON'T SPAWN TASK: User input event infinite loop.
    let _ = user_input::main_loop(
        client_lifecycle_control_sender.clone(),
        write_half,
        terminal_async.clone(),
    )
    .await;

    Ok(())
}

pub mod user_input {
    use super::*;

    /// This has an infinite loop, so you might want to spawn a task before calling it.
    /// And it has a blocking call, so you can't exit it preemptively.
    #[instrument(name = "user_input:main_loop", skip_all, fields(client_id))]
    pub async fn main_loop(
        client_lifecycle_control_sender: Sender<Message>,
        write_half: OwnedWriteHalf,
        mut terminal_async: TerminalAsync,
    ) -> miette::Result<()> {
        // Artificial delay to let all the other tasks settle.
        tokio::time::sleep(DELAY_UNIT).await;

        let prompt = format!(
            "{}, eg: {}, etc.",
            "Enter a message".yellow().bold(),
            "exit, get_all, clear".green().bold()
        );
        terminal_async.println_raw_and_flush(prompt).await;

        let mut buf_writer = BufWriter::new(write_half);
        loop {
            match terminal_async.get_readline_event().await? {
                rustyline_async::ReadlineEvent::Line(input) => {
                    // Parse the input into a ClientMessage.
                    let result_user_input_message = protocol::ClientMessage::from_str(&input); // input.parse::<protocol::ClientMessage>();
                    match result_user_input_message {
                        Ok(user_input_message) => {
                            handle_user_input(
                                user_input_message,
                                &mut buf_writer,
                                &client_lifecycle_control_sender,
                                terminal_async.clone(),
                            )
                            .await;
                        }
                        Err(_) => {
                            terminal_async
                                .print_output_and_flush(format!(
                                    "Unknown command: {}",
                                    input.red().bold()
                                ))
                                .await;
                        }
                    }
                }
                rustyline_async::ReadlineEvent::Eof => {
                    // 00: do something meaningful w/ the EOF event
                    todo!()
                }
                rustyline_async::ReadlineEvent::Interrupted => {
                    // 00: do something meaningful w/ the Interrupted event
                    todo!()
                }
            }
        }
    }

    /// Please refer to the [ClientMessage] enum in the [protocol] module for the list of
    /// commands.
    #[instrument(skip_all, fields(client_message))]
    pub async fn handle_user_input(
        client_message: ClientMessage,
        buf_writer: &mut BufWriter<OwnedWriteHalf>,
        client_lifecycle_control_channel_sender: &Sender<Message>,
        terminal_async: TerminalAsync,
    ) {
        match client_message {
            ClientMessage::Exit => {
                let bar = ProgressBarAsync::try_new_and_start(
                    "Sending exit message".to_string(),
                    DELAY_UNIT,
                    terminal_async,
                )
                .await;

                // Artificial delay to see the spinner spin.
                tokio::time::sleep(ARTIFICIAL_UI_DELAY).await;

                // Ignore the result of the write operation, since the client is exiting.
                if let Ok(payload_buffer) = bincode::serialize(&protocol::ClientMessage::Exit) {
                    let _ = protocol::write_bytes(buf_writer, payload_buffer).await;
                }

                // Stop progress bar.
                if let Ok(mut bar) = bar {
                    bar.finish_with_message("Sent exit message").await;
                }

                // Send the exit message to the monitor_mpsc_channel_task.
                let _ = client_lifecycle_control_channel_sender
                    .send(Message::Exit)
                    .await;
            }
            _ => {
                // 00: do something meaningful w/ the other client messages
                todo!()
            }
        }
    }
}

pub mod monitor_lifecycle_channel_task {
    use super::*;

    /// This has an infinite loop, so you might want to call it in a spawn block.
    #[instrument(
        name = "monitor_lifecycle_channel_task:main_loop",
        skip_all,
        fields(client_id)
    )]
    pub async fn main_loop(
        mut client_lifecycle_control_receiver: mpsc::Receiver<Message>,
        arc_vec_abort_handles: Arc<Mutex<Vec<AbortHandle>>>,
        terminal_async: TerminalAsync,
    ) -> miette::Result<()> {
        loop {
            // 00: check all the exit codes to see that they are complete
            if let ControlFlow::Break(_) = handle_client_lifecycle_channel_message(
                &mut client_lifecycle_control_receiver,
                &arc_vec_abort_handles,
                terminal_async.clone(),
            )
            .await
            {
                break;
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(client_id))]
    pub async fn handle_client_lifecycle_channel_message(
        receiver_mpsc_channel: &mut mpsc::Receiver<Message>,
        arc_vec_abort_handles: &Arc<Mutex<Vec<AbortHandle>>>,
        mut terminal_async: TerminalAsync,
    ) -> ControlFlow<()> {
        match receiver_mpsc_channel.recv().await {
            Some(Message::Exit) => {
                terminal_async
                    .print_output_and_flush("Exiting start_client due to user input")
                    .await;
                exit(arc_vec_abort_handles.clone(), 0);
                return ControlFlow::Break(());
            }
            Some(Message::ExitDueToError(error)) => {
                terminal_async
                    .print_output_and_flush(format!("Exiting start_client due to error: {}", error))
                    .await;
                exit(arc_vec_abort_handles.clone(), 1);
                return ControlFlow::Break(());
            }
            it => {
                terminal_async
                    .print_output_and_flush(format!("Unknown message from server: {:?}", it))
                    .await;
            }
        }
        ControlFlow::Continue(())
    }

    /// Have to exit this process since there is no way to get stdin().read_line() unblocked
    /// once it is blocked. Even if that task is wrapped in a thread::spawn, it isn't possible
    /// to cancel or abort that thread, without cooperatively asking it to exit.
    ///
    /// More info:
    /// - <https://docs.rs/tokio/latest/tokio/io/struct.Stdin.html>
    /// - <https://users.rust-lang.org/t/stopping-a-thread/6328/7>
    /// - <https://internals.rust-lang.org/t/thread-cancel-support/3056/16>
    pub fn exit(arc_vec_abort_handles: Arc<Mutex<Vec<AbortHandle>>>, code: i32) {
        // This will stop the listen_to_server_task_handle.
        arc_vec_abort_handles
            .lock()
            .unwrap()
            .iter()
            .for_each(|task| {
                task.abort();
            });
        std::process::exit(code);
    }
}

const DEFAULT_CLIENT_ID: &str = "none";

pub mod monitor_tcp_conn_task {
    use super::*;

    /// This has an infinite loop, so you might want to call it in a spawn block.
    #[instrument(name = "monitor_tcp_conn_task:main_loop", fields(client_id), skip_all)]
    pub async fn main_loop(
        sender_exit_channel: Sender<Message>,
        buf_reader: OwnedReadHalf,
        terminal_async: TerminalAsync,
    ) -> miette::Result<()> {
        let mut buf_reader = BufReader::new(buf_reader);

        // This will be set by the server when the client connects.
        let mut client_id = DEFAULT_CLIENT_ID.to_string();

        info!("Entering loop");

        loop {
            // 00: listen for data from the server, can send message over channel for TCP connection drop
            match protocol::read_bytes(&mut buf_reader).await {
                Ok(payload_buffer) => {
                    let server_message =
                        bincode::deserialize::<protocol::ServerMessage>(&payload_buffer)
                            .into_diagnostic()?;
                    handle_server_message(server_message, &mut client_id, terminal_async.clone())
                        .await?;
                }
                Err(error) => {
                    error!(?error);
                    let _ = sender_exit_channel
                        .send(Message::ExitDueToError(miette!(
                            "Error reading from server for client task w/ 'client_id': {}",
                            client_id.to_string().yellow().bold(),
                        )))
                        .await;
                    break;
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(server_message, client_id))]
    async fn handle_server_message(
        server_message: protocol::ServerMessage,
        client_id: &mut String,
        mut terminal_async: TerminalAsync,
    ) -> Result<(), ErrReport> {
        // Display the message to the console.
        terminal_async
            .print_output_and_flush(format!(
                "[{}]: {}: {}",
                client_id.to_string().yellow().bold(),
                "Received message from server".green().bold(),
                format!("{:?}", server_message).magenta().bold(),
            ))
            .await;

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
