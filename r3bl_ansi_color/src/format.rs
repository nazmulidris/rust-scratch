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

use crate::TransformColor;

pub struct FormattedString<'a> {
    pub text: &'a str,
    pub foreground_color: &'a dyn TransformColor,
    pub background_color: &'a dyn TransformColor,
}

// https://doc.rust-lang.org/std/fmt/trait.Display.html
impl fmt::Display for FormattedString<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fg = self.foreground_color.as_rgb();
        let bg = self.background_color.as_rgb();

        // TODO: replace w/ ANSI color codes emitted to stdout
        write!(
            formatter,
            "(text: {}, fg: {},{},{}, bg: {},{},{})",
            self.text, fg.red, fg.green, fg.blue, bg.red, bg.green, bg.blue
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::FormattedString;

    #[test]
    fn test_formatted_string_creation() -> Result<(), String> {
        use crate::RgbColor;

        let eg_1 = FormattedString {
            text: "Hello",
            foreground_color: &RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
            background_color: &RgbColor {
                red: 1,
                green: 1,
                blue: 1,
            },
        };

        assert_eq!(
            format!("{0}", eg_1),
            "(text: Hello, fg: 0,0,0, bg: 1,1,1)".to_string()
        );

        let eg_2 = FormattedString {
            text: "World",
            foreground_color: &crate::Ansi256Color { index: 150 },
            background_color: &RgbColor {
                red: 1,
                green: 1,
                blue: 1,
            },
        };

        assert_eq!(
            format!("{0}", eg_2),
            "(text: World, fg: 175,215,135, bg: 1,1,1)".to_string()
        );

        Ok(())
    }
}
