/*
 *   Copyright (c) 2025 Nazmul Idris
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

//! ```
//! ┌─────────────────┐    ┌──────────────┐    ┌───────────────────┐
//! │   Your Program  │◄──►│ Controller   │    │ Cargo Process     │
//! │                 │    │      ↕       │    │                   │
//! │ Reads/writes    │    │     PTY      │    │ stdin/stdout/     │
//! │ through         │    │      ↕       │    │ stderr redirected │
//! │ controller      │    │   Controlled │◄──►│ to controlled     │
//! └─────────────────┘    └──────────────┘    └───────────────────┘
//! ```

use miette::IntoDiagnostic;
use portable_pty::{MasterPty, PtySize, SlavePty, native_pty_system};
use std::io::Read; // For reading from the PTY controller
use tokio::task::JoinHandle;
// Add import for stripping ANSI/OSC sequences.
use strip_ansi_escapes::strip as strip_ansi; // Provides a convenient function to strip codes.

// ANSI color codes for better readability.
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

// Type aliases for better readability.
type Controlled = Box<dyn SlavePty + Send>;
type Controller = Box<dyn MasterPty>;
type ControlledChild = Box<dyn portable_pty::Child>;

#[tokio::main]
async fn main() -> miette::Result<()> {
    spawn_with_pty().await
}

async fn spawn_with_pty() -> miette::Result<()> {
    // Create a new PTY (pseudo-terminal).
    let native_pty_system = native_pty_system();
    let pty_pair = native_pty_system
        .openpty(PtySize {
            rows: 30,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| miette::miette!("Failed to open PTY: {}", e))?;

    // Extract the endpoints of the PTY.
    let controller: Controller = pty_pair.master;
    let controlled: Controlled = pty_pair.slave;

    // Run cargo clean to make sure the following build is fresh.
    run_cargo_clean()?;

    // [SPAWN 1] Build the command to run via controlled.
    let mut controlled_child = spawn_cargo_build_with_controlled(controlled)?;

    // [SPAWN 2] Read output from the PTY via controller.
    let read_task_handle = spawn_read_task_with_controller(controller);

    // [WAIT 1] Wait for the child process (cargo build) to complete.
    //
    // IMPORTANT: Use spawn_blocking because child.wait() is a synchronous blocking
    // operation. If we called controlled_child.wait() directly in this async context, it
    // would block the entire tokio runtime thread, preventing other async tasks from
    // running. spawn_blocking moves the blocking operation to a dedicated thread pool.
    // The `.into_diagnostic()?.into_diagnostic()?` handles two layers of Result unwrapping:
    // - First `?` unwraps the JoinResult from spawn_blocking (handles task
    //   panics/cancellation)
    // - Second `?` unwraps the Result from child.wait() (handles process wait errors)
    //
    // NOTE: portable-pty doesn't have native async support, but provides async-compatible
    // methods:
    // - try_clone_reader() returns readers that work with tokio::io::AsyncReadExt
    // - child.wait() is blocking, so we use spawn_blocking to avoid blocking the async
    //   runtime
    // - For fully async PTY operations, consider alternatives like tokio-ptyprocess
    //   (experimental)
    let status = tokio::task::spawn_blocking(move || controlled_child.wait())
        .await
        .into_diagnostic()?
        .into_diagnostic()?;

    // [WAIT 2] Wait for the reader task to complete and get the summary report.
    // Pass the exit status to the reader task for better error detection.
    let summary_report = read_task_handle.await.into_diagnostic()??;
    println!("🛬 Read task completed successfully");

    // Print the final report.
    // Check if the build failed based on exit status (most reliable method).
    if !status.success() {
        // The process failed - this is the most reliable indicator
        return Err(miette::miette!(
            "🛬 Command in controlled PTY completed with error: {}\n\nOutput report:\n{}",
            status,
            summary_report
        ));
    }
    println!("🛬 Command in controlled PTY completed successfully");
    println!("{}", summary_report);

    Ok(())
}

/// Spawn the cargo build command with the controlled PTY endpoint, which makes cargo
/// think that it is running in an interactive terminal.
///
/// Set the `TERM_PROGRAM` environment variable to "WezTerm" to ensure that cargo uses the
/// correct terminal capabilities.
///
/// These 2 steps are necessary to ensure that cargo build progress is displayed using OSC
/// codes.
fn spawn_cargo_build_with_controlled(controlled: Controlled) -> miette::Result<ControlledChild> {
    println!("🛫 Spawning cargo build with controlled PTY...");

    // Create a cargo build command.
    let mut cmd = portable_pty::CommandBuilder::new("cargo");
    cmd.arg("build");
    cmd.env("TERM_PROGRAM", "WezTerm");

    // Set the current directory to the project directory. If you don't do this,
    // the PTY starts in the home folder!
    let current_dir = std::env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    cmd.cwd(current_dir);

    // Spawn the command with the controlled PTY endpoint.
    let child = controlled
        .spawn_command(cmd)
        .map_err(|e| miette::miette!("Failed to spawn cargo build: {}", e))?;

    Ok(child)
}

fn spawn_read_task_with_controller(controller: Controller) -> JoinHandle<miette::Result<String>> {
    println!("🛫 Spawning read task with controller...");

    tokio::spawn(async move {
        let mut controller_reader = controller
            .try_clone_reader()
            .map_err(|e| miette::miette!("Failed to clone reader: {}", e))?;

        let mut controlled_child_process_output = String::new();
        controller_reader
            .read_to_string(&mut controlled_child_process_output)
            .map_err(|e| miette::miette!("Failed to read from PTY: {}", e))?;

        // Build the report.
        let mut report = String::new();
        report.push_str("=== Cargo Build Output Snippet ===\n");

        // Include a truncated version of the output.
        let max_lines_to_display = 5;
        let lines: Vec<&str> = controlled_child_process_output.lines().collect();
        let truncated_output = if lines.len() > max_lines_to_display {
            let displayed_lines = lines
                .iter()
                .take(max_lines_to_display)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");
            let raw = format!(
                "{}\n... ({} more lines)",
                displayed_lines,
                lines.len() - max_lines_to_display
            );
            // Strip ANSI / OSC only from the preview snippet so the summary keeps
            // color info in counters while the sample output is safe.
            let bytes: Vec<u8> = strip_ansi(raw.as_bytes());
            String::from_utf8_lossy(&bytes).into_owned()
        } else {
            // Strip here too to keep preview safe.
            let bytes: Vec<u8> = strip_ansi(controlled_child_process_output.as_bytes());
            String::from_utf8_lossy(&bytes).into_owned()
        };
        report.push_str(&truncated_output);

        let has_osc_codes = controlled_child_process_output.contains("\x1b]");

        report.push_str("\n\n=== Summary ===\n");
        report.push_str(&format!(
            "Total bytes read: {}\n",
            controlled_child_process_output.len()
        ));
        report.push_str(&format!(
            "OSC codes found: {}\n",
            if has_osc_codes {
                format!("{}YES ✓{}", GREEN, RESET)
            } else {
                format!("{}NO ✗{}", RED, RESET)
            }
        ));

        // Return report as-is (only the sample portion was sanitized) so summary can
        // retain color decorations.
        Ok(report)
    })
}

fn run_cargo_clean() -> miette::Result<()> {
    println!("Running 'cargo clean' to ensure a fresh build...");

    let status = std::process::Command::new("cargo")
        .arg("clean")
        .arg("-q") // Quiet flag to suppress cargo's output
        .status()
        .map_err(|e| miette::miette!("Failed to run command 'cargo clean': {}", e))?;

    miette::ensure!(
        status.success(),
        "Command 'cargo clean' failed with exit code: {:?}",
        status.code()
    );

    println!("Command 'cargo clean' completed successfully");
    Ok(())
}
