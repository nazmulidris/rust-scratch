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

use crossterm::event::{read, KeyEvent};
use r3bl_rs_utils::{debug, CommonResult};
use tui_layout_crossterm::*;

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
