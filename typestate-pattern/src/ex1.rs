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

//! Simple version of this is using enums to encapsulate states as variants.
//!
//! Real example:
//! - <https://github.com/r3bl-org/r3bl-open-core/blob/main/tui/src/tui/terminal_lib_backends/input_event.rs>.
//!
//! Limitations are:
//! - You can't create methods that are specific to each variant.
//! - Methods apply to all variants.

#[derive(Debug)]
pub enum InputEvent {
    Keyboard((KeyPress, Option<Vec<Modifier>>)),
    Resize(Size),
    Mouse(MouseEvent),
}

#[derive(Debug)]
pub enum Modifier {
    Shift,
    Control,
    Alt,
}

#[derive(Debug)]
pub enum KeyPress {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    F(u8),
}

#[derive(Debug)]
pub enum Size {
    Height(u16),
    Width(u16),
}

#[derive(Debug)]
pub enum MouseEvent {
    Press(MouseButton, u16, u16),
    Release(u16, u16),
    Hold(u16, u16),
}

#[derive(Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl InputEvent {
    pub fn pretty_print(&self) {
        let it = match self {
            InputEvent::Keyboard((keypress, modifiers)) => {
                let mut result = format!("{:?}", keypress);
                if let Some(modifiers) = modifiers {
                    result.push_str(&format!("{:?}", modifiers));
                }
                result
            }
            InputEvent::Resize(size) => format!("{:?}", size),
            InputEvent::Mouse(mouse_event) => format!("{:?}", mouse_event),
        };
        println!("{}", it);
    }
}

fn main() {
    let a_pressed = InputEvent::Keyboard((KeyPress::Char('a'), None));
    println!("{:?}", a_pressed);

    let ctrl_c_pressed = InputEvent::Keyboard((KeyPress::Char('c'), Some(vec![Modifier::Control])));
    println!("{:?}", ctrl_c_pressed);

    let enter_pressed = InputEvent::Keyboard((KeyPress::Enter, None));
    enter_pressed.pretty_print();

    let mouse_pressed = InputEvent::Mouse(MouseEvent::Press(MouseButton::Left, 10, 20));
    mouse_pressed.pretty_print();
}
