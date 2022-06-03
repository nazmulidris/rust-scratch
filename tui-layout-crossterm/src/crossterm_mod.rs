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

use crossterm::event::{
  read,
  Event::{Key, Mouse, Resize},
  KeyCode, KeyEvent, KeyModifiers, MouseEvent,
};
use r3bl_rs_utils::{debug, CommonResult};
use tui_layout_crossterm::{println_raw, raw_mode};

pub async fn emit_crossterm_commands() -> CommonResult<()> {
  return raw_mode!({
    repl().await?;
    Ok(())
  });
}

async fn repl() -> CommonResult<()> {
  println_raw!("Type Ctrl+q to exit repl.");
  loop {
    let state = InputEvent::read_crossterm_event_to_state().await?;
    match state {
      InputEvent::Exit => break,

      InputEvent::InputKeyEvent(key_event) => {
        let KeyEvent { modifiers, code } = key_event;
        let msg = format!("{:?} + {:?}", modifiers, code);
        println_raw!(msg);
      }

      InputEvent::InputNormalChar(character) => {
        println_raw!(character);
      }

      InputEvent::Resize(rows, cols) => {
        debug!(rows, cols);
      }

      InputEvent::InputMouseEvent(mouse_event) => {
        debug!(mouse_event);
      }
    }
  }
  Ok(())
}

enum InputEvent {
  Exit,
  InputNormalChar(char),
  InputKeyEvent(KeyEvent),
  /// first: rows, second: cols
  Resize(u16, u16),
  InputMouseEvent(MouseEvent),
}

impl InputEvent {
  async fn read_crossterm_event_to_state() -> CommonResult<InputEvent> {
    match read()? {
      Key(key_event) => InputEvent::handle_key_event(key_event),
      Mouse(mouse_event) => InputEvent::handle_mouse_event(mouse_event),
      Resize(cols, rows) => InputEvent::handle_resize_event(rows, cols),
    }
  }

  fn handle_key_event(key_event: KeyEvent) -> CommonResult<InputEvent> {
    match key_event {
      KeyEvent {
        code: KeyCode::Char(character),
        modifiers: KeyModifiers::NONE,
      } => Ok(InputEvent::InputNormalChar(character)),

      KeyEvent {
        code: KeyCode::Char(character),
        modifiers: KeyModifiers::CONTROL,
      } if character == 'q' => Ok(InputEvent::Exit),

      _ => Ok(InputEvent::InputKeyEvent(key_event)),
    }
  }

  fn handle_resize_event(
    rows: u16,
    cols: u16,
  ) -> CommonResult<InputEvent> {
    Ok(InputEvent::Resize(rows, cols))
  }

  fn handle_mouse_event(mouse_event: MouseEvent) -> CommonResult<InputEvent> {
    Ok(InputEvent::InputMouseEvent(mouse_event))
  }
}
