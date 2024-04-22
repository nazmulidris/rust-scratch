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
use r3bl_rs_utils_core::UnicodeString;
use r3bl_terminal_async::{tracing_setup, TerminalAsync, TracingConfig};
use r3bl_tui::{
    ColorWheel, ColorWheelConfig, ColorWheelSpeed, GradientGenerationPolicy, TextColorizationPolicy,
};
use tcp_api_server::clap_args;
use tracing::instrument;

/// Gradients: <https://uigradients.com>
fn colorize_header() -> String {
    const TCP_API_SERVER: &str = r#"
░░░░░ ░░░░ ░░░░░   ░░░░░ ░░░░░ ░    ░░░░░ ░░░░ ░░░░░  ░░   ░ ░░░░ ░░░░░
  ░   ░    ░   ░   ░   ░ ░   ░ ░    ░     ░    ░   ░  ░░   ░ ░    ░   ░
  ░░  ░░   ░░░░░   ░░░░░ ░░░░░ ░░   ░░░░░ ░░░░ ░░░░░░ ░░  ░░ ░░░░ ░░░░░░
  ░░  ░░   ░░      ░░  ░ ░░    ░░      ░░ ░░   ░░   ░  ░  ░  ░░   ░░   ░
  ░░  ░░░░ ░░      ░░  ░ ░░    ░░   ░░░░░ ░░░░ ░░   ░  ░░░░  ░░░░ ░░   ░
"#;

    let color_wheel_config = ColorWheelConfig::Rgb(
        // Stops.
        vec!["#4e54c8", "#9d459a"]
            .into_iter()
            .map(String::from)
            .collect(),
        // Speed.
        ColorWheelSpeed::Medium,
        // Steps.
        50,
    );

    ColorWheel::new(vec![color_wheel_config]).colorize_into_string(
        &UnicodeString::from(TCP_API_SERVER),
        GradientGenerationPolicy::ReuseExistingGradientAndResetIndex,
        TextColorizationPolicy::ColorEachCharacter(None),
    )
}

#[tokio::main]
#[instrument]
async fn main() -> miette::Result<()> {
    let cli_args = clap_args::CLIArg::parse();
    println!("{}", colorize_header());

    match cli_args.subcommand {
        // Start server (non interactive, no need for TerminalAsync. Normal stdout.
        tcp_api_server::CLISubcommand::Server => {
            tracing_setup::init(TracingConfig {
                writers: cli_args.enable_tracing.clone(),
                level: cli_args.tracing_log_level,
                tracing_log_file_path_and_prefix: format!(
                    "{}_{}",
                    cli_args.tracing_log_file_path_and_prefix, cli_args.subcommand
                ),
                stdout_override: None,
            })?;
            tcp_api_server::server_task::server_main(cli_args).await?
        }
        // Start client (interactive and needs TerminalAsync). Async writer for stdout.
        tcp_api_server::CLISubcommand::Client => {
            let maybe_terminal_async = TerminalAsync::try_new("> ").await?;

            tracing_setup::init(TracingConfig {
                writers: cli_args.enable_tracing.clone(),
                level: cli_args.tracing_log_level,
                tracing_log_file_path_and_prefix: format!(
                    "{}_{}",
                    cli_args.tracing_log_file_path_and_prefix, cli_args.subcommand
                ),
                stdout_override: maybe_terminal_async
                    .as_ref()
                    .map(|terminal_async| terminal_async.clone_shared_writer()),
            })?;

            if let Some(terminal_async) = maybe_terminal_async {
                tcp_api_server::client_task::client_main(cli_args, terminal_async).await?
            }
        }
    }

    Ok(())
}
