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

use miette::IntoDiagnostic;
use portable_pty::{CommandBuilder, MasterPty, PtySize, SlavePty, native_pty_system};
use std::{
    io::Read,
    ops::{Add, AddAssign},
    path::PathBuf,
    pin::Pin,
};
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
pub enum OscEvent {
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

/// Unified event type for PTY output that can contain both
/// OSC sequences and raw output data.
#[derive(Debug, Clone)]
pub enum PtyEvent {
    /// OSC sequence event (if OSC capture is enabled)
    Osc(OscEvent),
    /// Raw output data (stdout/stderr combined)
    Output(Vec<u8>),
    /// Process exited with status
    Exit(portable_pty::ExitStatus),
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

/// Configuration options that can be combined to build a PTY configuration.
///
/// These options are the building blocks that combine using the `+` operator
/// to create a [`PtyConfig`]. The combination follows a "last write wins per field"
/// strategy - each option modifies only the fields it cares about:
///
/// - `Osc` only sets `capture_osc` to true
/// - `Output` only sets `capture_output` to true
/// - `Size` only modifies `pty_size`
/// - `NoCaptureOutput` sets both capture flags to false
///
/// # Examples
///
/// ```
/// use PtyConfigOption::*;
///
/// // Single option (automatically converts to PtyConfig)
/// spawn_pty(cmd, Osc, sender);
///
/// // Combine multiple options
/// spawn_pty(cmd, Osc + Output, sender);
///
/// // With custom size (last size wins)
/// spawn_pty(cmd, Osc + Output + Size(custom_size), sender);
///
/// // NoCaptureOutput overrides previous capture settings
/// let config = Osc + Output + NoCaptureOutput; // Both captures disabled
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PtyConfigOption {
    /// Capture and parse OSC sequences
    Osc,
    /// Capture raw output data
    Output,
    /// Set custom PTY dimensions
    Size(PtySize),
    /// Disable all capture (sets both capture flags to false)
    NoCaptureOutput,
}

/// Final configuration for PTY command execution.
///
/// This struct is built by combining [`PtyConfigOption`] values using the `+` operator.
/// It represents the complete configuration state after all options have been applied.
///
/// # Examples
///
/// ```
/// use PtyConfigOption::*;
///
/// // Build from options
/// let config = Osc + Output; // Creates a PtyConfig
///
/// // Can continue adding to an existing config
/// let config = config + Size(custom_size);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PtyConfig {
    pub capture_osc: bool,
    pub capture_output: bool,
    pub pty_size: PtySize,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            capture_osc: false,
            capture_output: true,
            pty_size: PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            },
        }
    }
}

impl PtyConfig {
    /// Check if OSC capture is enabled
    pub fn is_osc_capture_enabled(&self) -> bool {
        self.capture_osc
    }

    /// Check if output capture is enabled
    pub fn is_output_capture_enabled(&self) -> bool {
        self.capture_output
    }

    /// Get the PTY size configuration
    pub fn get_pty_size(&self) -> PtySize {
        self.pty_size
    }

    /// Apply a configuration option to this config.
    /// Uses "last write wins per field" strategy.
    fn apply(&mut self, option: PtyConfigOption) {
        match option {
            PtyConfigOption::Osc => self.capture_osc = true,
            PtyConfigOption::Output => self.capture_output = true,
            PtyConfigOption::Size(size) => self.pty_size = size,
            PtyConfigOption::NoCaptureOutput => {
                self.capture_osc = false;
                self.capture_output = false;
            }
        }
    }
}

/// Convert a single option into a complete PtyConfig
impl From<PtyConfigOption> for PtyConfig {
    fn from(option: PtyConfigOption) -> Self {
        let mut config = PtyConfig::default();
        config.apply(option);
        config
    }
}

/// Combine two options to create a PtyConfig
impl Add for PtyConfigOption {
    type Output = PtyConfig;

    fn add(self, rhs: Self) -> PtyConfig {
        let mut config = PtyConfig::from(self);
        config.apply(rhs);
        config
    }
}

/// Add an option to an existing config
impl Add<PtyConfigOption> for PtyConfig {
    type Output = PtyConfig;

    fn add(mut self, rhs: PtyConfigOption) -> PtyConfig {
        self.apply(rhs);
        self
    }
}

/// Add a config to an option (for symmetry)
impl Add<PtyConfig> for PtyConfigOption {
    type Output = PtyConfig;

    fn add(self, rhs: PtyConfig) -> PtyConfig {
        rhs + self
    }
}

