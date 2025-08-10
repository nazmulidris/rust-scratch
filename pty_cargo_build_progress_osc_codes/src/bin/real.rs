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

//! Binary for capturing and displaying OSC progress sequences from cargo builds.
//!
//! This program demonstrates how to capture OSC (Operating System Command) sequences
//! emitted by cargo when running in a terminal that supports progress reporting.
//! It uses a pseudo-terminal (PTY) to make cargo think it's running in an interactive
//! terminal, which triggers the emission of OSC 9;4 progress sequences.
//!
//! # OSC Sequence Format
//!
//! Cargo emits OSC sequences in the format: `ESC]9;4;{state};{progress}ESC\\`
//!
//! Where:
//! - `state` 0: Clear/remove progress
//! - `state` 1: Set specific progress (0-100%)
//! - `state` 2: Build error occurred
//! - `state` 3: Indeterminate progress
//!
//! # Usage
//!
//! Run this binary to see cargo build progress in real-time:
//! ```bash
//! cargo run --bin real
//! ```

use miette::IntoDiagnostic;
use portable_pty::{CommandBuilder, MasterPty, PtySize, SlavePty, native_pty_system};
use std::{io::Read, path::PathBuf, pin::Pin};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

// ANSI color codes for better readability.
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

// Buffer size for reading PTY output.
const READ_BUFFER_SIZE: usize = 4096;

// Type aliases for better readability.
type Controlled = Box<dyn SlavePty + Send>;
type Controller = Box<dyn MasterPty>;
type ControlledChild = Box<dyn portable_pty::Child>;

/// Represents the different types of OSC progress events
/// that Cargo can emit.
#[derive(Debug, Clone, PartialEq)]
enum OscEvent {
    /// Set specific progress value 0-100% (OSC state 1).
    ProgressUpdate(u8),
    /// Clear/remove progress indicator (OSC state 0).
    ProgressCleared,
    /// Build error occurred (OSC state 2).
    BuildError,
    /// Indeterminate progress - build is running but no
    /// specific progress (OSC state 3).
    IndeterminateProgress,
}

/// OSC 9;4 sequence constants wrapped in a dedicated module for clarity.
mod osc {
    /// Sequence prefix: ESC ] 9 ; 4 ;
    pub const START: &str = "\x1b]9;4;";
    /// Sequence terminator: ESC \\ (String Terminator)
    pub const END: &str = "\x1b\\";
}

/// Buffer for accumulating and parsing OSC (Operating System Command) sequences.
///
/// This is not the raw PTY read buffer, but a dedicated buffer that accumulates
/// OSC sequences as they are read from the PTY output. It handles partial sequences
/// that may be split across multiple read operations.
struct OscBuffer {
    data: String,
}

impl OscBuffer {
    /// Creates a new empty OSC buffer.
    fn new() -> Self {
        Self {
            data: String::new(),
        }
    }

    /// Appends new bytes to the buffer and extracts any complete OSC sequences.
    ///
    /// # Arguments
    /// * `buffer` - Raw bytes read from the PTY
    /// * `n` - Number of valid bytes in the buffer
    ///
    /// # Returns
    /// A vector of parsed `OscEvent`s from any complete sequences found
    fn append_and_extract(&mut self, buffer: &[u8], n: usize) -> Vec<OscEvent> {
        // Convert bytes to string and append to accumulated data
        let text = String::from_utf8_lossy(&buffer[..n]);
        self.data.push_str(&text);

        let mut events = Vec::new();

        // Find and process all complete OSC sequences
        while let Some(event) = self.extract_next_sequence() {
            events.push(event);
        }

        events
    }

