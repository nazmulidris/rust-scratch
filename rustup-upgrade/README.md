# PTY-Driven Rustup Update & Cargo Install with Live Spinner

This crate preserves the complete, generalized implementation of running long-running
compiler toolchain updates and Cargo installations inside a Pseudoterminal (PTY), parsing
real-time progress, and displaying it on an animated TUI `Spinner`.

## Key Techniques

1. **PTY Session Spawning**: Runs `rustup` and `cargo` inside a PTY via
   `r3bl_tui::core::pty::PtySessionBuilder` so the child processes believe they are
   attached to a real terminal.
2. **Rustup Output Parsing**: Parses streaming line-oriented stdout from `rustup`
   (`extract_rustup_progress`) to capture download and component installation steps
   (`downloading component 'rust-std'`).
3. **Cargo OSC 9;4 Progress Capture**: Enables `PtySessionConfigOption::CaptureOsc` so
   Cargo emits OSC 9;4 progress sequences during compilation (0-100%), updating the
   spinner live.
4. **Cooperative Cancellation**: Uses `tokio::select!` on `tokio::signal::ctrl_c()` to
   cleanly abort without leaving terminal state corrupted.

## Usage

```bash
cargo run -- r3bl-cmdr
```
