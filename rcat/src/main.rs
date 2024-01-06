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
use core::net::{IpAddr, SocketAddr};
use crossterm::style::Stylize;
use std::{
    io::{stdin, Read, Write},
    net::{TcpListener, TcpStream},
};

type IOResult = std::io::Result<()>;

const DEFAULT_BUFFER_SIZE: usize = 1024;
const DEFAULT_PORT_NUM: u16 = 3000;
const DEFAULT_ADDRESS_STR: &str = "127.0.0.1";

pub mod clap_config {
    use super::*;

    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub struct Arg {
        /// Address to connect or listen to
        #[clap(global = true, default_value = DEFAULT_ADDRESS_STR)]
        #[arg(short, long)]
        pub address: IpAddr,

        /// Port to connect or listen to
        #[clap(global = true, default_value_t = DEFAULT_PORT_NUM)]
        #[arg(short, long)]
        pub port: u16,

        #[command(subcommand)]
        pub subcommand: CLISubcommand,
    }

    #[derive(Subcommand, Debug)]
    pub enum CLISubcommand {
        /// Start a TCP server at the given address and port
        Server,
        /// Start a TCP client to connect to the given address and port
        Client,
    }
}

fn main() -> IOResult {
    println!("{}", format!("{}", "Welcome to rcat!").yellow());

    let cli_arg = clap_config::Arg::parse();
    let address = cli_arg.address;
    let port = cli_arg.port;

    let result = match cli_arg.subcommand {
        clap_config::CLISubcommand::Client => client::try_connect_to(address, port),
        clap_config::CLISubcommand::Server => server::try_listen_on(address, port),
    };

    match result {
        Ok(_) => println!("{}", "Done!".magenta().to_string()),
        Err(error) => eprintln!(
            "{}",
            format!(
                "There was a problem running the subcommand: {:?}: {}",
                cli_arg.subcommand, error
            )
            .red()
            .to_string()
        ),
    }

    Ok(())
}

pub mod client {
    use std::io::stdout;

    use super::*;

    pub fn try_connect_to(address: IpAddr, port: u16) -> IOResult {
        println!(
            "{}",
            format!("[Connecting to {}:{}]", address, port)
                .blue()
                .to_string()
        );

        let socket_address = SocketAddr::new(address, port);
        let mut stream = TcpStream::connect(&socket_address)?;

        loop {
            // Display prompt to stdout.
            print!(
                "{}",
                format!("(Type something. Press enter to send) > ")
                    .green()
                    .to_string()
            );
            let _ = stdout().flush(); // Without this line, the print! does not display.

            // Read user input.
            let user_input = {
                let mut it = String::new();
                let _num_bytes_read_from_stdin = stdin().read_line(&mut it)?;
                it.trim().as_bytes().to_vec()
            };

            // Send the user input over the stream.
            let num_bytes_written = stream.write(&user_input)?;
            println!(
                "{}",
                format!(
                    ">>> '{}', size: {} bytes",
                    String::from_utf8_lossy(&user_input).white().on_dark_grey(),
                    num_bytes_written,
                )
                .magenta()
                .to_string()
            );

            // Read the response from the server.
            let response_from_server = {
                let mut it = [0; DEFAULT_BUFFER_SIZE];
                let num_bytes_read_from_stream = stream.read(&mut it)?;
                it[..num_bytes_read_from_stream].to_vec()
            };
            println!(
                "{}",
                format!(
                    "<<< '{}', size: {} bytes",
                    String::from_utf8_lossy(&response_from_server)
                        .yellow()
                        .on_dark_grey(),
                    response_from_server.len()
                )
                .magenta()
                .to_string()
            );
        }
    }
}

/// Both `accept()` and `bind()` are functions used in socket programming, but they serve
/// different purposes:
///
/// **bind():**
///
/// * **Associates a socket with a specific address and port.** This essentially tells the
///   operating system where you want to listen for incoming connections. Think of it as
///   "planting a flag" on a specific port number.
/// * **Usually called before a server starts listening for connections.** It reserves the
///   chosen port for your application, preventing other programs from binding to it
///   simultaneously.
/// * **Non-blocking in most cases.** It quickly assigns the address and port to the
///   socket unless there's an issue like port already being in use or system overload.
///
/// **accept():**
///
/// * **Waits for an incoming connection on a listening socket.** It blocks the program's
///   execution until a client attempts to connect to the server's address and port.
/// * **Used only after a server has called `listen()` on the socket.** Listening puts the
///   socket in a passive mode, ready to accept incoming connections.
/// * **Returns a new socket object for the established connection.** This allows you to
///   communicate with the client separately from other connections.
/// * **Blocking operation.** It remains suspended until a connection arrives, effectively
///   pausing the program's execution.
///
/// **Here's an analogy:**
///
/// * Imagine `bind()` like setting up a shop with a specific address and storefront
///   (port).
/// * Now, `accept()` would be like a customer entering the shop. You wouldn't know
///   they're coming until they actually walk in, so you'd wait (be blocked) until they
///   did. Once they enter, you can interact with them (use the new socket) like any other
///   customer.
///
/// **In summary:**
///
/// * `bind()` prepares the socket for incoming connections by specifying its location.
/// * `accept()` receives incoming connections on a prepared socket and establishes
///   communication channels.
///
/// From: <https://g.co/bard/share/41f97be3e0e9>
pub mod server {
    use super::*;