/// Implement AddAssign for `+=` operator on PtyConfig
impl AddAssign<PtyConfigOption> for PtyConfig {
    fn add_assign(&mut self, rhs: PtyConfigOption) {
        self.apply(rhs);
    }
}

/// Allow creating PtyConfig from PtySize via PtyConfigOption
impl From<PtySize> for PtyConfigOption {
    fn from(size: PtySize) -> Self {
        PtyConfigOption::Size(size)
    }
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
    /// If not called, defaults to the current directory when [`build()`](Self::build) is invoked.
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

    /// Builds the final [`CommandBuilder`] with all configurations applied.
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


/// Spawns a command in a PTY to capture output without providing input.
///
/// This is a read-only PTY command spawner that can:
/// 1. Run any command (not just cargo)
/// 2. Optionally capture and parse OSC sequences
/// 3. Optionally capture raw output data
/// 4. Use custom PTY dimensions
///
/// Note: This function does not allow sending input to the spawned process.
/// For interactive PTY sessions, a future `spawn_pty_interactive` function would be needed.
///
/// # Arguments
/// * `command` - The command to execute (configured via [`CommandBuilder`])
/// * `config` - Configuration for what to capture (implements [`Into<PtyConfig>`])
/// * `event_sender` - Channel sender for [`PtyEvent`]s
///
/// # Returns
/// A pinned [`JoinHandle`](tokio::task::JoinHandle) that resolves to the process exit status.
///
/// ## Why Pinning is Required
///
/// The [`JoinHandle`](tokio::task::JoinHandle) is pinned to the heap using [`Box::pin`] for two important reasons:
///
/// 1. **[`tokio::select!`] requirement**: The [`JoinHandle`](tokio::task::JoinHandle) doesn't implement [`Unpin`], and
///    [`tokio::select!`] requires all futures to be [`Unpin`]. By returning a pre-pinned handle,
///    callers can use it directly in [`select!`](tokio::select!) blocks without additional pinning:
///    ```rust
///    let mut handle = spawn_pty_capture_output_no_input(cmd, config, sender);
///    tokio::select! {
///        result = &mut handle => { /* handle completion */ }
///        event = receiver.recv() => { /* handle events */ }
///    }
///    ```
///
/// 2. **Heap vs Stack pinning**: We use [`Box::pin`] (heap pinning) rather than the [`pin!`](std::pin::pin!)
///    macro (stack pinning) because this function returns the pinned value. Stack pinning
///    with [`pin!`](std::pin::pin!) creates a value that cannot outlive the function's stack frame, making it
///    impossible to return. Heap pinning ensures the pinned future remains valid after the
///    function returns and can be safely moved between async contexts.
///
/// This design simplifies usage by eliminating the need for manual pinning at the call site
/// while ensuring the future can be safely polled across await points.
///
/// # Examples
///
/// ## Basic command with output capture
/// ```
/// use PtyConfigOption::*;
///
/// let cmd = PtyCommandBuilder::new("ls")
///     .args(["-la"])
///     .build()?;
///
/// let (sender, mut receiver) = unbounded_channel();
/// let handle = spawn_pty_capture_output_no_input(cmd, Output, sender);
///
/// // Process events
/// while let Some(event) = receiver.recv().await {
///     match event {
///         PtyEvent::Output(data) => print!("{}", String::from_utf8_lossy(&data)),
///         PtyEvent::Exit(status) => println!("Exited with: {:?}", status),
///         _ => {}
///     }
/// }
/// ```
///
/// ## Cargo build with OSC progress tracking
/// ```
/// use PtyConfigOption::*;
///
/// let cmd = PtyCommandBuilder::new("cargo")
///     .args(["build"])
///     .enable_osc_sequences()
///     .build()?;
///
/// let config = Osc + Output;
/// let (sender, mut receiver) = unbounded_channel();
/// let handle = spawn_pty_capture_output_no_input(cmd, config, sender);
///
/// while let Some(event) = receiver.recv().await {
///     match event {
///         PtyEvent::Osc(OscEvent::ProgressUpdate(pct)) => {
///             println!("Build progress: {}%", pct);
///         }
///         PtyEvent::Output(data) => {
///             // Also see the actual cargo output
///             print!("{}", String::from_utf8_lossy(&data));
///         }
///         _ => {}
///     }
/// }
/// ```
///
/// ## Custom PTY dimensions
/// ```
/// use PtyConfigOption::*;
///
/// let config = Size(PtySize { rows: 40, cols: 120, pixel_width: 0, pixel_height: 0 })
///     + Output;
/// ```
///
/// The function handles all PTY lifecycle management internally, including proper
/// cleanup of file descriptors to avoid deadlocks.
pub fn spawn_pty_capture_output_no_input(
    /* move */ command: CommandBuilder,
    /* move */ config: impl Into<PtyConfig>,
    /* move */ event_sender: UnboundedSender<PtyEvent>,
) -> Pin<Box<tokio::task::JoinHandle<miette::Result<portable_pty::ExitStatus>>>> {
    let config = config.into();

    Box::pin(tokio::spawn(async move {
        // Create a pseudo-terminal with configured dimensions.
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(config.get_pty_size())
            .map_err(|e| miette::miette!("Failed to open PTY: {}", e))?;

        // Extract the endpoints of the PTY using type aliases.
        let controller: Controller = pty_pair.master;
        let controlled: Controlled = pty_pair.slave;

        // [SPAWN 1] Spawn the command with PTY (makes is_terminal() return true).
        let mut controlled_child: ControlledChild = controlled
            .spawn_command(command)
            .map_err(|e| miette::miette!("Failed to spawn command: {}", e))?;

        // [SPAWN 2] Spawn the reader task to process output.
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
        // ## Why We Need Explicit Resource Management
        //
        // Even though the child process has exited and closed its slave FD, our `controlled`
        // variable keeps the slave side open. The PTY won't send EOF to the master
        // until ALL slave file descriptors are closed. Without explicitly dropping
        // `controlled`, it would remain open until this entire function returns,
        // causing the reader to block forever waiting for EOF that never comes.
        //
        // ## The Solution
        //
        // 1. Move controller into spawn_blocking - ensures it drops after creating reader
        // 2. Explicitly drop controlled after process exits - closes our slave FD
        // 3. This allows the reader to receive EOF and exit cleanly
        //
        let capture_osc = config.is_osc_capture_enabled();
        let capture_output = config.is_output_capture_enabled();

        // Clone the sender for the blocking task
        let reader_sender = event_sender.clone();

        let blocking_reader_task_join_handle =
            tokio::task::spawn_blocking(move || -> miette::Result<()> {
                // Controller is MOVED into this closure, so it will be dropped
                // when this task completes, allowing proper PTY cleanup.
                let mut controller_reader = controller
                    .try_clone_reader()
                    .map_err(|e| miette::miette!("Failed to clone pty reader: {}", e))?;

                let mut read_buffer = [0u8; READ_BUFFER_SIZE];
                let mut osc_buffer = if capture_osc {
                    Some(OscBuffer::new())
                } else {
                    None
                };

                loop {
                    // This is a synchronous blocking read operation.
                    match controller_reader.read(&mut read_buffer) {
                        Ok(0) => break, // EOF - PTY closed
                        Ok(n) => {
                            let data = &read_buffer[..n];

                            // Send raw output if configured
                            if capture_output {
                                let _ = reader_sender.send(PtyEvent::Output(data.to_vec()));
                            }

                            // Process OSC sequences if configured
                            if let Some(ref mut osc_buf) = osc_buffer {
                                for event in osc_buf.append_and_extract(data, n) {
                                    let _ = reader_sender.send(PtyEvent::Osc(event));
                                }
                            }
                        }
                        Err(_) => break, // Error reading - PTY likely closed
                    }
                }

                // Controller drops here automatically when the closure ends.
                drop(controller);

                Ok(())
            });

        // [WAIT 1] Wait for the command to complete.
        let status = tokio::task::spawn_blocking(move || controlled_child.wait())
            .await
            .into_diagnostic()?
            .into_diagnostic()?;

        // Store exit code before moving status into event
        let exit_code = status.exit_code();

        // Send exit event (moves status)
        let _ = event_sender.send(PtyEvent::Exit(status));

        // Explicitly drop the controlled (slave) side after process exits.
        drop(controlled);

        // [WAIT 2] Wait for the reader task to complete.
        blocking_reader_task_join_handle.await.into_diagnostic()??;

        // Recreate status for return value
        Ok(portable_pty::ExitStatus::with_exit_code(exit_code))
    }))
}

/// Binary for capturing and displaying OSC progress sequences from cargo builds.
///
/// This program demonstrates how to capture OSC (Operating System Command) sequences
/// emitted by cargo when running in a terminal that supports progress reporting.
/// It uses a pseudo-terminal (PTY) to make cargo think it's running in an interactive
/// terminal, which triggers the emission of OSC 9;4 progress sequences.
///
/// # OSC Sequence Format
///
/// Cargo emits OSC sequences in the format: `ESC]9;4;{state};{progress}ESC\\`
///
/// Where:
/// - `state` 0: Clear/remove progress
/// - `state` 1: Set specific progress (0-100%)
/// - `state` 2: Build error occurred
/// - `state` 3: Indeterminate progress
///
/// # Usage
///
/// Run this binary to see cargo build progress in real-time:
/// ```bash
/// cargo run --bin real
/// ```
#[tokio::main]
async fn main() -> miette::Result<()> {
    /// Runs cargo clean using the generic PTY command.
    async fn run_cargo_clean() -> miette::Result<()> {
        println!(
            "{}🧹 Running 'cargo clean' to ensure a fresh build...{}",
            YELLOW, RESET
        );

        let cmd = PtyCommandBuilder::new("cargo")
            .args(["clean", "-q"])
            .build()?;

        let (sender, mut receiver) = unbounded_channel();
        let mut handle =
            spawn_pty_capture_output_no_input(cmd, PtyConfigOption::NoCaptureOutput, sender);

        // Wait for completion
        tokio::select! {
            result = &mut handle => {
                let status = result.into_diagnostic()??;
                if status.success() {
                    println!("{}✓ Cargo clean completed successfully{}\n", GREEN, RESET);
                } else {
                    return Err(miette::miette!("Cargo clean failed"));
                }
            }
            Some(event) = receiver.recv() => {
                if let PtyEvent::Exit(status) = event {
                    if !status.success() {
                        return Err(miette::miette!("Cargo clean failed"));
                    }
                }
            }
        }

        Ok(())
    }

    /// Runs a single cargo build with OSC capture.
    async fn run_build_with_osc_capture(run_number: u32) -> miette::Result<()> {
        println!("{}========================================", YELLOW);
        println!(
            "{}Starting Cargo build #{} with OSC capture...",
            YELLOW, run_number
        );
        println!(
            "{}========================================{}",
            YELLOW, RESET
        );

        // Configure cargo build command with OSC sequences enabled
        let cmd = PtyCommandBuilder::new("cargo")
            .args(["build"])
            .enable_osc_sequences()
            .build()?;

        // Create channel for PTY events
        let (sender, mut receiver) = unbounded_channel();

        // Use generic PTY command with OSC capture only
        let mut handle = spawn_pty_capture_output_no_input(cmd, PtyConfigOption::Osc, sender);

        // Track if we saw any progress updates
        let mut saw_progress = false;

        // Handle events as they arrive until cargo completes
        loop {
            tokio::select! {
                // Handle cargo build completion
                result = &mut handle => {
                    let status = result.into_diagnostic()??;

                    // Print summary
                    if saw_progress {
                        println!(
                            "{}✅ Build #{} completed with progress tracking (status: {:?}){}",
                            GREEN, run_number, status, RESET
                        );
                    } else {
                        println!(
                            "{}✅ Build #{} completed (no progress - everything cached) (status: {:?}){}",
                            GREEN, run_number, status, RESET
                        );
                    }
                    break;
                }
                // Handle incoming PTY events
                Some(event) = receiver.recv() => {
                    match event {
                        PtyEvent::Osc(osc_event) => {
                            match osc_event {
                                OscEvent::ProgressUpdate(percentage) => {
                                    saw_progress = true;
                                    println!(
                                        "{}📊 Build #{} progress: {}%{}",
                                        GREEN, run_number, percentage, RESET
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
                        PtyEvent::Exit(_) => {
                            // Exit event will be handled by the handle completion above
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    println!(
        "\
        {}╔═══════════════════════════════════════════════════════════════╗\n\
        {}║  Demo: Cargo Build OSC Progress Sequences with Generic API    ║\n\
        {}╚═══════════════════════════════════════════════════════════════╝{}",
        YELLOW, YELLOW, YELLOW, RESET
    );

    // Step 1: Run cargo clean to ensure the following build generates OSC sequences
    println!(
        "\n{}▶ Step 1: Running cargo clean to ensure fresh build{}",
        YELLOW, RESET
    );
    run_cargo_clean().await?;

    // Step 2: Run cargo build - should generate OSC sequences
    println!(
        "\n{}▶ Step 2: First cargo build (expect progress updates){}",
        YELLOW, RESET
    );
    run_build_with_osc_capture(1).await?;

    // Step 3: Run cargo build again - should NOT generate OSC sequences (cached)
    println!(
        "\n{}▶ Step 3: Second cargo build (expect no progress - cached){}",
        YELLOW, RESET
    );
    run_build_with_osc_capture(2).await?;

    println!(
        "\n{}✨ Demo complete! The generic spawn_pty_command successfully captured OSC sequences.{}",
        GREEN, RESET
    );

    Ok(())
}