    /// Extracts and parses the next complete OSC sequence from the buffer.
    ///
    /// Looks for sequences in the format: `ESC]9;4;{state};{progress}ESC\`
    ///
    /// # Returns
    /// * `Some(OscEvent)` if a complete sequence was found and parsed
    /// * `None` if no complete sequence is available
    fn extract_next_sequence(&mut self) -> Option<OscEvent> {
        // OSC sequence format: osc::START {state};{progress} osc::END
        // Find start of OSC sequence
        let start_idx = self.data.find(osc::START)?;
        let after_start_idx = start_idx + osc::START.len();

        // Find end of sequence
        let end_idx = self.data[after_start_idx..].find(osc::END)?;
        let params_end_idx = after_start_idx + end_idx;
        let sequence_end_idx = params_end_idx + osc::END.len();

        // Extract parameters
        let params = &self.data[after_start_idx..params_end_idx];

        // Parse the sequence
        let event = self.parse_osc_params(params);

        // Remove processed portion from buffer (including
        // everything up to sequence end)
        self.data.drain(0..sequence_end_idx);

        event
    }

    /// Parses OSC parameters into an `OscEvent`.
    ///
    /// # Arguments
    /// * `params` - The parameter string in format "{state};{progress}"
    ///
    /// # Returns
    /// * `Some(OscEvent)` if parameters were valid
    /// * `None` if parameters were malformed or state was unknown
    fn parse_osc_params(&self, params: &str) -> Option<OscEvent> {
        let parts: Vec<&str> = params.split(';').collect();
        if parts.len() != 2 {
            // Gracefully handle malformed sequences
            return None;
        }

        let state = parts[0].parse::<u8>().ok()?;
        let progress = parts[1].parse::<f64>().ok()?;

        match state {
            0 => Some(OscEvent::ProgressCleared),
            1 => Some(OscEvent::ProgressUpdate(progress as u8)),
            2 => Some(OscEvent::BuildError),
            3 => Some(OscEvent::IndeterminateProgress),
            _ => None, // Gracefully ignore unknown states
        }
    }
}

/// Configuration builder for PTY commands with sensible defaults.
///
/// This builder ensures critical settings are not forgotten when creating PTY commands:
/// - Automatically sets the current working directory if not specified
/// - Provides methods for common terminal environment variables
/// - Ensures commands spawn in the correct context (not in `$HOME`)
///
/// # Examples
///
/// Basic cargo command with OSC sequences:
/// ```
/// let cmd = PtyCommandBuilder::new("cargo")
///     .args(["build", "--release"])
///     .enable_osc_sequences()
///     .build()?;
/// ```
///
/// Command with custom working directory:
/// ```
/// let cmd = PtyCommandBuilder::new("npm")
///     .args(["install"])
///     .cwd("/path/to/project")
///     .env("NODE_ENV", "production")
///     .build()?;
/// ```
struct PtyCommandBuilder {
    command: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
}