    /// **Breakdown:**
    ///
    /// 1. **Binding with `bind()`:**
    ///    - `TcpListener::bind("127.0.0.1:8080").unwrap()` creates a listener and binds
    ///      it to the specified address (localhost) and port (8080).
    ///
    /// 2. **Accepting with `accept()`:**
    ///    - `listener.incoming()` returns an iterator that yields `Result<TcpStream,
    ///      std::io::Error>`.
    ///    - The `for` loop iterates over incoming connections.
    ///    - `match stream { Ok(stream) => ... }` handles successful connections.
    ///    - The `accept()` call is implicit within `listener.incoming()`.
    ///
    /// 3. **Handling Connections:**
    ///    - `handle_connection_blocking(stream)` is called for each accepted connection to manage
    ///      communication with the client.
    ///
    /// **Remember:**
    ///
    /// - This example uses Rust's `std::net` library for synchronous networking.
    /// - For asynchronous networking, you'd use crates like Tokio with their own `bind()`
    ///   and `accept()` methods within an asynchronous context.
    ///
    /// From: <https://g.co/bard/share/7c67bbab3bc2>
    pub fn try_listen_on(address: IpAddr, port: u16) -> IOResult {
        println!(
            "{}",
            format!("[Listening on {}:{}]", address, port)
                .blue()
                .to_string()
        );

        // Create a TCP listener.
        let tcp_listener = TcpListener::bind((address, port))?;

        println!(
            "{}",
            "try_listen_on -> iter loop start".green().on_dark_grey()
        );

        // Server loop.
        // 1. This block never returns unless there's an error. It just infinitely waits
        //    around (in a single thread) waiting for an incoming connection, and then
        //    processes that connection in that single thread.
        // 2. The `accept()` blocking call is implicit within the iterator's `next()`
        //    implementation.
        let incoming_iterator = tcp_listener.incoming();
        for result_stream in incoming_iterator {
            println!(
                "{}",
                "try_listen_on -> iter loop next".green().on_dark_grey()
            );
            let stream = &mut result_stream?;

            println!(
                "{}",
                "handle_connection_blocking -> start"
                    .black()
                    .on_dark_green()
            );

            let _ = handle_connection_blocking(stream)?;

            println!(
                "{}",
                "handle_connection_blocking -> end".red().on_dark_green()
            );
        }

        // THE FOLLOWING CODE NEVER EXECUTES!
        // println!("{}", "try_listen_on -> iter loop end".black().on_white());
        Ok(())
    }

    /// Handle communication w/ the client. This function is a blocking infinite loop.
    /// - For more information on doing the read using different approaches, read this:
    ///   <https://g.co/bard/share/0382f2b4f4ac>
    fn handle_connection_blocking(stream: &mut TcpStream) -> IOResult {
        loop {
            // Read bytes.
            let bytes_read = {
                let mut buffer = [0; DEFAULT_BUFFER_SIZE];
                let num_bytes_read = stream.read(&mut buffer)?;
                buffer[..num_bytes_read].to_vec()
            };

            // EOF.
            if bytes_read.len() == 0 {
                println!(
                    "{}",
                    format!("End of connection")
                        .red()
                        .on_dark_green()
                        .to_string()
                );
                return Ok(());
            }

            // Generate a response.
            let response = process_bytes(&bytes_read);

            // Write bytes & send the response to the client.
            let num_bytes_written = stream.write(&response)?;

            println!(
                "{}",
                format!(
                    ">>> Server received: '{}', size: {} bytes\n<<< sent response: '{}', size: {} bytes]",
                    String::from_utf8_lossy(&bytes_read).white().on_dark_grey(),
                    bytes_read.len(),
                    String::from_utf8_lossy(&response).yellow().on_dark_grey(),
                    num_bytes_written
                )
                .blue()
                .to_string()
            );
        }
    }

    fn process_bytes(incoming: &[u8]) -> Vec<u8> {
        let buffer_string = String::from_utf8_lossy(&incoming);
        let outgoing = format!("echo: {}", buffer_string);
        outgoing.as_bytes().to_vec()
    }
}
