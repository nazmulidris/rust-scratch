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
  Event::{self, Key, Mouse, Resize},
  KeyCode, KeyEvent, KeyModifiers, MouseEvent,
};
use r3bl_rs_utils::{debug, CommonResult};
use tui_layout_crossterm::{println_raw, raw_mode, Size};

pub async fn emit_crossterm_commands() -> CommonResult<()> {
  return raw_mode!({
    repl().await?;
    Ok(())
  });
}

async fn repl() -> CommonResult<()> {
  println_raw!("Type Ctrl+q to exit repl.");

  loop {
    match read()?.into() {
      InputEvent::Exit => break,

      InputEvent::NonDisplayableKeypress(key_event) => {
        let KeyEvent { modifiers, code } = key_event;
        let msg = format!("InputKeyEvent: {:?} + {:?}", modifiers, code);
        println_raw!(msg);
      }

      InputEvent::DisplayableKeypress(character) => {
        println_raw!(character);
      }

      InputEvent::Resize(Size { height, width }) => {
        debug!(height, width);
      }

      InputEvent::Mouse(mouse_event) => {
        debug!(mouse_event);
      }
    }
  }

  Ok(())
}

pub enum InputEvent {
  Exit,
  DisplayableKeypress(char),
  NonDisplayableKeypress(KeyEvent),
  Resize(Size),
  Mouse(MouseEvent),
}

/// Typecast / convert [Event] to [InputEvent].
impl From<Event> for InputEvent {
  fn from(event: Event) -> Self {
    match event {
      Key(key_event) => key_event.into(),
      Mouse(mouse_event) => mouse_event.into(),
      Resize(cols, rows) => (rows, cols).into(),
    }
  }
}

/// Typecast / convert [(u16, u16)] to [InputEvent::TerminalSize].
impl From<(/* rows: */ u16, /* cols: */ u16)> for InputEvent {
  fn from(size: (u16, u16)) -> Self {
    let (rows, cols) = size;
    InputEvent::Resize(Size { width: cols, height: rows })
  }
}

/// Typecast / convert [MouseEvent] to [InputEvent::InputMouseEvent].
impl From<MouseEvent> for InputEvent {
  fn from(mouse_event: MouseEvent) -> Self {
    InputEvent::Mouse(mouse_event)
  }
}

/// Typecast / convert [KeyEvent] to [InputEvent::].
impl From<KeyEvent> for InputEvent {
  fn from(key_event: KeyEvent) -> Self {
    match key_event {
      // Check for `Ctrl + q` to exit.
      KeyEvent {
        code: KeyCode::Char(character),
        modifiers: KeyModifiers::CONTROL,
      } if character == 'q' => InputEvent::Exit,

      // Check if "normal character" is pressed.
      KeyEvent {
        code: KeyCode::Char(character),
        modifiers: KeyModifiers::NONE,
      } => InputEvent::DisplayableKeypress(character),

      // All other key presses.
      _ => InputEvent::NonDisplayableKeypress(key_event),
    }
  }
}
