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

use r3bl_tui::set_jemalloc_in_main;
use clap::Parser;
use miette::IntoDiagnostic;
use r3bl_tui::{
    setup_default_miette_global_report_handler, ColorWheel, ColorWheelConfig, ColorWheelSpeed,
    DisplayPreference, GradientGenerationPolicy, TextColorizationPolicy, TracingConfig,
};
use r3bl_tui::{try_create_layers, ReadlineAsync};
use tcp_api_server::{
    clap_args::{self, CLISubcommand},
    convert_args_into_writer_config, jaeger_setup,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const ERROR_REPORT_HANDLER_FOOTER:&str = "If you believe this is a bug, please report it: https://github.com/nazmulidris/rust-scratch/issues";

mod header_banner {
    use r3bl_tui::InlineString;
    use smallvec::smallvec;

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
    pub fn get_colorful(header: Header) -> InlineString {
        let it = match header {
            Header::Server => TCP_API_SERVER,
            Header::Client => TCP_API_CLIENT,
        };

        let color_wheel_config = ColorWheelConfig::Rgb(
            // Stops.
            smallvec!["#4e54c8".into(), "#9d459a".into()],
            // Speed.
            ColorWheelSpeed::Medium,
            // Steps.
            50,
        );

        ColorWheel::new(smallvec![color_wheel_config]).colorize_into_string(
            it,
            GradientGenerationPolicy::ReuseExistingGradientAndResetIndex,
            TextColorizationPolicy::ColorEachCharacter(None),
            None,
        )
    }
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    set_jemalloc_in_main!();    
    
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

    // Setup readline_async.
    let maybe_readline_async = match cli_args.subcommand {
        CLISubcommand::Server => None,
        CLISubcommand::Client => ReadlineAsync::try_new(Some("ⴾ "))?,
    };

    // Create a tracing config based on whether this is server or client.
    let tracing_config = match &cli_args.subcommand {
        CLISubcommand::Server => {
            let level_filter: LevelFilter = cli_args.tracing_log_level.into();
            let file_path_and_prefix = format!(
                "{}_{}.log",
                cli_args.tracing_log_file_path_and_prefix, cli_args.subcommand
            );

            TracingConfig {
                writer_config: convert_args_into_writer_config(
                    &cli_args.enable_tracing,
                    file_path_and_prefix,
                    DisplayPreference::Stdout,
                ),
                level_filter,
            }
        }
        CLISubcommand::Client => {
            let level_filter: LevelFilter = cli_args.tracing_log_level.into();
            let file_path_and_prefix = format!(
                "{}_{}.log",
                cli_args.tracing_log_file_path_and_prefix, cli_args.subcommand
            );
            let display_preference = match &maybe_readline_async {
                Some(readline_async) => {
                    let shared_writer = readline_async.clone_shared_writer();
                    DisplayPreference::SharedWriter(shared_writer)
                }
                None => DisplayPreference::Stdout,
            };
            TracingConfig {
                writer_config: convert_args_into_writer_config(
                    &cli_args.enable_tracing,
                    file_path_and_prefix,
                    display_preference,
                ),
                level_filter,
            }
        }
    };

    let service_name = match cli_args.subcommand {
        CLISubcommand::Server => "server",
        CLISubcommand::Client => "client",
    };

    // Setup tracing with OTel & Jaeger. Create a variable to hold the drop handle, so
    // that it can be dropped at the end of the program, and the tracer can be shutdown.
    // Don't assign this to `_` because it will be dropped immediately.
    let mut _maybe_drop_tracer = None;
    if let Some(mut tracing_layers) = try_create_layers(tracing_config)? {
        if let Some((otel_layer, drop_tracer)) =
            jaeger_setup::try_get_otel_layer(service_name, Some(cli_args.otel_collector_endpoint))
                .await?
        {
            tracing_layers.push(Box::new(otel_layer));
            _maybe_drop_tracer.replace(drop_tracer);
        }

        // Initialize the subscriber with the tracing layer.
        tracing_subscriber::registry()
            .with(tracing_layers)
            .try_init()
            .into_diagnostic()?;
    }

    // Setup miette global report handler.
    setup_default_miette_global_report_handler(ERROR_REPORT_HANDLER_FOOTER);

    // Run the server or client.
    match cli_args.subcommand {
        CLISubcommand::Server => tcp_api_server::server_task::server_entry_point(cli_args).await?,
        CLISubcommand::Client => {
            if let Some(readline_async) = maybe_readline_async {
                tcp_api_server::client_task::client_entry_point(cli_args, readline_async).await?
            }
        }
    }

    Ok(())
}
