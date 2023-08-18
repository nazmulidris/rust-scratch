/*
 *   Copyright (c) 2023 R3BL LLC
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

use std::fmt;

use crate::RgbColor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedString {
    pub text: String,
    pub foreground_color: RgbColor,
    pub background_color: RgbColor,
}

// https://doc.rust-lang.org/std/fmt/trait.Display.html
impl fmt::Display for FormattedString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "(text: {}, fg: {},{},{}, bg: {},{},{})",
            self.text,
            self.foreground_color.red,
            self.foreground_color.green,
            self.foreground_color.blue,
            self.background_color.red,
            self.background_color.green,
            self.background_color.blue
        )
    }
}

#[test]
fn test_formatted_string_creation() -> Result<(), String> {
    let eg_1 = FormattedString {
        text: "Hello".to_string(),
        foreground_color: RgbColor {
            red: 0,
            green: 0,
            blue: 0,
        },
        background_color: RgbColor {
            red: 1,
            green: 1,
            blue: 1,
        },
    };

    assert_eq!(
        format!("{0}", eg_1),
        "(text: Hello, fg: 0,0,0, bg: 1,1,1)".to_string()
    );

    Ok(())
}
