# OSC Terminal Control Sequences in Cargo

> - Tracking issue:
>   [rust-scratch 117](https://github.com/nazmulidris/rust-scratch/issues/117)
> - Related issue: [r3bl-cmdr 437](https://github.com/r3bl-org/r3bl-open-core/issues/437)
> - Turn this into a developerlife.com article once the POC is complete and the
>   implementation details are verified to work.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Overview](#overview)
- [Quick Summary for Developers](#quick-summary-for-developers)
- [OSC Sequence Format](#osc-sequence-format)
  - [State Values](#state-values)
  - [Example Sequences](#example-sequences)
- [Implementation Details](#implementation-details)
  - [Key Files](#key-files)
  - [Terminal Detection](#terminal-detection)
  - [Progress Flow](#progress-flow)
- [Configuration](#configuration)
- [Capturing OSC Sequences from Cargo](#capturing-osc-sequences-from-cargo)
  - [Quick Start: Dependencies](#quick-start-dependencies)
  - [The TTY Problem](#the-tty-problem)
  - [Why `Stdio::inherit()` Won't Work for Capture](#why-stdioinherit-wont-work-for-capture)
  - [The Solution: Using PTY with `portable-pty`](#the-solution-using-pty-with-portable-pty)
    - [Understanding PTY Architecture](#understanding-pty-architecture)
      - [How PTY Solves the OSC Problem](#how-pty-solves-the-osc-problem)
      - [Why Other Approaches Fail](#why-other-approaches-fail)
      - [The Virtual Terminal Effect](#the-virtual-terminal-effect)
    - [Code Snippet](#code-snippet)
- [Complete Working Example](#complete-working-example)
  - [Full POC Implementation](#full-poc-implementation)
  - [Running the POC](#running-the-poc)
  - [Expected Output](#expected-output)
  - [Troubleshooting](#troubleshooting)
- [Environment Considerations](#environment-considerations)
- [Testing](#testing)
- [References](#references)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Overview

Cargo emits OSC (Operating System Command) terminal control sequences during build
operations to communicate progress status to compatible terminal emulators. These
sequences allow terminals to display build progress in title bars, taskbars, or other UI
elements.

This document explains how to programmatically capture and parse these OSC sequences when
spawning `cargo build` from a Rust program.

Here are some useful links for context:

1. [My cargo repo fork](https://github.com/nazmulidris/cargo), which is cloned to
   `/home/nazmul/github/cargo` folder
2. [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code#Operating_System_Command_sequences)
3. [OSC terminal control sequences](https://blog.vucica.net/2017/07/what-are-osc-terminal-control-sequences-escape-codes.html)

## Quick Summary for Developers

1. **Goal:**
   - Capture and parse progress information from `cargo build` using OSC escape sequences.
2. **Key Challenge:**

   - Cargo only emits OSC sequences when connected to an interactive TTY, and a compatible
     terminal (like `wezterm`).
   - Spawning a child process using `std::process::Command` and standard process pipes
     don't work, since the child process correctly detects the terminal is not-interactive
     (not connected to TTY).
   - Inheriting stdio from the current process doesn't work either, for two reasons. While
     the child process does detect the terminal as interactive (connected to TTY), it also
     clobbers output from the parent CLI or TUI process by sending all the output
     including OSC sequences directly to the terminal, bypassing the parent process. And
     there is no way to capture or parse the output/OSC codes.

     | Method             | `is_terminal()` | OSC Emitted? | Can Capture? | Problem                      |
     | ------------------ | --------------- | ------------ | ------------ | ---------------------------- |
     | **Standard pipes** | ❌ false        | ❌ No        | ✅ Yes       | No OSC sequences generated   |
     | **Stdio inherit**  | ✅ true         | ✅ Yes       | ❌ No        | Output bypasses your program |
     | **PTY**            | ✅ true         | ✅ Yes       | ✅ Yes       | Perfect solution!            |

3. **Solution:**
   - Use a pseudo-terminal (PTY) via the `portable-pty` crate to simulate a terminal
     environment.
   - And "fake" that `wezterm` is the terminal program by setting `TERM_PROGRAM=WezTerm`.
4. **What You'll Get:**
   - Real-time build progress updates (0-100%) that you can process programmatically.
5. **Jump to:**
   - [Complete Working Example](#complete-working-example) for a ready-to-run POC.

## OSC Sequence Format

Cargo uses the **OSC 9;4** format (ConEmu-style progress reporting):

```
ESC ] 9 ; 4 ; st ; pr ST
```

Where:

- `ESC` = `\x1b` (escape character)
- `]` = OSC introducer
- `9;4` = ConEmu-specific progress command
- `st` = state (0-4)
- `pr` = progress value (0-100)
- `ST` = `\x1b\\` (string terminator)

### State Values

- `0`: Remove progress indicator
- `1`: Set progress value (0-100)
- `2`: Set error state in taskbar
- `3`: Set indeterminate state (animated, no specific progress)
- `4`: Set paused state (not used by Cargo)

### Example Sequences

- Start progress: `\x1b]9;4;1;0\x1b\\`
- 50% complete: `\x1b]9;4;1;50\x1b\\`
- Build error: `\x1b]9;4;2;100\x1b\\`
- Remove progress: `\x1b]9;4;0;0\x1b\\`

## Implementation Details

### Key Files

The Cargo source code is in `/home/nazmul/github/cargo/` folder.

1. **`src/cargo/util/progress.rs`**

   - Contains the main progress bar implementation
   - `StatusValue` enum (lines 87-99) defines progress states
   - `Display` implementation (lines 161-179) formats OSC sequences
   - `TerminalIntegration` struct manages OSC emission

2. **`src/cargo/core/shell.rs`**

   - Terminal capability detection (lines 594-600)
   - Checks environment variables for terminal support

3. **`src/cargo/core/compiler/job_queue/mod.rs`**
   - Creates progress bar during build (line 481)
   - Updates progress as compilation proceeds
   - Marks progress as error on build failure (line 872)

### Terminal Detection

Cargo detects OSC support by checking environment variables in
`supports_term_integration()`:

```rust
fn supports_term_integration(stream: &dyn IsTerminal) -> bool {
    let windows_terminal = std::env::var("WT_SESSION").is_ok();
    let conemu = std::env::var("ConEmuANSI").ok() == Some("ON".into());
    let wezterm = std::env::var("TERM_PROGRAM").ok() == Some("WezTerm".into());

    (windows_terminal || conemu || wezterm) && stream.is_terminal()
}
```

Supported terminals:

- **Windows Terminal**: Detected via `WT_SESSION` environment variable
- **ConEmu**: Detected via `ConEmuANSI=ON`
- **WezTerm**: Detected via `TERM_PROGRAM=WezTerm`

### Progress Flow

1. Build starts → Progress bar created with `Progress::with_style()`
2. Terminal integration initialized based on detection and config
3. During compilation:
   - `tick()` or `tick_now()` called with current/max units
   - Progress percentage calculated
   - OSC sequence emitted to stderr
4. On completion or error:
   - Final state sent (100% or error)
   - Progress cleared with remove sequence

## Configuration

Users can control OSC emission via `.cargo/config.toml`:

```toml
[term]
progress.term-integration = true   # Enable OSC sequences (auto-detected by default)
# or
progress.term-integration = false  # Disable OSC sequences

# Other progress options
progress.when = "auto"     # "auto", "always", or "never"
progress.width = 80        # Terminal width for progress bar
```

## Capturing OSC Sequences from Cargo

### Quick Start: Dependencies

Add this to your `Cargo.toml`:

```toml
[dependencies]
portable-pty = "0.8"
tokio = { version = "1.0", features = ["full"] }
miette = { version = "7.2", features = ["fancy"] }
```

### The TTY Problem

**Important**: When using pipes with `std::process::Command`, the spawned process detects
it's NOT connected to a TTY, causing `is_terminal()` to return false and **no OSC
sequences will be emitted**. You need a pseudo-terminal (PTY) to capture OSC sequences.

### Why `Stdio::inherit()` Won't Work for Capture

While you can use `Stdio::inherit()` to let the spawned process use your real terminal
(and OSC codes will be displayed), this approach has a critical limitation:

```rust
use std::process::{Command, Stdio};

let mut cmd = Command::new("cargo");
cmd.arg("build")
   .env("TERM_PROGRAM", "WezTerm")
   .stdin(Stdio::inherit())
   .stdout(Stdio::inherit())
   .stderr(Stdio::inherit());  // Uses real TTY, OSC goes directly to terminal

let mut child = cmd.spawn()?;
child.wait()?;
// Problem: Cannot capture or parse the output/OSC codes!
```

**Why this doesn't work for capture:**

- Output goes directly to your terminal, bypassing your program
- You cannot intercept, parse, or process the OSC sequences
- You have no programmatic access to the build progress data
- This is only useful if you want to display progress without processing it

### The Solution: Using PTY with `portable-pty`

The only way to both trigger OSC emission AND capture the sequences is to use a
pseudo-terminal (PTY).

#### Understanding PTY Architecture

A **PTY (Pseudo-Terminal)** is a software-based virtual terminal that creates two
connected endpoints:

- **Controller endpoint**: Your program connects here to manage the virtual terminal
- **Controlled endpoint**: The spawned process connects here, believing it's a real
  terminal

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Your Program  │◄──►│     PTY      │◄──►│ Spawned Process │
│  (Controller)   │    │ Controller/  │    │  (Controlled)   │
│                 │    │ Controlled   │    │                 │
└─────────────────┘    └──────────────┘    └─────────────────┘
```

##### How PTY Solves the OSC Problem

The genius of PTY is that it creates a **bidirectional pipe that appears to be a real
terminal** to the spawned process:

1. **Your program** creates a PTY pair and gets handles to both endpoints
2. **Cargo process** is launched with the controlled endpoint as its terminal
3. **Cargo believes** it's connected to a real terminal → `is_terminal()` returns `true`
4. **OSC sequences are emitted** because terminal integration is enabled
5. **Your program** can read all output (including OSC) through the controller endpoint

##### Why Other Approaches Fail

| Method             | `is_terminal()` | OSC Emitted? | Can Capture? | Problem                      |
| ------------------ | --------------- | ------------ | ------------ | ---------------------------- |
| **Standard pipes** | ❌ false        | ❌ No        | ✅ Yes       | No OSC sequences generated   |
| **Stdio inherit**  | ✅ true         | ✅ Yes       | ❌ No        | Output bypasses your program |
| **PTY**            | ✅ true         | ✅ Yes       | ✅ Yes       | Perfect solution!            |

##### The Virtual Terminal Effect

PTY essentially turns your program into a "virtual terminal emulator":

- **Satisfies Cargo's terminal detection** (makes `is_terminal()` return true)
- **Enables OSC emission** (because it looks like a compatible terminal)
- **Allows programmatic capture** (your program controls the virtual terminal)
- **Provides full control** (you can process, filter, or transform the data)

This elegant solution is **the only method** that satisfies both requirements: triggering
OSC emission AND capturing the sequences for programmatic processing.

#### Code Snippet

> **Note**: The `portable-pty` crate uses the legacy API terms `master`/`slave` for
> historical compatibility, but conceptually these represent the controller/controlled
> relationship described above.

````rust
// Cargo.toml dependencies:
// portable-pty = "0.8"
// tokio = { version = "1.0", features = ["full"] }
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio::io::AsyncReadExt;


/// ```
/// ┌─────────────────┐    ┌──────────────┐    ┌───────────────────┐
/// │   Your Program  │◄──►│ Controller   │    │ Cargo Process     │
/// │                 │    │      ↕       │    │                   │
/// │ Reads/writes    │    │     PTY      │    │ stdin/stdout/     │
/// │ through         │    │      ↕       │    │ stderr redirected │
/// │ controller      │    │   Controlled │◄──►│ to controlled     │
/// └─────────────────┘    └──────────────┘    └───────────────────┘
/// ```
async fn spawn_with_pty() -> Result<(), Box<dyn std::error::Error>> {
    // Create a pseudo-terminal
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,           // Terminal height: 24 lines of text (classic terminal size)
                            // Cargo uses this for vertical scrolling behavior
        cols: 80,           // Terminal width: 80 characters per line (standard terminal width)
                            // Cargo formats progress bars and output to fit within this width
        pixel_width: 0,     // Pixel width: Not needed for text-based output capture
        pixel_height: 0,    // Pixel height: Not needed for text-based output capture
    }).into_diagnostic()?;

    // Extract endpoints with descriptive names
    let controller = pair.master;  // Where your program reads/writes
    let controlled = pair.slave;   // Where the spawned process connects

    // Configure the command
    let mut cmd = CommandBuilder::new("cargo");
    cmd.arg("build");
    cmd.env("TERM_PROGRAM", "WezTerm");

    // Spawn with PTY (this makes is_terminal() return true!)
    // Note: The cargo child process uses 'controlled' as its stdin/stdout/stderr
    // This makes cargo believe it's connected to a real terminal
    let mut controlled_child = controlled.spawn_command(cmd)?;

    // Read output with OSC sequences (async version)
    // Note: controller is the "controller" endpoint where your program reads from
    let mut controller_reader = controller.try_clone_reader()?;

    // Spawn the reading operation on a separate green thread/task
    // NOTE: This approach reads ALL output into a string at once after the process completes.
    // This is simple but NOT ideal for real-time progress monitoring since OSC sequences
    // are only processed after the entire build finishes. For real-time progress updates,
    // use the buffered reading approach shown in the main example below.
    let read_handle = tokio::spawn(async move {
        let mut output = String::new();
        // Read entire output stream into a single string (blocks until EOF)
        controller_reader.read_to_string(&mut output).await?;
        Ok::<String, Box<dyn std::error::Error + Send + Sync>>(output)
    });

    // Wait for the child process to complete
    //
    // IMPORTANT: Use spawn_blocking because child.wait() is a synchronous blocking operation.
    // If we called controlled_child.wait() directly in this async context, it would block
    // the entire tokio runtime thread, preventing other async tasks from running.
    // spawn_blocking moves the blocking operation to a dedicated thread pool.
    // The `??` double question mark operator handles two layers of Result unwrapping:
    // - First `?` unwraps the JoinResult from spawn_blocking (handles task panics/cancellation)
    // - Second `?` unwraps the Result from child.wait() (handles process wait errors)
    //
    // NOTE: portable-pty doesn't have native async support, but provides async-compatible methods:
    // - try_clone_reader() returns readers that work with tokio::io::AsyncReadExt
    // - child.wait() is blocking, so we use spawn_blocking to avoid blocking the async runtime
    // - For fully async PTY operations, consider alternatives like tokio-ptyprocess (experimental)
    let status = tokio::task::spawn_blocking(move || controlled_child.wait()).await??;

    // Wait for the reading task to complete and get the output
    // The `??` operator handles two layers of Result unwrapping:
    // - First `?` unwraps the JoinResult from tokio::spawn (handles task panics/cancellation)
    // - Second `?` unwraps the Result<String, Error> from the task's return value
    let output = read_handle.await??;

    println!("Output with OSC: {}", output);
    println!("Build completed with status: {}", status);
    Ok(())
}
````

**Why PTY works:**

- Creates a virtual terminal that satisfies `is_terminal()` check
- Allows your program to act as the "terminal" for the spawned process
- Captures all output including OSC sequences
- Provides full programmatic control over the data

## Complete Working Example

### Full POC Implementation

Save this as `src/main.rs`:

```rust
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio::io::AsyncReadExt;
use std::convert::TryFrom;
use miette::{IntoDiagnostic, Result};

/// Represents the different types of OSC progress events that Cargo can emit
#[derive(Debug, Clone, PartialEq)]
enum OscEvent {
    /// Clear/remove progress indicator (OSC state 0)
    ProgressCleared,
    /// Set specific progress value 0-100% (OSC state 1)
    ProgressUpdate(f64),
    /// Build error occurred (OSC state 2)
    BuildError(f64),
    /// Indeterminate progress - build is running but no specific progress (OSC state 3)
    IndeterminateProgress,
    /// Unknown or unsupported OSC state
    Unknown(u8, f64),
}

impl OscEvent {
    /// Check if two events are of the same variant type (ignoring data)
    fn is_same_kind(&self, other: &OscEvent) -> bool {
        matches!(
            (self, other),
            (Self::ProgressCleared, Self::ProgressCleared)
            | (Self::ProgressUpdate(_), Self::ProgressUpdate(_))
            | (Self::BuildError(_), Self::BuildError(_))
            | (Self::IndeterminateProgress, Self::IndeterminateProgress)
            | (Self::Unknown(_, _), Self::Unknown(_, _))
        )
    }
}

impl From<(u8, f64)> for OscEvent {
    /// Convert raw OSC state and progress values into a typed event
    fn from((state, progress): (u8, f64)) -> Self {
        match state {
            0 => OscEvent::ProgressCleared,
            1 => OscEvent::ProgressUpdate(progress),
            2 => OscEvent::BuildError(progress),
            3 => OscEvent::IndeterminateProgress,
            _ => OscEvent::Unknown(state, progress),
        }
    }
}

impl TryFrom<&str> for OscEvent {
    type Error = String;

    /// Parse OSC 9;4 progress sequences from text and return typed event
    fn try_from(text: &str) -> Result<Self, Self::Error> {
        // OSC sequence format: \x1b]9;4;{state};{progress}\x1b\\
        let osc_start = "\x1b]9;4;";
        let osc_end = "\x1b\\";

        // Find the start of an OSC sequence
        let start_idx = text.find(osc_start)
            .ok_or_else(|| "No OSC sequence found".to_string())?;
        let after_start = &text[start_idx + osc_start.len()..];

        // Find the end of the sequence
        let end_idx = after_start.find(osc_end)
            .ok_or_else(|| "OSC sequence not terminated".to_string())?;
        let params = &after_start[..end_idx];

        // Parse state;progress
        let parts: Vec<&str> = params.split(';').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid OSC parameter count: expected 2, got {}", parts.len()));
        }

        let state = parts[0].parse::<u8>()
            .map_err(|e| format!("Failed to parse state: {}", e))?;
        let progress = parts[1].parse::<f64>()
            .map_err(|e| format!("Failed to parse progress: {}", e))?;

        Ok(OscEvent::from((state, progress)))
    }
}

/// Tracks progress state and handles display logic with intelligent filtering
#[derive(Debug)]
struct ProgressTracker {
    last_percentage: f64,
    current_state: Option<OscEvent>,
}

impl ProgressTracker {
    fn new() -> Self {
        Self {
            last_percentage: -1.0,
            current_state: None,
        }
    }

    /// Process an OSC event and display updates when significant changes occur
    fn handle_event(&mut self, event: OscEvent) {
        // Determine if this event should trigger a display update
        let should_display = match (&self.current_state, &event) {
            // Always display state changes (different event types)
            (Some(prev), current) if !prev.is_same_kind(current) => true,

            // For progress updates, only display if change is significant (> 0.1%)
            (_, OscEvent::ProgressUpdate(percentage)) => {
                (percentage - self.last_percentage).abs() > 0.1
            },

            // Always display non-progress events
            (_, OscEvent::ProgressCleared) => true,
            (_, OscEvent::BuildError(_)) => true,
            (_, OscEvent::IndeterminateProgress) => true,
            (_, OscEvent::Unknown(_, _)) => true,

            // First event - always display
            (None, _) => true,
        };

        if should_display {
            self.display_event(&event);

            // Update tracking state
            if let OscEvent::ProgressUpdate(percentage) = &event {
                self.last_percentage = *percentage;
            }
            self.current_state = Some(event);
        }
    }

    /// Display the event with appropriate formatting and emoji
    fn display_event(&self, event: &OscEvent) {
        match event {
            OscEvent::ProgressCleared => {
                println!("\n✓ Progress tracking cleared");
            },
            OscEvent::ProgressUpdate(percentage) => {
                println!("📊 Build progress: {:.1}%", percentage);
            },
            OscEvent::BuildError(last_progress) => {
                println!("\n❌ Build error (progress was at {:.1}%)", last_progress);
            },
            OscEvent::IndeterminateProgress => {
                println!("⏳ Build in progress (indeterminate)");
            },
            OscEvent::Unknown(state, value) => {
                println!("❓ Unknown OSC state: {} (value: {:.1})", state, value);
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Cargo build with OSC capture...\n");
    run_cargo_with_osc_capture().await?;
    Ok(())
}

async fn run_cargo_with_osc_capture() -> Result<()> {
    // Create a pseudo-terminal with reasonable dimensions
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,           // Terminal height: 24 lines of text (classic terminal size)
                            // Cargo uses this for vertical scrolling behavior
        cols: 80,           // Terminal width: 80 characters per line (standard terminal width)
                            // Cargo formats progress bars and output to fit within this width
        pixel_width: 0,     // Pixel width: Not needed for text-based output capture
        pixel_height: 0,    // Pixel height: Not needed for text-based output capture
    }).into_diagnostic()?;

    // Configure the cargo build command
    let mut cmd = CommandBuilder::new("cargo");
    cmd.arg("build");
    // CRITICAL: Set TERM_PROGRAM to trigger OSC emission
    cmd.env("TERM_PROGRAM", "WezTerm");

    // Spawn the command with PTY (makes is_terminal() return true)
    // The controlled endpoint (pair.slave) is where the cargo process connects
    let mut child = pair.slave.spawn_command(cmd).into_diagnostic()?;

    // Read and parse output in a separate async task
    // The controller endpoint (pair.master) is where we read the output
    let mut reader = pair.master.try_clone_reader().into_diagnostic()?;
    let handle = tokio::spawn(async move {
        // 4KB buffer: Optimal size for terminal output (page-aligned, handles OSC + context)
        let mut buffer = [0u8; 4096];
        let mut progress_tracker = ProgressTracker::new();

        loop {
            match reader.read(&mut buffer).await {
                Ok(/* bytes read */ 0) => break, // EOF
                Ok(/* bytes read */ n) => {
                    let text = String::from_utf8_lossy(&buffer[..n]);

                    // Parse OSC sequences and update progress tracker
                    if let Ok(osc_event) = OscEvent::try_from(text.as_ref()) {
                        progress_tracker.handle_event(osc_event);
                    }

                    // Also display regular cargo output (optional)
                    // Commented out to focus on OSC sequences
                    // print!("{}", text);
                },
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for the build to complete using a blocking task
    // since child.wait() is a blocking operation
    let status = tokio::task::spawn_blocking(move || child.wait())
        .await
        .into_diagnostic()?
        .into_diagnostic()?;

    // Wait for the reader task to complete
    handle.await.into_diagnostic()?;

    println!("\n✅ Build completed with status: {}", status);
    Ok(())
}

```

### Running the POC

1. **Create a new Rust project:**

   ```bash
   cargo new pty_cargo_build_progress_osc_codes
   cd pty_cargo_build_progress_osc_codes
   ```

2. **Update `Cargo.toml`:**

   ```toml
   [package]
   name = "pty_cargo_build_progress_osc_codes"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   portable-pty = "0.8"
   tokio = { version = "1.0", features = ["full"] }
   miette = { version = "7.2", features = ["fancy"] }
   ```

3. **Replace `src/main.rs` with the code above**

4. **Run the POC:**
   ```bash
   cargo run
   ```

### Expected Output

When running in a project with dependencies, you should see output like:

```
Starting Cargo build with OSC capture...

📊 Build progress: 0.0%
📊 Build progress: 12.5%
📊 Build progress: 25.0%
📊 Build progress: 37.5%
📊 Build progress: 50.0%
📊 Build progress: 75.0%
📊 Build progress: 100.0%

✓ Progress tracking cleared
✅ Build completed with status: exit status: 0
```

### Troubleshooting

1. **No OSC sequences detected:**

   - Ensure the project being built has enough compilation units to trigger progress
     reporting
   - Try building a larger project or one with dependencies
   - Verify the TERM_PROGRAM environment variable is set correctly

2. **Build output mixed with progress:**

   - Uncomment the `print!("{}", text);` line to see full cargo output
   - OSC sequences are embedded in the normal output stream

3. **Permission errors:**
   - Some systems may require additional permissions for PTY creation
   - The portable-pty crate handles most platform differences automatically

## Environment Considerations

- **CI/CD**: Progress bars (and OSC) disabled when `CI` env var is set
- **Quiet mode**: No progress or OSC when `--quiet` flag used
- **TERM=dumb**: Progress disabled for dumb terminals
- **Non-TTY**: OSC only emitted when stderr is a TTY

## Testing

To test OSC emission:

1. Set `TERM_PROGRAM=WezTerm` (or use actual WezTerm)
2. Run `cargo build` on a large project
3. Monitor terminal title bar for progress updates
4. Or capture stderr and look for `\x1b]9;4;` sequences

## References

- [ConEmu OSC Documentation](https://conemu.github.io/en/AnsiEscapeCodes.html#ConEmu_specific_OSC)
- [OSC Escape Codes Overview](https://blog.vucica.net/2017/07/what-are-osc-terminal-control-sequences-escape-codes.html)
- [ANSI Escape Code Wikipedia](https://en.wikipedia.org/wiki/ANSI_escape_code)
