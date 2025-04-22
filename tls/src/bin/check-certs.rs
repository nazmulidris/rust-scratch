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
use r3bl_tui::ok;
use tls::{
    certificate_ops::server_load_server_cert_chain, key_ops::server_load_single_private_key,
    root_cert_store_ops::client_create_root_cert_store,
};

fn main() -> miette::Result<()> {
    // Load the server keys (from server-key.pem).
    let server_key = server_load_single_private_key()?;
    println!("{}: {:?}", "Server key".blue().underlined(), server_key);

    // Load the server certificate (from server.pem).
    let server_cert_chain = server_load_server_cert_chain()?;
    println!(
        "{}: {:?}",
        "Server certificate".blue().underlined(),
        server_cert_chain
    );

    // Create the root certificate store.
    let root_store = client_create_root_cert_store()?;
    println!(
        "{}: {:?}",
        "Root certificate store".blue().underlined(),
        root_store
    );

    ok!()
}
