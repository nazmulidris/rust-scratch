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

use crossterm::event::*;
use r3bl_rs_utils::*;
use tui_layout_crossterm::{EventStreamExt, *};

pub async fn emit_crossterm_commands() -> CommonResult<()> {
  raw_mode!({
    let mut event_stream = EventStream::new();
    loop {
      let maybe_input_event = event_stream.get_input_event().await;
      if let Some(input_event) = maybe_input_event {
        let should_exit = process_input_event(input_event).await;
        if should_exit {
          break;
        }
      }
    }
  })
}

/// Array of [KeyEvent]s that the user can press to exit the REPL.
const EXIT_KEYS: [crossterm::event::KeyEvent; 1] = [KeyEvent {
  code: KeyCode::Char('q'),
  modifiers: KeyModifiers::CONTROL,
}];

/// Returns true if user presses any of the keys in [EXIT_KEYS].
async fn process_input_event(input_event: InputEvent) -> bool {
  match input_event {
    InputEvent::NonDisplayableKeypress(key_event) => {
      // Check for REPL exit.
      if EXIT_KEYS.contains(&key_event) {
        return true;
      }
      let KeyEvent { modifiers, code } = key_event;
      log_no_err!(INFO, "KeyEvent: {:?} + {:?}", modifiers, code);
    }

    InputEvent::DisplayableKeypress(character) => log_no_err!(INFO, "DisplayableKeypress: {:?}", character),

    InputEvent::Resize(Size { height, width }) => log_no_err!(INFO, "Resize: {:?}", (height, width)),

    InputEvent::Mouse(mouse_event) => log_no_err!(INFO, "Mouse: {:?}", mouse_event),

    _ => log_no_err!(INFO, "Other: {:?}", input_event),
  }

  false
}
