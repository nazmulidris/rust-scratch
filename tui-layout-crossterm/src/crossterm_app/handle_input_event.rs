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
use tui_layout_crossterm::*;

/// Returns `true` if user presses any of the keys in [EXIT_KEYS]. Otherwise, returns `false`.
pub async fn handle_input_event(
  input_event: InputEvent,
  state: &mut State,
) -> LoopContinuation {
  match input_event {
    InputEvent::NonDisplayableKeypress(key_event) => match should_exit(&key_event) {
      true => return LoopContinuation::Exit,
      _ => {
        let KeyEvent { modifiers, code } = key_event;
        log_no_err!(INFO, "KeyEvent: {:?} + {:?}", modifiers, code);
      }
    },

    InputEvent::DisplayableKeypress(character) => log_no_err!(INFO, "DisplayableKeypress: {:?}", character),

    InputEvent::Resize(size) => on_resize(size, state),

    InputEvent::Mouse(mouse_event) => log_no_err!(INFO, "Mouse: {:?}", mouse_event),

    _ => log_no_err!(INFO, "Other: {:?}", input_event),
  }

  return LoopContinuation::Continue;
}

fn on_resize(
  size: Size,
  state: &mut State,
) {
  state.terminal_size = size;
  log_no_err!(INFO, "Resize: {:?}", (size.height, size.width));
  call_if_true!(DEBUG, state.dump_to_log("Resize"));
}

fn should_exit(key_event: &KeyEvent) -> bool {
  EXIT_KEYS.contains(&key_event)
}

pub enum LoopContinuation {
  Exit,
  Continue,
}

/// Array of [KeyEvent]s that the user can press to exit the REPL.
const EXIT_KEYS: [crossterm::event::KeyEvent; 1] = [KeyEvent {
  code: KeyCode::Char('q'),
  modifiers: KeyModifiers::CONTROL,
}];
