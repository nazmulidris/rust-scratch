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
use crossterm::event::*;
use r3bl_rs_utils::*;

pub struct TerminalWindow {
  pub terminal_size: Size,
  pub event_stream: EventStream,
}

impl TerminalWindow {
  fn new_instance() -> CommonResult<Self> {
    Ok(Self {
      event_stream: EventStream::new(),
      terminal_size: Size::try_to_get_from_crossterm_terminal()?,
    })
  }

  pub async fn start_event_loop<S>(mut_state: &mut S, box_draw: Box<dyn Draw<S>>) -> CommonResult<()>
  where
    S: Send + Sync,
  {
    raw_mode!({
      let mut terminal_window = TerminalWindow::new_instance()?;
      call_if_true!(DEBUG, terminal_window.dump_to_log("Startup"));

      loop {
        let maybe_input_event = terminal_window.event_stream.get_input_event().await;
        if let Some(input_event) = maybe_input_event {
          let loop_continuation = handle_input_event(input_event, &mut terminal_window).await;
          if let LoopContinuation::Exit = loop_continuation {
            break;
          } else {
            // TODO: replace this w/ something more meaningful.
            println!("hello world\r");
          }
        }
      }
    })
  }

  pub fn dump_to_log(&self, msg: &str) {
    log_no_err!(INFO, "{} -> {}", msg, self.to_string());
  }
}
