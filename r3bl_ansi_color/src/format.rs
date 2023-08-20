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

use crate::*;

pub struct FormattedString<'a> {
    pub text: &'a str,
    pub foreground: Color,
    pub background: Color,
}

impl<'a> FormattedString<'a> {
    pub fn foreground_color(&self) -> &Color {
        &self.foreground
    }
}

mod formatted_string_impl {
    use crate::{
        color_support_override_get, ColorSupportOverride, FormattedString, TransformColor,
    };
    use std::fmt::{Display, Formatter, Result};

    // https://doc.rust-lang.org/std/fmt/trait.Display.html
    impl Display for FormattedString<'_> {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            let color_support_override =
                if color_support_override_get() == ColorSupportOverride::NotSet {
                    if concolor_query::truecolor() {
                        ColorSupportOverride::Truecolor
                    } else {
                        ColorSupportOverride::Ansi256
                    }
                } else {
                    color_support_override_get()
                };

            // TODO: replace w/ ANSI color codes emitted to stdout
            match color_support_override {
                ColorSupportOverride::Ansi256 => {
                    // ANSI 256 color mode.
                    let fg = self.foreground.as_ansi256();
                    let bg = self.background.as_ansi256();
                    write!(
                        formatter,
                        "(text: {}, fg: {}, bg: {})",
                        self.text, fg.index, bg.index
                    )
                }
                _ => {
                    // True color mode.
                    let fg = self.foreground.as_rgb();
                    let bg = self.background.as_rgb();
                    write!(
                        formatter,
                        "(text: {}, fg: {},{},{}, bg: {},{},{})",
                        self.text, fg.red, fg.green, fg.blue, bg.red, bg.green, bg.blue
                    )
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::*;

        #[test]
        fn test_formatted_string_creation_ansi256() -> Result<(), String> {
            color_support_override_set(ColorSupportOverride::Ansi256);
            let eg_1 = FormattedString {
                text: "Hello",
                foreground: Color::Rgb(0, 0, 0),
                background: Color::Rgb(1, 1, 1),
            };

            assert_eq!(
                format!("{0}", eg_1),
                "(text: Hello, fg: 16, bg: 16)".to_string()
            );

            let eg_2 = FormattedString {
                text: "World",
                foreground: Color::Ansi256(150),
                background: Color::Rgb(1, 1, 1),
            };

            assert_eq!(
                format!("{0}", eg_2),
                "(text: World, fg: 150, bg: 16)".to_string()
            );

            Ok(())
        }

        #[test]
        fn test_formatted_string_creation_truecolor() -> Result<(), String> {
            color_support_override_set(ColorSupportOverride::Truecolor);
            let eg_1 = FormattedString {
                text: "Hello",
                foreground: Color::Rgb(0, 0, 0),
                background: Color::Rgb(1, 1, 1),
            };

            assert_eq!(
                format!("{0}", eg_1),
                "(text: Hello, fg: 0,0,0, bg: 1,1,1)".to_string()
            );

            let eg_2 = FormattedString {
                text: "World",
                foreground: Color::Ansi256(150),
                background: Color::Rgb(1, 1, 1),
            };

            assert_eq!(
                format!("{0}", eg_2),
                "(text: World, fg: 175,215,135, bg: 1,1,1)".to_string()
            );

            Ok(())
        }
    }
}
