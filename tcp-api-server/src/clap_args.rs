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
use r3bl_terminal_async::WriterArg;
use std::{fmt::Display, net::IpAddr};

const DEFAULT_PORT_NUM: u16 = 3000;
const DEFAULT_ADDRESS_STR: &str = "127.0.0.1";

#[derive(Parser, Debug)]
#[clap(
    author = "Nazmul Idris <idris@developerlife.com>",
    version,
    about,
    long_about = None,
    after_help = color_print::cstr!(
        "Visit <cyan,bold>https://developerlife.com</> for more detailed information.\n"
    ),
    // To override the entire help_template, here's an example:
    // https://github.com/nazmulidris/cargo/blob/master/src/bin/cargo/cli.rs#L592
    help_template = color_print::cstr!(
        "<green,bold>{bin}</> <yellow,bold>v{version}</> by <cyan,bold>{author}</>\n\
        USAGE:\n  {usage}\n\n\
        OPTIONS:\n{options}\n\n\
        SUBCOMMANDS:\n{subcommands})"
    )
)]
pub struct CLIArg {
    #[arg(
        short = 'a',
        long = "address", // Can't colorize this. Won't match when the user types it in.
        name = color_print::cstr!("Address to <bright-green,bold>connect</> or <bright-red,bold>listen</> to"),
        global = true,
        default_value = DEFAULT_ADDRESS_STR
    )]
    pub address: IpAddr,

    #[arg(
        short = 'p',
        long = "port", // Can't colorize this. Won't match when the user types it in.
        name = color_print::cstr!("Port to <bright-green,bold>connect</> or <bright-red,bold>listen</> to"),
        global = true,
        default_value_t = DEFAULT_PORT_NUM,
    )]
    pub port: u16,

    #[arg(
        short = 't',
        long = "configure-tracing",
        name = color_print::cstr!("Enable tracing via \
            <bright-yellow,bold>none</>, \
            <bright-yellow,bold>stdout</>, \
            <bright-yellow,bold>file</>, \
            <bright-yellow,bold>stdout+file</>"),
        global = true,
        default_values = &["stdout+file"],
        value_delimiter = '+',
    )]
    pub enable_tracing: Vec<WriterArg>,

    #[arg(
        short = 'f',
        long = "tracing-log-file-path-and-prefix",
        name = color_print::cstr!("Set log file <bright-yellow,bold>path and prefix</>"),
        global = true,
        default_value = "tcp_api_server",
    )]
    pub tracing_log_file_path_and_prefix: String,

    #[arg(
        short = 'l',
        long = "tracing-log-level",
        name = color_print::cstr!("Set tracing <bright-yellow,bold>log level</>"),
        global = true,
        default_value = "info",
    )]
    pub tracing_log_level: tracing::Level,

    #[command(subcommand)]
    pub subcommand: CLISubcommand,
}

#[derive(Subcommand, Debug)]
pub enum CLISubcommand {
    #[command(
        name = "server", // Can't colorize this. Won't match when the user types it in.
        short_flag = 's',
        long_about = color_print::cstr!("Start a TCP <bright-red,bold>server</> at the given <bright-cyan,bold>address</> and <bright-cyan,bold>port</>")
    )]
    Server,
    #[command(
        name = "client", // Can't colorize this. Won't match when the user types it in.
        short_flag = 'c',
        about =color_print::cstr!("Start a TCP <bright-green,bold>client</> to connect to the given <bright-cyan,bold>address</> and <bright-cyan,bold>port</>")
    )]
    Client,
}

impl Display for CLISubcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CLISubcommand::Server => write!(f, "server"),
            CLISubcommand::Client => write!(f, "client"),
        }
    }
}
