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

use crate::*;
use crossterm::{
  cursor::{self},
  event::*,
  queue,
  terminal::{self, *},
};
use paste::paste;
use r3bl_rs_utils::*;
use std::io::{stdout, Write};

/// Given a crossterm command, this will run it and [log!] the [Result] that is returned.
/// If [log!] fails, then it will print a message to stderr.
///
/// https://github.com/dtolnay/paste
#[macro_export]
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

/// This works together w/ [CrosstermCmd] to enqueue commands, and then flush them at the
/// end.
#[macro_export]
macro_rules! enqueue_and_flush {
  ($it: block) => {{
    $it
    CrosstermCmd::flush()
  }};
}

/// Contains convenience associated functions to make it easier to create crossterm
/// commands. All of these associated functions use the
/// [try_to_run_crossterm_command_and_log_result!] macro. And they also enqueue commands
/// so make sure to flush them at the end or just use this macro [enqueue_and_flush!].
pub struct CrosstermCmd;

// TODO: think about being able to pass cmds around in a Vec rather than making direct calls.
// pub enum CrosstermCmd {
//   enable_raw_mode,
//   enable_mouse_capture,
//   enter_alternate_screen,
//   leave_alternate_screen,
//   disable_raw_mode,
//   disable_mouse_mode,
//   reset_cursor_position,
//   clear_screen
// }

impl CrosstermCmd {
  pub fn try_to_enable_raw_mode() {
    try_to_run_crossterm_command_and_log_result! {
      terminal::enable_raw_mode(),
      enable_raw_mode
    };
  }

  pub fn try_to_enable_mouse_capture() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), EnableMouseCapture),
      enable_mouse_capture
    };
  }

  pub fn try_to_enter_alternate_screen() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), EnterAlternateScreen),
      enter_alternate_screen
    };
  }

  pub fn try_to_leave_alternate_screen() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), LeaveAlternateScreen),
      leave_alternate_screen
    };
  }

  pub fn try_to_disable_raw_mode() {
    try_to_run_crossterm_command_and_log_result! {
      terminal::disable_raw_mode(),
      disable_raw_mode
    };
  }

  pub fn try_to_disable_mouse_mode() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), DisableMouseCapture) ,
      disable_mouse_mode
    };
  }

  pub fn reset_cursor_position() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), cursor::MoveTo(0, 0)),
      reset_cursor_position
    };
  }

  pub fn clear_screen() {
    try_to_run_crossterm_command_and_log_result! {
      queue!(stdout(), terminal::Clear(ClearType::All)),
      clear_screen
    };
  }

  pub fn reset_screen() {
    CrosstermCmd::reset_cursor_position();
    CrosstermCmd::clear_screen();
  }

  pub fn flush() {
    try_to_run_crossterm_command_and_log_result! {
      stdout().flush(),
      flush
    };
  }
}
