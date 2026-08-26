// Copyright (c) 2025 R3BL LLC. Licensed under Apache License, Version 2.0.

//! # PTY-Driven Rustup Toolchain Update and Cargo Install with Live Spinner
//!
//! This module provides a reusable, generalized pipeline for running long-running
//! build commands inside a pseudoterminal ([`PTY`]) and streaming real-time progress
//! to an animated [`Spinner`]:
//!
//! 1. **Rustup Nightly Update**: Runs `rustup toolchain install nightly --force` in a PTY,
//!    parsing streaming stdout text (e.g. `downloading component 'rust-std'`) and updating
//!    the spinner message live.
//! 2. **Cargo Nightly Install with OSC Progress**: Runs `cargo +nightly install <crate_name>`
//!    in a PTY with OSC escape sequence capture enabled, parsing OSC 9;4 compilation
//!    percentages (0-100%) and updating the spinner in real time.
//! 3. **Cooperative Ctrl+C Cancellation**: Concurrently monitors for `SIGINT` via
//!    [`tokio::signal::ctrl_c()`], cleanly terminating child processes without corrupting
//!    terminal state.

use r3bl_tui::{
    core::pty::{
        DefaultPtySessionConfig, PtyOutputEvent, PtySessionBuilder, PtySessionConfigOption,
    },
    OscEvent,
    OutputDevice, Spinner, SpinnerStyle, TuiAvailability,
};
use std::{
    io::{Error, ErrorKind},
    process::ExitStatus,
    time::Duration,
};
use tokio::signal;

pub mod constants {
    pub const INFO_PREFIX: &str = "info: ";
    pub const ELLIPSIS: &str = "...";
    pub const MAX_DISPLAY_LEN: usize = 50;
    pub const TRUNCATE_LEN: usize = 47;
}

/// Extracts meaningful progress information from rustup output.
///
/// Looks for patterns starting with `info: `, such as:
/// - `info: syncing channel updates for 'nightly-x86_64-unknown-linux-gnu'`
/// - `info: downloading component 'rust-std'`
/// - `info: installing component 'cargo'`
/// - `info: checking for self-updates`
#[must_use]
pub fn extract_rustup_progress(output: &str) -> String {
    use constants::{ELLIPSIS, INFO_PREFIX, MAX_DISPLAY_LEN, TRUNCATE_LEN};

    let lines: Vec<&str> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    if let Some(last_line) = lines.last() {
        let trimmed = last_line.trim();
        let cleaned = trimmed.strip_prefix(INFO_PREFIX).unwrap_or(trimmed);

        if cleaned.len() > MAX_DISPLAY_LEN {
            format!("{}{ELLIPSIS}", &cleaned[..TRUNCATE_LEN])
        } else {
            cleaned.to_string()
        }
    } else {
        String::new()
    }
}

/// Runs rustup toolchain update inside a PTY session with stdout progress capture.
pub async fn run_rustup_update(spinner: Option<&Spinner>) -> Result<ExitStatus, Error> {
    let mut session = PtySessionBuilder::new("rustup")
        .cli_args(["toolchain", "install", "nightly", "--force"])
        .start()
        .map_err(Error::other)?;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                return Err(Error::new(ErrorKind::Interrupted, "Update cancelled by user"));
            }
            event = session.rx_output_event.recv() => {
                match event {
                    Some(PtyOutputEvent::Output(data)) => {
                        if let Ok(text) = std::str::from_utf8(&data) {
                            let progress_info = extract_rustup_progress(text);
                            if let Some(spinner) = spinner
                                && !progress_info.is_empty()
                            {
                                spinner.update_message(format!("Updating Rust toolchain... {progress_info}"));
                            }
                        }
                    }
                    Some(PtyOutputEvent::Exit(status)) => {
                        return Ok(status.into());
                    }
                    None => {
                        return Err(Error::other("PTY session ended unexpectedly"));
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Runs `cargo +nightly install <crate_name>` inside a PTY session with OSC capture enabled.
pub async fn run_cargo_install_with_progress(
    crate_name: &str,
    spinner: Option<&Spinner>,
) -> Result<ExitStatus, Error> {
    let mut session = PtySessionBuilder::new("cargo")
        .cli_args(["+nightly", "install", crate_name])
        .with_config(DefaultPtySessionConfig + PtySessionConfigOption::CaptureOsc)
        .start()
        .map_err(Error::other)?;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                return Err(Error::new(ErrorKind::Interrupted, "Installation cancelled by user"));
            }
            event = session.rx_output_event.recv() => {
                match event {
                    Some(PtyOutputEvent::Osc(osc)) => {
                        handle_osc_event(osc, crate_name, spinner);
                    }
                    Some(PtyOutputEvent::Exit(status)) => {
                        return Ok(status.into());
                    }
                    None => {
                        return Err(Error::other("PTY session ended unexpectedly"));
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Handles OSC 9;4 compilation progress notifications from Cargo.
pub fn handle_osc_event(event: OscEvent, crate_name: &str, spinner: Option<&Spinner>) {
    if let Some(spinner) = spinner {
        match event {
            OscEvent::ProgressUpdate(percentage) => {
                spinner.update_message(format!("Installing {crate_name}... {percentage}%"));
            }
            OscEvent::IndeterminateProgress => {
                spinner.update_message(format!("Installing {crate_name}... (building)"));
            }
            OscEvent::ProgressCleared => {
                spinner.update_message(format!("Installing {crate_name}..."));
            }
            OscEvent::BuildError => {
                spinner.update_message(format!("Installing {crate_name}... (error occurred)"));
            }
            OscEvent::Hyperlink { .. } | OscEvent::SetTitleAndTab(_) => {}
        }
    }
}

/// Executes the full 2-step upgrade pipeline (rustup update -> cargo install) with live spinner.
pub async fn install_upgrade_command_with_spinner_and_ctrl_c(
    crate_name: &str,
) -> Result<ExitStatus, Error> {
    let res_spinner = Spinner::try_start(
        "Updating Rust toolchain...",
        "Installation ended.",
        Duration::from_millis(100),
        SpinnerStyle::default(),
        OutputDevice::default(),
        None,
    )
    .await;

    let mut maybe_spinner = match res_spinner {
        TuiAvailability::Available(spinner) => Some(spinner),
        _ => None,
    };

    // Step 1: Rustup toolchain update
    let rustup_result = run_rustup_update(maybe_spinner.as_ref()).await;
    if let Err(e) = rustup_result {
        if let Some(mut spinner) = maybe_spinner.take() {
            spinner.request_shutdown();
            spinner.await_shutdown().await;
        }
        return Err(e);
    }

    // Step 2: Update spinner message for cargo install
    if let Some(ref spinner) = maybe_spinner {
        spinner.update_message(format!("Installing {crate_name}..."));
    }

    // Step 3: Cargo install with OSC progress tracking
    let install_result =
        run_cargo_install_with_progress(crate_name, maybe_spinner.as_ref()).await;

    // Shutdown spinner
    if let Some(mut spinner) = maybe_spinner.take() {
        spinner.request_shutdown();
        spinner.await_shutdown().await;
    }

    install_result
}
