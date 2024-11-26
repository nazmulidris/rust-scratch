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
use r3bl_core::ok;
use tls::net_io;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let addr = format!("{}:{}", net_io::constants::HOST, net_io::constants::PORT);
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
    let stream = tokio::net::TcpStream::connect(addr.as_str())
        .await
        .into_diagnostic()?;

    // Upgrade to secure connection.
    let tls_connector = tls::tls_ops::try_create_client_tls_connect()?;
    let server_name =
        rustls::pki_types::ServerName::try_from(net_io::constants::SERVER_NAME).into_diagnostic()?;
    let secure_stream = tls_connector
        .connect(server_name, stream)
        .await
        .into_diagnostic()?;

    println!(
        "{} {} {}",
        "Connected".green().italic(),
        "securely".green().italic().bold().underlined(),
        "to server".green().italic()
    );

    // Read and write to the server.
    println!(
        "{} {}",
        "Reading from client".green().italic(),
        "and writing to server".green().italic()
    );

    ok!()
}
