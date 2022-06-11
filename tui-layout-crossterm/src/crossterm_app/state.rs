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

use crossterm::{event::*, terminal::*};
use r3bl_rs_utils::*;
use std::fmt::{Debug, Display, Formatter};
use tui_layout_crossterm::*;

pub struct State {
  pub terminal_size: Size,
  pub event_stream: EventStream,
}

impl State {
  pub fn new() -> CommonResult<Self> {
    let mut retval = Self {
      event_stream: EventStream::new(),
      terminal_size: Size::try_to_get_from_crossterm_terminal()?,
    };
    retval.set_size(size()?);
    Ok(retval)
  }

  pub fn set_size(
    &mut self,
    size: (u16, u16),
  ) {
    self.terminal_size = Size::from(size);
  }

  pub fn dump_to_log(
    &self,
    msg: &str,
  ) {
    log_no_err!(INFO, "{} -> {:?}", msg, self);
  }
}

impl Display for State {
  fn fmt(
    &self,
    f: &mut Formatter<'_>,
  ) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Debug for State {
  fn fmt(
    &self,
    f: &mut Formatter<'_>,
  ) -> std::fmt::Result {
    f.debug_struct("State")
      .field("event_stream", &DebugDisplay::Ok)
      .field("terminal_size", &self.terminal_size)
      .finish()
  }
}

/// This is just for pretty printing the impl of [Debug] trait for [State].
enum DebugDisplay {
  Ok,
}

impl Debug for DebugDisplay {
  fn fmt(
    &self,
    f: &mut Formatter<'_>,
  ) -> std::fmt::Result {
    match self {
      Self::Ok => write!(f, "✅"),
    }
  }
}