impl PtyCommandBuilder {
    /// Creates a new PTY command builder for the specified command.
    fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            cwd: None,
            env_vars: Vec::new(),
        }
    }

    /// Adds arguments to the command.
    fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Sets the working directory.
    ///
    /// If not called, defaults to the current directory when `build()` is invoked.
    fn cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    /// Adds an environment variable to the command's environment.
    fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Enables OSC sequence emission by setting appropriate environment variables.
    ///
    /// Cargo requires specific terminal environment variables to emit OSC 9;4 progress
    /// sequences. This method automatically detects and configures the appropriate
    /// environment based on the current terminal:
    ///
    /// - **Windows Terminal**: Detected via `WT_SESSION` (no additional config needed)
    /// - **ConEmu**: Detected via `ConEmuANSI=ON` (no additional config needed)
    /// - **WezTerm**: Set via `TERM_PROGRAM=WezTerm` (fallback for all platforms)
    ///
    /// This approach ensures maximum compatibility across different terminals and
    /// operating systems, particularly on Windows where Windows Terminal is the
    /// default in Windows 11.
    ///
    /// Here is a link to the Cargo source code that emits these sequences:
    /// - <https://github.com/rust-lang/cargo/blob/master/src/cargo/core/shell.rs#L594-L600>
    fn enable_osc_sequences(self) -> Self {
        // Windows Terminal sets WT_SESSION automatically, so we don't need to override it.
        if std::env::var("WT_SESSION").is_ok() {
            // Already in Windows Terminal, no need to set anything.
            self
        } else if std::env::var("ConEmuANSI").ok() == Some("ON".into()) {
            // Already in ConEmu with ANSI enabled.
            self
        } else {
            // Fall back to WezTerm which works on all platforms.
            self.env("TERM_PROGRAM", "WezTerm")
        }
    }

    /// Builds the final `CommandBuilder` with all configurations applied.
    ///
    /// Always sets a working directory - uses the provided one or defaults to current
    /// directory. This is critical to ensure the PTY starts in the expected location,
    /// since by default it uses `$HOME`.
    ///
    /// # Returns
    /// * `Ok(CommandBuilder)` - Configured command ready for PTY execution
    /// * `Err(miette::Error)` - If current directory cannot be determined
    fn build(mut self) -> miette::Result<CommandBuilder> {
        // Ensure working directory is always set - use current if not specified.
        // This prevents PTY from spawning in an unexpected location.
        if self.cwd.is_none() {
            let current_dir = std::env::current_dir()
                .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
            self = self.cwd(current_dir);
        }

        let mut cmd = CommandBuilder::new(&self.command);

        // Add all arguments.
        for arg in &self.args {
            cmd.arg(arg);
        }

        // Set the working directory (guaranteed to be Some at this point).
        cmd.cwd(self.cwd.unwrap());

        // Apply all environment variables.
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        Ok(cmd)
    }
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    /// Runs cargo clean to ensure a fresh build.
    fn run_cargo_clean() -> miette::Result<()> {
        println!(
            "{}🧹 Running 'cargo clean' to ensure a fresh build...{}",
            YELLOW, RESET
        );

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

        println!("{}✓ Cargo clean completed successfully{}\n", GREEN, RESET);
        Ok(())
    }

    /// Runs a single cargo build with OSC capture.
    async fn run_build_with_osc_capture() -> miette::Result<()> {
        println!("{}========================================", YELLOW);
        println!("{}Starting Cargo build with OSC capture...", YELLOW);
        println!(
            "{}========================================{}",
            YELLOW, RESET
        );

        // Create channel for OSC events.
        let (sender, mut receiver) = unbounded_channel::<OscEvent>();

        // Spawn the cargo build task - it's already pinned and spawned internally.
        let cargo_args = &["build"];
        let mut pinned_join_handle_cargo_task = spawn_cargo_task_with_osc_capture(
            /* move */ cargo_args.iter().map(ToString::to_string).collect(),
            /* move */ sender,
        );

        // Handle events as they arrive until cargo completes.
        let exit_status = loop {
            tokio::select! {
                // Handle cargo build completion - this takes priority.
                build_result = &mut pinned_join_handle_cargo_task => {
                    match build_result {
                        Ok(status_result) => {
                            let status = status_result?;
                            // Exit the loop and return the status.
                            break Some(status);
                        }
                        Err(e) => {
                            return Err(miette::miette!("Cargo task failed: {}", e));
                        }
                    }
                }
                // Handle incoming OSC events. Don't break on these - let the cargo task
                // completion handle the exit.
                Some(event) = receiver.recv() => {
                    match event {
                        OscEvent::ProgressUpdate(percentage) => {
                            println!(
                                "{}📊 cargo {} progress: {}%{}",
                                GREEN,
                                cargo_args.join(" "),
                                percentage,
                                RESET
                            );
                        }
                        OscEvent::ProgressCleared => {
                            println!("{}✓ Progress tracking cleared{}", GREEN, RESET);
                        }
                        OscEvent::BuildError => {
                            println!("{}❌ Build error occurred{}", RED, RESET);
                        }
                        OscEvent::IndeterminateProgress => {
                            println!("{}⏳ Build in progress (indeterminate){}", GREEN, RESET);
                        }
                    }
                }
            }
        };

        // Print final status.
        println!(
            "{}✅ Build completed with status: {:?}{}",
            GREEN, exit_status, RESET
        );

        Ok(())
    }

    // Run twice to demonstrate both scenarios:
    // 1. First run with cargo clean - should show progress
    // 2. Second run without clean - should not show progress (everything cached)

    println!(
        "\n{}╔══════════════════════════════════════════════════════╗",
        YELLOW
    );
    println!(
        "{}║  RUN 1: With cargo clean (expect progress updates)  ║",
        YELLOW
    );
    println!(
        "{}╚══════════════════════════════════════════════════════╝{}\n",
        YELLOW, RESET
    );

    // Clean the build to ensure we have work to do
    run_cargo_clean()?;

    // Run the first build - should show progress
    run_build_with_osc_capture().await?;

    println!(
        "\n{}╔══════════════════════════════════════════════════════╗",
        YELLOW
    );
    println!(
        "{}║  RUN 2: Without clean (expect no progress updates)  ║",
        YELLOW
    );
    println!(
        "{}╚══════════════════════════════════════════════════════╝{}\n",
        YELLOW, RESET
    );

    // Run the second build - should not show progress (everything cached)
    run_build_with_osc_capture().await?;

    Ok(())
}

