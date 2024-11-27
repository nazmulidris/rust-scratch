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

use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use r3bl_core::ok;
use tls::common_io;
use tokio::{io::split, net::TcpListener};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let addr = format!("{}:{}", common_io::constants::HOST, common_io::constants::PORT);
    println!(
        "{} {} {} {}",
        "Starting".yellow().italic(),
        "secure".yellow().italic().bold().underlined(),
        "server at:".yellow().italic(),
        addr.as_str().blue().underlined()
    );

    /*
    Handle SIGINT while waiting for incoming connection, before accepting it. This is a
    background task that will run until the program exits. Even after the connection is
    accepted, this will continue to run and exit the program when Ctrl+C is pressed.
    */
    tokio::spawn(async {
        _ = tokio::signal::ctrl_c().await;
        println!(
            "{} {} {}",
            "Received".red().italic(),
            "SIGINT".red().italic(),
            "while running server".blue()
        );
        std::process::exit(0);
    });

    /*
    Accept insecure connections from clients. `listener.accept()` is a blocking call
    that will wait until a client connects to the server.
    */
    let listener = TcpListener::bind(addr.as_str()).await.into_diagnostic()?;
    let (stream, _) = listener.accept().await.into_diagnostic()?;

    // Upgrade to secure connection.
    let tls_acceptor = tls::tls_ops::try_create_server_tls_acceptor()?;
    let secure_stream = tls_acceptor.accept(stream).await.into_diagnostic()?;
    let (reader, writer) = split(secure_stream);

    println!(
        "{} {} {}",
        "Accepted".green().italic(),
        "secure".green().italic().bold().underlined(),
        "connection from client".green().italic()
    );

    /*
    Read from client and write to client until either:
    - Ctrl+C pressed by user.
    - client side of connection sends EOF or fails.
    */
    common_io::read_write(reader, writer).await?;

    // Close all connections and exit.
    println!(
        "{} {}",
        "Closing".yellow().italic(),
        "all connections".yellow().italic()
    );

    println!(
        "{} {}",
        "Exiting".yellow().italic(),
        "server".yellow().italic()
    );

    ok!()
}
