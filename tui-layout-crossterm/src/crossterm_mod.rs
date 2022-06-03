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
  KeyCode, KeyEvent, KeyModifiers,
};
use r3bl_rs_utils::CommonResult;
use tui_layout_crossterm::{println_raw, raw_mode};

pub async fn emit_crossterm_commands() -> CommonResult<()> {
  println!("TODO: crossterm: Hello, world!");
  return raw_mode!({
    repl().await?;
    Ok(())
  });
}

async fn repl() -> CommonResult<()> {
  println_raw!("Type x to exit repl.");
  loop {
    let state = StdinState::read_crossterm_event_to_state().await?;
    match state {
      StdinState::NoInput => break,
      StdinState::InputNormalChar('x') => break,
      StdinState::InputControlChar(number) => {
        let msg = format!("CONTROL {}", number);
        println_raw!(msg);
      }
      StdinState::InputNormalChar(character) => {
        println_raw!(character);
      }
    }
  }
  Ok(())
}

enum StdinState {
  NoInput,
  InputControlChar(u8),
  InputNormalChar(char),
}

impl StdinState {
  async fn read_crossterm_event_to_state() -> CommonResult<StdinState> {
    match read()? {
      Key(key_event) => match key_event {
        KeyEvent {
          code: KeyCode::Char(character),
          modifiers: KeyModifiers::NONE,
        } => Ok(StdinState::InputNormalChar(character)),
        _ => todo!(),
      },
      Mouse(_) => todo!(),
      Resize(_, _) => todo!(),
    }
  }
}