/// Spawns a cargo command in a PTY and captures OSC progress sequences.
///
/// It is typically used to run cargo commands like:
/// 1. `cargo build`
/// 2. `cargo install`
///
/// Note that other programs like `rustup` do not emit OSC sequences.
///
/// This function spawns itself in a separate task and returns a pinned `JoinHandle` that
/// the caller can await. This design:
/// 1. **Simplifies usage**: No need for manual `tokio::spawn` and pinning at call site
/// 2. **Blocking operations**: Uses `spawn_blocking` for PTY I/O operations internally
/// 3. **Concurrent execution**: Automatically runs in parallel with event handling
/// 4. **Resource management**: Ensures proper cleanup of PTY resources
///
/// # Arguments
/// * `cargo_args` - Arguments to pass to the cargo command
/// * `event_sender` - Channel sender for OSC events
///
/// # Returns
/// A pinned `JoinHandle` that resolves to the cargo process exit status.
///
/// The `JoinHandle` is pinned to the heap using `Box::pin` rather than the stack-based
/// `pin!` macro because this function returns the pinned value. Stack pinning with `pin!`
/// would create a value that cannot outlive the function's stack frame, making it
/// impossible to return. Heap pinning ensures the pinned future remains valid after the
/// function returns.
///
/// The pinning is necessary because `JoinHandle` does not implement `Unpin`, and
/// `tokio::select!` requires futures to be `Unpin`. By returning it pre-pinned, callers
/// can use it directly in `tokio::select!` blocks without additional pinning.
///
/// # Environment
/// Sets `TERM_PROGRAM=WezTerm` to trigger OSC sequence emission from cargo.
fn spawn_cargo_task_with_osc_capture(
    /* move */ cargo_args: Vec<String>,
    /* move */ event_sender: UnboundedSender<OscEvent>,
) -> Pin<Box<tokio::task::JoinHandle<miette::Result<portable_pty::ExitStatus>>>> {
    Box::pin(tokio::spawn(async move {
        // Create a pseudo-terminal with reasonable dimensions.
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,        // Terminal height: 24 lines of text
                cols: 80,        // Terminal width: 80 characters per line
                pixel_width: 0,  // Not needed for text-based output
                pixel_height: 0, // Not needed for text-based output
            })
            .map_err(|e| miette::miette!("Failed to open PTY: {}", e))?;

        // Extract the endpoints of the PTY using type aliases.
        let controller: Controller = pty_pair.master;
        let controlled: Controlled = pty_pair.slave;

        // Configure the cargo command using our builder to ensure critical settings.
        let cmd = PtyCommandBuilder::new("cargo")
            .args(cargo_args)
            .enable_osc_sequences()
            .build()?;

        // [SPAWN 1] Spawn the command with PTY (makes is_terminal() return true).
        let mut controlled_child: ControlledChild = controlled
            .spawn_command(cmd)
            .map_err(|e| miette::miette!("Failed to spawn cargo command: {}", e))?;

        // [SPAWN 2] Spawn the reader task to process OSC sequences.
        //
        // CRITICAL: PTY LIFECYCLE AND FILE DESCRIPTOR MANAGEMENT
        // ========================================================
        // Understanding how PTYs handle EOF is crucial to avoiding deadlocks.
        //
        // ## The PTY File Descriptor Reference Counting Problem
        //
        // A PTY consists of two sides: master (controller) and slave (controlled).
        // The kernel's PTY implementation requires BOTH conditions for EOF:
        //
        // 1. The slave side must be closed (happens when the child process exits)
        // 2. The reader must be the ONLY remaining reference to the master
        //
        // ## How File Descriptors Work in Our Code
        //
        // 1. When we split the PTY pair (line 502-503):
        //    - `controller`: Holds the master side file descriptor
        //    - `controlled`: Holds the slave side file descriptor
        //
        // 2. The controller is MOVED into spawn_blocking (line 561):
        //    - Inside the closure, `controller.try_clone_reader()` creates a cloned FD
        //    - Both controller and controller_reader reference the same master PTY
        //    - When the closure ends, controller drops, leaving only controller_reader
        //
        // 3. The controlled side spawns cargo (line 512-514):
        //    - Cargo inherits the slave FD for its stdin/stdout/stderr
        //    - When cargo exits, it closes its copy of the slave FD
        //    - BUT our `controlled` variable STILL holds the original slave FD!
        //
        // ## Why We Need drop(controlled)
        //
        // Even though cargo has exited and closed its slave FD, our `controlled`
        // variable keeps the slave side open. The PTY won't send EOF to the master
        // until ALL slave file descriptors are closed. Without explicitly dropping
        // `controlled`, it would remain open until this entire function returns,
        // causing the reader to block forever waiting for EOF that never comes.
        //
        // ## The Solution: Explicit Resource Management
        //
        // 1. Move controller into spawn_blocking - ensures it drops after creating reader
        // 2. Explicitly drop controlled after cargo exits - closes our slave FD
        // 3. This allows the reader to receive EOF and exit cleanly
        //
        let blocking_reader_task_join_handle =
            tokio::task::spawn_blocking(move || -> miette::Result<()> {
                // Controller is MOVED into this closure, so it will be dropped
                // when this task completes, allowing proper PTY cleanup.
                let mut controller_reader = controller
                    .try_clone_reader()
                    .map_err(|e| miette::miette!("Failed to clone pty reader: {}", e))?;

                let mut read_buffer = [0u8; READ_BUFFER_SIZE];
                let mut osc_buffer = OscBuffer::new();

                loop {
                    // This is a synchronous blocking read operation.
                    // It will receive EOF when:
                    // 1. The slave side (controlled) is closed/dropped
                    // 2. No other references to the master exist
                    match controller_reader.read(&mut read_buffer) {
                        Ok(0) => break, // EOF - PTY closed
                        Ok(n) => {
                            // Process the buffer for OSC sequences in real-time
                            for event in osc_buffer.append_and_extract(&read_buffer, n) {
                                // Send events through channel for real-time display
                                // Ignore send errors (channel may be closed if main exited)
                                let _ = event_sender.send(event);
                            }
                        }
                        Err(_) => break, // Error reading - PTY likely closed
                    }
                }

                // Controller drops here automatically when the closure ends,
                // decrementing the master side's reference count.
                drop(controller);

                Ok(())
            });

        // [WAIT 1] Wait for the cargo build to complete.
        let status = tokio::task::spawn_blocking(move || controlled_child.wait())
            .await
            .into_diagnostic()?
            .into_diagnostic()?;

        // Explicitly drop the controlled (slave) side after cargo exits.
        // This closes the slave end of the PTY, which is necessary for the
        // reader to receive EOF. Without this, the slave FD would remain open
        // until this function returns, preventing EOF delivery to the reader.
        drop(controlled);

        // [WAIT 2] Wait for the reader task to complete.
        // Now that controlled is dropped, the reader will get EOF on its next
        // read() call and exit cleanly. This wait should complete quickly.
        let _unused = blocking_reader_task_join_handle.await.into_diagnostic()??;

        Ok(status)
    }))
}
