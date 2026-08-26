// Copyright (c) 2025 R3BL LLC. Licensed under Apache License, Version 2.0.

use rustup_upgrade::install_upgrade_command_with_spinner_and_ctrl_c;
use std::env;

#[tokio::main]
async fn main() {
    let crate_name = env::args()
        .nth(1)
        .unwrap_or_else(|| "r3bl-cmdr".to_string());

    println!("Starting upgrade pipeline for: {crate_name}...");
    match install_upgrade_command_with_spinner_and_ctrl_c(&crate_name).await {
        Ok(status) => {
            if status.success() {
                println!("✅ Successfully upgraded {crate_name}!");
            } else {
                eprintln!("❌ Upgrade exited with non-zero status: {status:?}");
            }
        }
        Err(err) => {
            eprintln!("❌ Upgrade failed: {err}");
        }
    }
}
