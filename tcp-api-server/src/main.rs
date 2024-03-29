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
use tcp_api_server::{clap_args, tracing_setup, TerminalAsync, TracingConfig};
use tracing::instrument;

const TCP_API_SERVER: &str = r#"
░░░░░ ░░░░ ░░░░░   ░░░░░ ░░░░░ ░    ░░░░░ ░░░░ ░░░░░  ░░   ░ ░░░░ ░░░░░
  ░   ░    ░   ░   ░   ░ ░   ░ ░    ░     ░    ░   ░  ░░   ░ ░    ░   ░
  ░░  ░░   ░░░░░   ░░░░░ ░░░░░ ░░   ░░░░░ ░░░░ ░░░░░░ ░░  ░░ ░░░░ ░░░░░░
  ░░  ░░   ░░      ░░  ░ ░░    ░░      ░░ ░░   ░░   ░  ░  ░  ░░   ░░   ░
  ░░  ░░░░ ░░      ░░  ░ ░░    ░░   ░░░░░ ░░░░ ░░   ░  ░░░░  ░░░░ ░░   ░
"#;

#[tokio::main]
#[instrument]
async fn main() -> miette::Result<()> {
    let cli_args = clap_args::CLIArg::parse();
    println!("{}", TCP_API_SERVER);

    match cli_args.subcommand {
        // Start server (non interactive, no need for TerminalAsync.
        tcp_api_server::CLISubcommand::Server => {
            // Normal stdout.
            tracing_setup::init(TracingConfig {
                writers: cli_args.enable_tracing.clone(),
                level: cli_args.tracing_log_level,
                tracing_log_file_path_and_prefix: format!(
                    "{}_{}",
                    cli_args.tracing_log_file_path_and_prefix, cli_args.subcommand
                ),
                stdout_override: None,
            })?;
            tcp_api_server::server_task::start_server(cli_args).await?
        }
        // Start client (interactive and needs TerminalAsync).
        tcp_api_server::CLISubcommand::Client => {
            // Async writer for stdout.
            let terminal_async = TerminalAsync::try_new("> ")?;
            tracing_setup::init(TracingConfig {
                writers: cli_args.enable_tracing.clone(),
                level: cli_args.tracing_log_level,
                tracing_log_file_path_and_prefix: format!(
                    "{}_{}",
                    cli_args.tracing_log_file_path_and_prefix, cli_args.subcommand
                ),
                stdout_override: Some(terminal_async.clone_stdout()),
            })?;
            tcp_api_server::client_task::start_client(cli_args, terminal_async).await?
        }
    }

    Ok(())
}
