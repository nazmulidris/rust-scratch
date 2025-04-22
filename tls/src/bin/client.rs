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

use crossterm::style::Stylize as _;
use miette::IntoDiagnostic;
use r3bl_tui::ok;
use tls::common_io;
use tokio::io::split;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let addr = format!(
        "{}:{}",
        common_io::constants::HOST,
        common_io::constants::PORT
    );
    println!(
        "{} {} {} {}",
        "Starting client to".yellow().italic(),
        "securely".yellow().italic().bold().underlined(),
        "connect to server at:".yellow().italic(),
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
            "while running client".blue()
        );
        std::process::exit(0);
    });

    // Connect to the server insecurely.
    let tcp_stream = tokio::net::TcpStream::connect(addr.as_str())
        .await
        .into_diagnostic()?;

    // Upgrade to secure connection.
    let tls_connector = tls::tls_ops::try_create_client_tls_connector()?;
    let server_name = rustls::pki_types::ServerName::try_from(common_io::constants::SERVER_NAME)
        .into_diagnostic()?;
    let secure_stream = tls_connector
        .connect(server_name, tcp_stream)
        .await
        .into_diagnostic()?;
    let (reader, writer) = split(secure_stream);

    println!(
        "{} {} {}",
        "Connected".green().italic(),
        "securely".green().italic().bold().underlined(),
        "to server".green().italic()
    );

    /*
    Read from client and write to client until either:
    - Ctrl+C pressed by user.
    - client side of connection sends EOF or fails.
    */
    common_io::read_write(reader, writer).await?;

    println!(
        "{} {}",
        "Exiting".yellow().italic(),
        "client".yellow().italic()
    );

    ok!()
}
