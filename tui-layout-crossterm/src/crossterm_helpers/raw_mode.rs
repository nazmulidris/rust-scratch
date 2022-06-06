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

use bitflags::bitflags;
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute, terminal,
};
use std::io::stdout;

const DEBUG: bool = true;

/// If DEBUG is set to true, then print OK or ERROR message to stdout.
macro_rules! println_raw_if_debug {
  (ERROR $msg:expr, $err:expr) => {
    if DEBUG {
      println!(
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
#[macro_export]
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
}

/// To use this, you need to make sure to create an instance using `default()` (which
/// enables raw mode) and then when this instance is dropped (when code_block falls out of
/// scope) raw mode will be disabled.
/// https://github.com/crossterm-rs/crossterm/blob/master/examples/event-poll-read.rs
pub struct RawMode;

bitflags! {
  /// https://docs.rs/bitflags/0.8.2/bitflags/macro.bitflags.html
  pub struct Status: u8 {
    const RAW_MODE_ENABLED       = 0b0000_0001;
    const RAW_MODE_DISABLED      = 0b1000_0000;
    const MOUSE_CAPTURE_ENABLED  = 0b0000_0010;
    const MOUSE_CAPTURE_DISABLED = 0b0100_0000;
  }
}

impl RawMode {
  /// https://hermanradtke.com/2016/09/12/rust-using-and_then-and-map-combinators-on-result-type.html/
  pub fn start() -> Self {
    let mut status = Status::empty();

    // Try enable raw mode.
    terminal::enable_raw_mode()
      .and_then(|_| {
        status.insert(Status::RAW_MODE_ENABLED);
        Ok(())
      })
      .unwrap_or_else(|e| {
        println_raw_if_debug! {ERROR "crossterm: Failed to enable raw mode mode due to {}", e};
      });

    // Try enable mouse capture.
    execute!(stdout(), EnableMouseCapture)
      .and_then(|_| {
        status.insert(Status::MOUSE_CAPTURE_ENABLED);
        Ok(())
      })
      .unwrap_or_else(|e| {
        println_raw_if_debug! {ERROR "crossterm: Failed to enable mouse capture due to {}", e};
      });

    if status == Status::MOUSE_CAPTURE_ENABLED | Status::RAW_MODE_ENABLED {
      println_raw_if_debug!(OK "✅ Raw mode enabled & ✅ Mouse capture enabled.");
    } else if status == Status::MOUSE_CAPTURE_ENABLED {
      println_raw_if_debug!(OK "✅ Mouse capture enabled.");
    } else if status == Status::RAW_MODE_ENABLED {
      println_raw_if_debug!(OK "✅ Raw mode enabled.");
    }

    RawMode
  }
}

impl Drop for RawMode {
  /// https://hermanradtke.com/2016/09/12/rust-using-and_then-and-map-combinators-on-result-type.html/
  fn drop(&mut self) {
    let mut status = Status::empty();

    // Try disable raw mode.
    terminal::disable_raw_mode()
      .and_then(|_| {
        status.insert(Status::RAW_MODE_DISABLED);
        Ok(())
      })
      .unwrap_or_else(|e| {
        println_raw_if_debug! {ERROR "crossterm: Failed to disable raw mode mode due to {}", e};
      });

    // Try disable mouse capture.
    execute!(stdout(), DisableMouseCapture)
      .and_then(|_| {
        status.insert(Status::MOUSE_CAPTURE_DISABLED);
        Ok(())
      })
      .unwrap_or_else(|e| {
        println_raw_if_debug! {ERROR "crossterm: Failed to disable mouse capture due to {}", e};
      });

    if status == Status::MOUSE_CAPTURE_DISABLED | Status::RAW_MODE_DISABLED {
      println_raw_if_debug!(OK "✅ Raw mode disabled & ✅ Mouse capture disabled.");
    } else if status == Status::MOUSE_CAPTURE_DISABLED {
      println_raw_if_debug!(OK "✅ Mouse capture disabled.");
    } else if status == Status::RAW_MODE_DISABLED {
      println_raw_if_debug!(OK "✅ Raw mode disabled.");
    }
  }
}
