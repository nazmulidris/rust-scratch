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
  cursor::{self},
  event::*,
  queue,
  terminal::{self, *},
};
use paste::paste;
use r3bl_rs_utils::*;
use std::io::{stdout, Write};

/// If set to true, and the [log!] fails, then it will print the error to stderr.
const DEBUG: bool = true;

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

/// This works together w/ [CrosstermCmd] to enqueue commands, and then flush them at the
/// end.
macro_rules! enqueue_and_flush {
  ($it: block) => {{
    $it
    CrosstermCmd::flush()
  }};
}

/// To use this, you need to make sure to create an instance using `start()` (which
/// enables raw mode) and then when this instance is dropped (when the enclosing code
/// block falls out of scope) raw mode will be disabled.
pub struct RawMode;

impl RawMode {
  pub fn start() -> Self {
    enqueue_and_flush!({
      CrosstermCmd::try_to_enable_raw_mode();
      CrosstermCmd::try_to_enable_mouse_capture();
      CrosstermCmd::try_to_enter_alternate_screen();
      CrosstermCmd::reset_screen();
    });
    RawMode
  }
}

impl Drop for RawMode {
  fn drop(&mut self) {
    enqueue_and_flush!({
      CrosstermCmd::try_to_leave_alternate_screen();
      CrosstermCmd::try_to_disable_mouse_mode();
      CrosstermCmd::try_to_disable_raw_mode();
    });
  }
}

/// Given a crossterm command, this will run it and [log!] the [Result] that is returned.
/// If [log!] fails, then it will print a message to stderr.
///
/// https://github.com/dtolnay/paste
macro_rules! try_to_run_crossterm_command_and_log_result {
  ($cmd: expr, $name: ident) => {{
    paste! {
      // Generate a new function that returns [CommonResult].
      pub fn [<_ $name>]() -> CommonResult<()> {
        throws!({
          if let Err(err) = $cmd {
            log!(ERROR, "crossterm: ❌ Failed to {} due to {}", stringify!($name), err);
          } else {
            log!(INFO, "crossterm: ✅ {} successfully", stringify!($name));
          }
        })
      }

      // Call this generated function. It will fail if there are problems w/ log!().
      // In this case, if DEBUG is true, then it will dump the error to stderr.
      if let Err(err) = [<_ $name>]() {
        let msg = format!("❌ Failed to {}", stringify!($name));
        call_if_true!(DEBUG,
          debug!(ERROR_RAW &msg, err)
        );

      }
    }
  }};
}

/// Contains convenience associated functions to make it easier to create crossterm
/// commands. All of these associated functions use the
/// [try_to_run_crossterm_command_and_log_result!] macro. And they also enqueue commands
/// so make sure to flush them at the end or just use this macro [enqueue_and_flush!].
struct CrosstermCmd;

impl CrosstermCmd {
  fn try_to_enable_raw_mode() {
    try_to_run_crossterm_command_and_log_result! {
      terminal::enable_raw_mode(),
      enable_raw_mode
    };
  }

  fn try_to_enable_mouse_capture() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), EnableMouseCapture),
      enable_mouse_capture
    };
  }

  fn try_to_enter_alternate_screen() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), EnterAlternateScreen),
      enter_alternate_screen
    };
  }

  fn try_to_leave_alternate_screen() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), LeaveAlternateScreen),
      leave_alternate_screen
    };
  }

  fn try_to_disable_raw_mode() {
    try_to_run_crossterm_command_and_log_result! {
      terminal::disable_raw_mode(),
      disable_raw_mode
    };
  }

  fn try_to_disable_mouse_mode() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), DisableMouseCapture) ,
      disable_mouse_mode
    };
  }

  fn reset_cursor_position() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), cursor::MoveTo(0, 0)),
      reset_cursor_position
    };
  }

  fn clear_screen() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), terminal::Clear(ClearType::All)),
      clear_screen
    };
  }

  fn reset_screen() {
    CrosstermCmd::reset_cursor_position();
    CrosstermCmd::clear_screen();
  }

  fn flush() {
    try_to_run_crossterm_command_and_log_result! {
      stdout().flush(),
      flush
    };
  }
}
