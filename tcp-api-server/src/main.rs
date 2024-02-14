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

use clap::Parser;
use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use tcp_api_server::clap_args;
use tracing::info;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let cli_args = clap_args::CLIArg::parse();

    // Enable tracing if the flag is set.
    if cli_args.enable_tracing {
        // Setup tracing. More info: <https://tokio.rs/tokio/topics/tracing>
        tracing::subscriber::set_global_default(
            // More info: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#configuration>
            tracing_subscriber::fmt()
                // .pretty() /* multi line pretty output*/
                .compact() /* single line output */
                .without_time() /* don't display time in output */
                .with_thread_ids(true)
                .with_ansi(true)
                .with_target(false)
                .with_file(false)
                .with_line_number(false)
                .finish(),
        )
        .into_diagnostic()?;
        info!("tracing enabled");
        info!("{}", format!("{:?}", cli_args).cyan().bold());
    };

    // Start the server or client task based on the subcommand.
    match cli_args.subcommand {
        tcp_api_server::CLISubcommand::Server => {
            tcp_api_server::server_task::start_server(cli_args).await?
        }
        tcp_api_server::CLISubcommand::Client => {
            tcp_api_server::client_task::start_client(cli_args).await?
        }
    }

    Ok(())
}
