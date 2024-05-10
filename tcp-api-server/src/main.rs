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
use miette::IntoDiagnostic;
use opentelemetry::global;
use r3bl_rs_utils_core::UnicodeString;
use r3bl_terminal_async::{tracing_setup, DisplayPreference, TerminalAsync, TracingConfig};
use r3bl_tui::{
    ColorWheel, ColorWheelConfig, ColorWheelSpeed, GradientGenerationPolicy, TextColorizationPolicy,
};
use tcp_api_server::{
    clap_args::{self, CLISubcommand},
    setup_default_miette_global_report_handler,
};
use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const ERROR_REPORT_HANDLER_FOOTER:&str = "If you believe this is a bug, please report it: https://github.com/nazmulidris/rust-scratch/issues";

mod header_banner {
    use super::*;

    const TCP_API_SERVER: &str = r#"
░░░░░ ░░░░ ░░░░░   ░░░░░ ░░░░░ ░    ░░░░░ ░░░░ ░░░░░  ░░   ░ ░░░░ ░░░░░
  ░   ░  ░ ░   ░   ░   ░ ░   ░ ░    ░     ░    ░   ░  ░░   ░ ░    ░   ░
  ░░  ░░   ░░░░░   ░░░░░ ░░░░░ ░░   ░░░░░ ░░░░ ░░░░░░ ░░  ░░ ░░░░ ░░░░░░
  ░░  ░░   ░░      ░░  ░ ░░    ░░      ░░ ░░   ░░   ░  ░  ░  ░░   ░░   ░
  ░░  ░░░░ ░░      ░░  ░ ░░    ░░   ░░░░░ ░░░░ ░░   ░  ░░░░  ░░░░ ░░   ░
"#;

    const TCP_API_CLIENT: &str = r#"
░░░░░ ░░░░ ░░░░░   ░░░░░ ░░░░░ ░    ░░░░ ░     ░  ░░░░ ░░░░░ ░░░░░
  ░   ░  ░ ░   ░   ░   ░ ░   ░ ░    ░  ░ ░     ░  ░    ░   ░   ░
  ░░  ░░   ░░░░░   ░░░░░ ░░░░░ ░░   ░░   ░░    ░░ ░░░░ ░░  ░   ░░
  ░░  ░░   ░░      ░░  ░ ░░    ░░   ░░   ░░    ░░ ░░   ░░  ░   ░░
  ░░  ░░░░ ░░      ░░  ░ ░░    ░░   ░░░░ ░░░░░ ░░ ░░░░ ░░  ░   ░░
"#;

    pub enum Header {
        Server,
        Client,
    }

    /// Gradients: <https://uigradients.com>
    pub fn get_colorful(header: Header) -> String {
        let it = match header {
            Header::Server => TCP_API_SERVER,
            Header::Client => TCP_API_CLIENT,
        };

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
            &UnicodeString::from(it),
            GradientGenerationPolicy::ReuseExistingGradientAndResetIndex,
            TextColorizationPolicy::ColorEachCharacter(None),
        )
    }
}

#[tokio::main]
#[instrument]
async fn main() -> miette::Result<()> {
    let cli_args = clap_args::CLIArg::parse();

    // Show header banner.
    match cli_args.subcommand {
        CLISubcommand::Server => {
            println!(
                "{}",
                header_banner::get_colorful(header_banner::Header::Server)
            );
        }
        CLISubcommand::Client => {
            println!(
                "{}",
                header_banner::get_colorful(header_banner::Header::Client)
            );
        }
    }

    // Setup terminal_async.
    let maybe_terminal_async = match cli_args.subcommand {
        CLISubcommand::Server => None,
        CLISubcommand::Client => TerminalAsync::try_new("> ").await?,
    };

    // Create a tracing config based on whether this is server or client.
    let tracing_config = match &cli_args.subcommand {
        CLISubcommand::Server => TracingConfig {
            writers: cli_args.enable_tracing.clone(),
            level: cli_args.tracing_log_level,
            tracing_log_file_path_and_prefix: format!(
                "{}_{}.log",
                cli_args.tracing_log_file_path_and_prefix, cli_args.subcommand
            ),
            preferred_display: tracing_setup::DisplayPreference::Stdout,
        },
        CLISubcommand::Client => TracingConfig {
            writers: cli_args.enable_tracing.clone(),
            level: cli_args.tracing_log_level,
            tracing_log_file_path_and_prefix: format!(
                "{}_{}.log",
                cli_args.tracing_log_file_path_and_prefix, cli_args.subcommand
            ),
            preferred_display: match &maybe_terminal_async {
                Some(terminal_async) => {
                    let shared_writer = terminal_async.clone_shared_writer();
                    DisplayPreference::SharedWriter(shared_writer)
                }
                None => DisplayPreference::Stdout,
            },
        },
    };

    // Setup tracing with OTel & Jaeger.
    if let Some(layers) = tracing_setup::create_layers(tracing_config)? {
        // Check whether TCP port 14268 is up. Or whether 6831 is up.
        let is_jaeger_up = tokio::net::UdpSocket::bind("127.0.0.1:6831").await.is_err();
        let otel_layer = match is_jaeger_up {
            true => {
                // Allows you to pass along context (i.e., trace IDs) across services set
                // the Global Propagator.
                global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

                // Sets up the machinery needed to export data to Jaeger. There are other
                // OTel crates that provide pipelines for other vendors.
                let tracer = opentelemetry_jaeger::new_pipeline()
                    .with_service_name("tcp-api-server")
                    .install_simple()
                    .into_diagnostic()?;

                // Create a tracing layer with the configured tracer.
                let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

                Some(otel_layer)
            }
            false => None,
        };

        // Initialize the subscriber with the tracing layer.
        tracing_subscriber::registry()
            .with(layers)
            .with(otel_layer)
            .try_init()
            .into_diagnostic()?
    }

    // Setup miette global report handler.
    setup_default_miette_global_report_handler(ERROR_REPORT_HANDLER_FOOTER);

    // Run the server or client.
    match cli_args.subcommand {
        CLISubcommand::Server => {
            tcp_api_server::server_task::server_main_event_loop(cli_args).await?
        }
        CLISubcommand::Client => {
            if let Some(terminal_async) = maybe_terminal_async {
                tcp_api_server::client_task::client_main(cli_args, terminal_async).await?
            }
        }
    }

    Ok(())
}
