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

//! Using `poll()` is inefficient. The following code will generate some CPU utilization
//! while idling.
//!
//! ```ignore
//! loop {
//!   if poll(Duration::from_millis(500))? { // This is inefficient.
//!     let input_event: InputEvent = read()?.into();
//!     if handle_input_event(input_event).await.is_err() {
//!       break;
//!     };
//!   }
//! }
//! ```
//!
//! The following code blocks the thread that its running on.
//!
//! ```ignore
//! async fn repl_blocking() -> CommonResult<()> {
//!   throws!({
//!     println_raw!("Type Ctrl+q to exit repl.");
//!     loop {
//!       let input_event: InputEvent = read()?.into();
//!       let result = handle_input_event(input_event).await;
//!       if result.is_err() {
//!         break;
//!       };
//!     }
//!   });
//! }
//! ```
//!
//! Docs:
//! - https://github.com/crossterm-rs/crossterm/wiki/Upgrade-from-0.13-to-0.14#115-event-polling
//! - https://github.com/crossterm-rs/crossterm/wiki/Upgrade-from-0.13-to-0.14#111-new-event-api
//! - https://github.com/crossterm-rs/crossterm/blob/master/examples/event-stream-tokio.rs

use async_trait::async_trait;
use crossterm::event::*;
use futures_util::{FutureExt, StreamExt};
use r3bl_rs_utils::*;
use tui_layout_crossterm::*;

#[async_trait]
pub trait EventStreamExt {
  /// Read an [Event] from the [EventStream]. This is a non-blocking call.
  async fn get_event_from(&mut self) -> CommonResult<Option<InputEvent>>;
}

#[async_trait]
impl EventStreamExt for EventStream {
  async fn get_event_from(&mut self) -> CommonResult<Option<InputEvent>> {
    let option_result_event = self.next().fuse().await;
    match option_result_event {
      Some(Ok(event)) => Ok(Some(event.into())),

      Some(Err(e)) => {
        log!(ERROR, "Error: {:?}", e);
        Ok(None)
      }

      None => Ok(None),
    }
  }
}

pub async fn emit_crossterm_commands() -> CommonResult<()> {
  raw_mode! { repl_nonblocking().await? }
}

async fn repl_nonblocking() -> CommonResult<()> {
  throws!({
    let mut event_stream = EventStream::new();
    loop {
      let input_event = event_stream.get_event_from().await?;
      match input_event {
        Some(input_event) => {
          if handle_input_event(input_event).await? {
            break;
          }
        }

        None => break,
      }
    }
  })
}

/// Array of [KeyEvent]s that the user can press to exit the REPL.
const EXIT_KEYS: [crossterm::event::KeyEvent; 1] = [KeyEvent {
  code: KeyCode::Char('q'),
  modifiers: KeyModifiers::CONTROL,
}];

/// Returns false if user presses any of the keys in [EXIT_KEYS].
async fn handle_input_event(input_event: InputEvent) -> CommonResult<bool> {
  match input_event {
    // Check for REPL exit.
    InputEvent::NonDisplayableKeypress(key_event) => {
      if EXIT_KEYS.contains(&key_event) {
        return Ok(true);
      }
      let KeyEvent { modifiers, code } = key_event;
      log!(INFO, "KeyEvent: {:?} + {:?}", modifiers, code);
    }

    InputEvent::DisplayableKeypress(character) => {
      log!(INFO, "DisplayableKeypress: {:?}", character);
    }

    InputEvent::Resize(Size { height, width }) => {
      log!(INFO, "Resize: {:?}", (height, width));
    }

    InputEvent::Mouse(mouse_event) => {
      log!(INFO, "Mouse: {:?}", mouse_event);
    }

    _ => {
      log!(INFO, "Other: {:?}", input_event);
    }
  }
  Ok(false)
}
