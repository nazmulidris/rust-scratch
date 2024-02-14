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

use std::net::IpAddr;

use clap::{Parser, Subcommand};

const DEFAULT_PORT_NUM: u16 = 3000;
const DEFAULT_ADDRESS_STR: &str = "127.0.0.1";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CLIArg {
    /// Address to connect or listen to
    #[clap(global = true, default_value = DEFAULT_ADDRESS_STR)]
    #[arg(short, long)]
    pub address: IpAddr,

    /// Port to connect or listen to
    #[clap(global = true, default_value_t = DEFAULT_PORT_NUM)]
    #[arg(short, long)]
    pub port: u16,

    /// Enable tracing
    #[clap(global = true, default_value_t = false)]
    #[arg(short = 't', long = "enable-tracing")]
    pub enable_tracing: bool,

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
