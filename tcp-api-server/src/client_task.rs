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

use crate::{print_output, protocol, CLIArg, ClientMessage, CHANNEL_SIZE};
use crossterm::style::Stylize;
use dialoguer::Input;
use indicatif::ProgressBar;
use miette::{miette, Context, ErrReport, IntoDiagnostic};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::{
    io::{BufReader, BufWriter},
    net::{tcp::OwnedReadHalf, TcpStream},
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};
use tracing::{error, info};

const DELAY_MS: u64 = 33;

#[derive(Debug)]
enum Message {
    Exit,
    ExitDueToError(miette::Report),
}

type MyJoinHandle = JoinHandle<Result<(), ErrReport>>;

pub async fn start_client(cli_args: CLIArg) -> miette::Result<()> {
    // Connect to the server.
    let address = format!("{}:{}", cli_args.address, cli_args.port);
    info!("Trying to connect to server on {}", &address);
    let tcp_stream = TcpStream::connect(&address)
        .await
        .into_diagnostic()
        .wrap_err(format!("Could not connect to the server at {address}"))?;
    info!("Connected to server on {}", &address);

    // Get reader and writer from TCP stream.
    let (read_half, write_half) = tcp_stream.into_split();

    // Create broadcast channel.
    let (sender_mpsc_channel, receiver_mpsc_channel) = mpsc::channel::<Message>(CHANNEL_SIZE);

    // Hold the task handles in a vector, so that they can be stopped when the user exits.
    let vec_tasks = Arc::new(Mutex::new(Vec::<MyJoinHandle>::new()));

    // SPAWN TASK 1: Listen to server messages.
    // Handle messages from the server in a separate task. This will ensure that both the
    // spawned tasks don't block each other (eg: if either of them sleeps).
    vec_tasks
        .lock()
        .unwrap()
        .push(tokio::spawn(monitor_tcp_connection_task(
            sender_mpsc_channel.clone(),
            read_half,
        )));

    // SPAWN TASK 2: Listen to channel messages.
    // Handle messages from the channel in a separate task.
    vec_tasks
        .clone()
        .lock()
        .unwrap()
        .push(tokio::spawn(monitor_mpsc_channel_task(
            receiver_mpsc_channel,
            vec_tasks.clone(),
        )));

    // DON'T SPAWN TASK: User input event infinite loop.
    // Sleep for 1 seconds to give the other tasks a chance to start
    tokio::time::sleep(Duration::from_millis(DELAY_MS)).await;
    main_event_infinite_loop(sender_mpsc_channel.clone(), write_half).await;

    Ok(())
}

async fn main_event_infinite_loop(
    sender_mpsc_channel: Sender<Message>,
    write_half: OwnedWriteHalf,
) {
    let mut buf_writer = BufWriter::new(write_half);
    loop {
        if let Some(input) = get_user_input().await {
            let message_result = input.parse::<protocol::ClientMessage>();
            match message_result {
                Ok(message) => {
                    match message {
                        ClientMessage::Exit => {
                            let bar =
                                ProgressBar::new_spinner().with_message("Sending exit message");
                            bar.enable_steady_tick(Duration::from_millis(DELAY_MS));

                            // Artificial delay to see the spinner spin.
                            tokio::time::sleep(Duration::from_millis(DELAY_MS * 15)).await;

                            // Ignore the result of the write, since the client is exiting.
                            if let Ok(payload_buffer) =
                                bincode::serialize(&protocol::ClientMessage::Exit)
                            {
                                let _ =
                                    protocol::write_bytes(&mut buf_writer, payload_buffer).await;
                            }

                            // Stop progress bar.
                            bar.finish_and_clear();

                            // Send the exit message to the monitor_mpsc_channel_task.
                            let _ = sender_mpsc_channel.send(Message::Exit).await;
                        }
                        _ => {
                            // 00: do something meaningful w/ the other client messages
                            todo!()
                        }
                    }
                }
                _ => {
                    print_output(&format!("Unknown command: {}", input.red().bold()));
                }
            }
        }
    }
}

async fn monitor_mpsc_channel_task(
    mut receiver_mpsc_channel: mpsc::Receiver<Message>,
    vec_tasks_clone: Arc<Mutex<Vec<MyJoinHandle>>>,
) -> miette::Result<()> {
    loop {
        // 00: check all the exit codes to see that they are complete
        match receiver_mpsc_channel.recv().await {
            Some(Message::Exit) => {
                print_output("Exiting start_client due to user input");
                exit(vec_tasks_clone.clone(), 0);
                break;
            }
            Some(Message::ExitDueToError(error)) => {
                print_output(format!("Exiting start_client due to error: {}", error));
                exit(vec_tasks_clone.clone(), 1);
                break;
            }
            it => {
                print_output(format!("Unknown message from server: {:?}", it));
            }
        }
    }

    Ok(())
}

/// Have to exit this process since there is no way to get stdin().read_line() unblocked
/// once it is blocked. Even if that task is wrapped in a thread::spawn, it isn't possible
/// to cancel or abort that thread, without cooperatively asking it to exit.
///
/// More info:
/// - https://docs.rs/tokio/latest/tokio/io/struct.Stdin.html
/// - https://users.rust-lang.org/t/stopping-a-thread/6328/7
/// - https://internals.rust-lang.org/t/thread-cancel-support/3056/16
fn exit(vec_tasks: Arc<Mutex<Vec<MyJoinHandle>>>, code: i32) {
    // This will stop the listen_to_server_task_handle.
    vec_tasks.lock().unwrap().iter().for_each(|task| {
        task.abort();
    });
    std::process::exit(code);
}

async fn monitor_tcp_connection_task(
    sender_exit_channel: Sender<Message>,
    buf_reader: OwnedReadHalf,
) -> miette::Result<()> {
    info!("monitor_tcp_connection_task task started");
    let mut buf_reader = BufReader::new(buf_reader);
    // This will be set by the server when the client connects.
    let mut client_id = "none".to_string();

    loop {
        // 00: listen for data from the server, can send message over channel for TCP connection drop
        match protocol::read_bytes(&mut buf_reader).await {
            Ok(payload_buffer) => {
                let server_message =
                    bincode::deserialize::<protocol::ServerMessage>(&payload_buffer)
                        .into_diagnostic()?;
                print_output(format!(
                    "[{}]: {}: {:?}",
                    client_id.to_string().yellow().bold(),
                    "Received message from server".green().bold(),
                    server_message
                ));
                match server_message {
                    protocol::ServerMessage::SetClientId(id) => {
                        client_id = id;
                        info!(
                            "[{}]: {}",
                            client_id.to_string().yellow().bold(),
                            "SetClientId",
                        );
                    }
                    _ => {
                        // 00: do something meaningful with the server message (eg: set the client_id)
                        todo!()
                    }
                }
            }
            Err(error) => {
                error!(
                    "[{}]: {} encountered error: {}",
                    client_id.to_string().yellow().bold(),
                    "monitor_tcp_connection_task",
                    error
                );
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

async fn get_user_input() -> Option<String> {
    match Input::<String>::new()
        .with_prompt(format!("{}", "Enter a message".yellow().bold()))
        .interact()
    {
        Ok(input) => {
            print_output(format!("You entered: {}", input));
            Some(input)
        }
        Err(error) => {
            print_output(format!("Error: {}", error));
            None
        }
    }
}
