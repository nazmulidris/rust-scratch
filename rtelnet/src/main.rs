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

use clap::{Parser, Subcommand};
use r3bl_ansi_color::SgrCode;
use r3bl_tui::ColorWheel;
use std::thread;
use std::{
    io::{stdin, BufRead, BufReader, BufWriter, Write},
    net::{IpAddr, TcpListener, TcpStream},
};

use type_aliases::*;
mod type_aliases {
    pub type IOResult<T> = std::io::Result<T>;
}

use defaults::*;
mod defaults {
    pub const DEFAULT_PORT: u16 = 3000;
    pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
}

use clap_config::*;
mod clap_config {
    use super::*;

    #[derive(Parser, Debug)]
    pub struct CLIArg {
        /// IP Address to connect to or start a server on
        #[clap(long, short, default_value = DEFAULT_ADDRESS, global = true)]
        pub address: IpAddr,

        /// TCP Port to connect to or start a server on
        #[clap(long, short, default_value_t = DEFAULT_PORT, global = true)]
        pub port: u16,

        /// Logs to stdout by default, set this flag to disable it
        #[clap(long, short = 'd', global = true)]
        pub log_disable: bool,

        /// The subcommand to run
        #[clap(subcommand)]
        pub subcommand: CLISubcommand,
    }

    #[derive(Subcommand, Debug)]
    pub enum CLISubcommand {
        /// Start a server on the given address and port
        Server,
        /// Connect to a server running on the given address and port
        Client,
    }
}

fn main() {
    println!("Welcome to rtelnet");

    let cli_arg = CLIArg::parse();
    let address = cli_arg.address;
    let port = cli_arg.port;
    let socket_address = format!("{}:{}", address, port);

    if !cli_arg.log_disable {
        femme::start()
    }

    match match cli_arg.subcommand {
        CLISubcommand::Server => start_server(socket_address),
        CLISubcommand::Client => start_client(socket_address),
    } {
        Ok(_) => {
            println!("Program exited successfully");
        }
        Err(error) => {
            println!("Program exited with an error: {}", error);
        }
    }
}

use server::*;
mod server {
    use super::*;

    pub fn start_server(socket_address: String) -> IOResult<()> {
        let tcp_listener = TcpListener::bind(socket_address)?;
        // Server connection accept loop.
        loop {
            log::info!("Waiting for a incoming connection...");
            let (tcp_stream, ..) = tcp_listener.accept()?; // This is a blocking call.

            // Spawn a new thread to handle this connection.
            thread::spawn(|| match handle_connection(tcp_stream) {
                Ok(_) => {
                    log::info!("Successfully closed connection to client...");
                }
                Err(_) => {
                    log::error!("Problem with client connection...");
                }
            });
        }
    }

    fn handle_connection(tcp_stream: TcpStream) -> IOResult<()> {
        log::info!("Start handle connection");

        let reader = &mut BufReader::new(&tcp_stream);
        let write = &mut BufWriter::new(&tcp_stream);

        // Process client connection loop.
        loop {
            let mut incoming: Vec<u8> = vec![];

            // Read from reader.
            let num_bytes_read = reader.read_until(b'\n', &mut incoming)?;

            // Check for EOF. The stream is closed.
            if num_bytes_read == 0 {
                break;
            }

            // Process.
            let outgoing = process(&incoming);

            // Write to writer.
            write.write(&outgoing)?;
            let _ = write.flush()?;

            // Print debug.
            log::info!("-> Rx(bytes) : {:?}", &incoming);
            log::info!(
                "-> Rx(string): '{}', size: {} bytes",
                String::from_utf8_lossy(&incoming).trim(),
                incoming.len(),
            );
            log::info!(
                "<- Tx(string): '{}', size: {} bytes",
                String::from_utf8_lossy(&outgoing).trim(),
                outgoing.len()
            );
        }

        log::info!("End handle connection - connection closed");

        Ok(())
    }

    fn process(incoming: &Vec<u8>) -> Vec<u8> {
        // Convert incoming to String, and remove any trailing whitespace (includes newline).
        let incoming = String::from_utf8_lossy(incoming);
        let incoming = incoming.trim();

        // Prepare outgoing payload.
        let outgoing = incoming.to_string();

        // Colorize it w/ a gradient.
        let outgoing = ColorWheel::lolcat_into_string(&outgoing);

        // Generate outgoing response. Add newline to the end of output (so client can process it).
        let outgoing = format!("{}\n", outgoing);

        // Return outgoing payload.
        outgoing.as_bytes().to_vec()
    }
}

fn start_client(socket_address: String) -> IOResult<()> {
    log::info!("Start client connection");
    let tcp_stream = TcpStream::connect(socket_address)?;
    let (mut reader, mut writer) = (BufReader::new(&tcp_stream), BufWriter::new(&tcp_stream));

    // Client loop.
    loop {
        // Read user input.
        let outgoing = {
            let mut it = String::new();
            let _ = stdin().read_line(&mut it)?;
            it.as_bytes().to_vec()
        };

        // Exit if EOF (Ctrl+D pressed).
        if outgoing.len() == 0 {
            break;
        }

        // Tx user input to writer.
        let _ = writer.write(&outgoing)?;
        writer.flush()?;

        // Rx response from reader.
        let incoming = {
            let mut it = vec![];
            let _ = reader.read_until(b'\n', &mut it);
            it
        };

        // Check for EOF, and exit.
        if incoming.len() == 0 {
            break;
        }

        let display_msg = String::from_utf8_lossy(&incoming);
        let display_msg = display_msg.trim();

        let reset = SgrCode::Reset.to_string();
        let display_msg = format!("{}{}", display_msg, reset);
        println!("{}", display_msg);

        // Print debug.
        log::info!(
            "-> Tx: '{}', size: {} bytes{}",
            String::from_utf8_lossy(&outgoing).trim(),
            outgoing.len(),
            reset,
        );
        log::info!(
            "<- Rx: '{}', size: {} bytes{}",
            String::from_utf8_lossy(&incoming).trim(),
            incoming.len(),
            reset,
        );
    }

    Ok(())
}
