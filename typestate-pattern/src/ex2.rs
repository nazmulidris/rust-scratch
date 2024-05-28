/*
 *   Copyright (c) 2024 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

//! Slightly more complex versions are where one type + data = another type.
//!
//! Real examples:
//! - <https://github.com/r3bl-org/r3bl-open-core/blob/main/tui/src/tui/terminal_lib_backends/input_event.rs#L87>.
//! - <https://github.com/r3bl-org/r3bl-open-core/blob/main/tui/src/tui/editor/editor_component/editor_event.rs#L74>.
//!
//! There are many advantages to this approach, such as having a event based API for
//! editor or just regular function calls. You can see this in `EditorEvent`.
//!
//! Same limitations as `ex1.rs`:
//! - You can't create methods that are specific to each variant.
//! - Methods apply to all variants.

mod ex1;
use ex1::InputEvent;

#[derive(Debug)]
pub enum EditorEvent {
    InsertChar(char),
    InsertNewLine,
    Delete,
    Backspace,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorUp,
    MoveCursorDown,
    Copy,
    Paste,
    Cut,
    Undo,
    Redo,
}

impl TryFrom<InputEvent> for EditorEvent {
    type Error = String;

    fn try_from(input_event: InputEvent) -> Result<Self, Self::Error> {
        match input_event {
            InputEvent::Keyboard((keypress, modifiers)) => match (keypress, modifiers) {
                (ex1::KeyPress::Char(ch), None) => Ok(Self::InsertChar(ch)),
                (ex1::KeyPress::Char(_), Some(_)) => todo!(),
                (ex1::KeyPress::Enter, None) => Ok(Self::InsertNewLine),
                (ex1::KeyPress::Enter, Some(_)) => todo!(),
                (ex1::KeyPress::Backspace, None) => todo!(),
                (ex1::KeyPress::Backspace, Some(_)) => todo!(),
                (ex1::KeyPress::Delete, None) => todo!(),
                (ex1::KeyPress::Delete, Some(_)) => todo!(),
                (ex1::KeyPress::Left, None) => todo!(),
                (ex1::KeyPress::Left, Some(_)) => todo!(),
                (ex1::KeyPress::Right, None) => todo!(),
                (ex1::KeyPress::Right, Some(_)) => todo!(),
                (ex1::KeyPress::Up, None) => todo!(),
                (ex1::KeyPress::Up, Some(_)) => todo!(),
                (ex1::KeyPress::Down, None) => todo!(),
                (ex1::KeyPress::Down, Some(_)) => todo!(),
                (ex1::KeyPress::Home, None) => todo!(),
                (ex1::KeyPress::Home, Some(_)) => todo!(),
                (ex1::KeyPress::End, None) => todo!(),
                (ex1::KeyPress::End, Some(_)) => todo!(),
                (ex1::KeyPress::PageUp, None) => todo!(),
                (ex1::KeyPress::PageUp, Some(_)) => todo!(),
                (ex1::KeyPress::PageDown, None) => todo!(),
                (ex1::KeyPress::PageDown, Some(_)) => todo!(),
                (ex1::KeyPress::Tab, None) => todo!(),
                (ex1::KeyPress::Tab, Some(_)) => todo!(),
                (ex1::KeyPress::F(_), None) => todo!(),
                (ex1::KeyPress::F(_), Some(_)) => todo!(),
            },
            InputEvent::Resize(_) => todo!(),
            InputEvent::Mouse(_) => todo!(),
        }
    }
}

fn main() {
    let a_pressed = InputEvent::Keyboard((ex1::KeyPress::Char('a'), None));
    println!("{:?}", EditorEvent::try_from(a_pressed));

    let enter_pressed = InputEvent::Keyboard((ex1::KeyPress::Enter, None));
    println!("{:?}", EditorEvent::try_from(enter_pressed));
}
