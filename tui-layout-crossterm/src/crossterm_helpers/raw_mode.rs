/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

use crossterm::{
  event::*,
  execute,
  terminal::{self, *},
};
use paste::paste;
use r3bl_rs_utils::*;
use std::io::stdout;

/// If set to true, and the [log!] fails, then it will print the error to stderr.
const DEBUG: bool = true;

/// If DEBUG is set to true, then print OK or ERROR message to stdout.
macro_rules! println_raw_if_debug {
  (ERROR $msg:expr, $err:expr) => {
    if DEBUG {
      eprintln!(
        "{} {} {}\r",
        r3bl_rs_utils::style_error("▶"),
        r3bl_rs_utils::style_prompt($msg),
        r3bl_rs_utils::style_dimmed(&format!("{:#?}", $err))
      );
    }
  };
  (OK $msg:expr) => {
    if DEBUG {
      println!(
        "{} {}\r",
        r3bl_rs_utils::style_error("▶"),
        r3bl_rs_utils::style_prompt($msg),
      );
    }
  };
}

/// Simply print message to stdout.
macro_rules! println_raw {
  ($arg:tt) => {
    println!("{}\r", $arg)
  };
}

/// This will automatically disable [raw
/// mode](https://docs.rs/crossterm/0.23.2/crossterm/terminal/index.html#raw-mode) when
/// the enclosed block ends. Note that this macro must be called from a function that
/// returns a `Result`.
///
/// Example 1:
/// ```ignore
/// pub async fn emit_crossterm_commands() -> CommonResult<()> {
///   raw_mode! { repl().await? }
/// }
/// ```
///
/// Example 2:
/// ```ignore
/// pub async fn emit_crossterm_commands() -> CommonResult<()> {
///   raw_mode!({
///     repl().await?;
///     Ok(())
///   })
/// }
/// ```
///
/// Example 3:
/// ```ignore
/// pub async fn emit_crossterm_commands() -> CommonResult<()> {
///   raw_mode!({
///     println!("crossterm: Entering raw mode...");
///     repl().await?;
///     println!("crossterm: Exiting raw mode...");
///     return Ok(());
///   });
/// }
/// ```
#[macro_export]
macro_rules! raw_mode {
  ($code_block: stmt) => {{
    use crate::*;
    let _raw_mode = RawMode::start();
    $code_block
    Ok(())
  }};
  ($code_block: block) => {{
    use crate::*;
    let _raw_mode = RawMode::start();
    $code_block
    Ok(())
  }};
}

/// To use this, you need to make sure to create an instance using `default()` (which
/// enables raw mode) and then when this instance is dropped (when code_block falls out of
/// scope) raw mode will be disabled.
/// https://github.com/crossterm-rs/crossterm/blob/master/examples/event-poll-read.rs
pub struct RawMode;

/// Given a crossterm command, this will run it and [log!] the [Result] that is returned.
/// The caller must return a [CommonResult], since it uses `?`.
macro_rules! try_to_run_crossterm_command_and_log_result {
  ($cmd: expr, $description: expr, $name: ident) => {{
    paste! {
      pub fn [<_ $name>]() -> CommonResult<()> {
        if let Err(err) = $cmd {
          log!(ERROR, "crossterm: Failed to {} due to {}", $description, err);
        } else {
          log!(INFO, $description);
        }
        Ok(())
      }
      if let Err(err) = [<_ $name>]() {
        let msg = format!("❌ Failed to {}", stringify!($name));
        println_raw_if_debug!(ERROR &msg, err);
      }
    }
  }};
}

/// Each associated method must wrap the call to
/// [try_to_run_crossterm_command_and_log_result] which calls [log!] (since it uses `?`)
/// in a function that returns a [CommonResult].
struct CrosstermCmd;

impl CrosstermCmd {
  fn try_to_enable_raw_mode() {
    try_to_run_crossterm_command_and_log_result! {
      terminal::enable_raw_mode(),
      "🍣 enable raw mode",
      enable_raw_mode
    };
  }

  fn try_to_enable_mouse_capture() {
    try_to_run_crossterm_command_and_log_result! {
      execute!(stdout(), EnableMouseCapture),
      "🐭 enable mouse capture",
      enable_mouse_capture
    };
  }

  fn try_to_enter_alternate_screen() {
    try_to_run_crossterm_command_and_log_result! {
      execute!(stdout(), EnterAlternateScreen),
      "🌒 enter alternate screen",
      enter_alternate_screen
    };
  }

  fn try_to_leave_alternate_screen() {
    try_to_run_crossterm_command_and_log_result! {
      execute!(stdout(), LeaveAlternateScreen),
      "🌒 leave alternate screen",
      leave_alternate_screen
    };
  }

  fn try_to_disable_raw_mode() {
    try_to_run_crossterm_command_and_log_result! {
      terminal::disable_raw_mode(),
      "🍣 disable raw mode",
      disable_raw_mode
    };
  }

  fn try_to_disable_mouse_mode() {
    try_to_run_crossterm_command_and_log_result! {
      execute!(stdout(), DisableMouseCapture) ,
      "🐭 disable mouse mode",
      disable_mouse_mode
    };
  }
}

impl RawMode {
  pub fn start() -> Self {
    CrosstermCmd::try_to_enable_raw_mode();
    CrosstermCmd::try_to_enable_mouse_capture();
    CrosstermCmd::try_to_enter_alternate_screen();

    RawMode
  }
}

impl Drop for RawMode {
  fn drop(&mut self) {
    CrosstermCmd::try_to_leave_alternate_screen();
    CrosstermCmd::try_to_disable_mouse_mode();
    CrosstermCmd::try_to_disable_raw_mode();
  }
}
