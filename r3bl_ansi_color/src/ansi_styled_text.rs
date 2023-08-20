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

pub struct AnsiStyledText<'a> {
    pub text: &'a str,
    pub style: &'a [Style],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Foreground(Color),
    Background(Color),
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Invert,
    Hidden,
    Strike,
}

mod style_impl {
    use crate::{
        color_support_override_get, Color, ColorSupportOverride, RgbColor, SgrCode, Style,
        TransformColor,
    };
    use std::fmt::{Display, Formatter, Result};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ColorKind {
        Foreground,
        Background,
    }

    fn query_color_support_override() -> ColorSupportOverride {
        if color_support_override_get() == ColorSupportOverride::NotSet {
            if concolor_query::truecolor() {
                ColorSupportOverride::Truecolor
            } else {
                ColorSupportOverride::Ansi256
            }
        } else {
            color_support_override_get()
        }
    }

    fn fmt_color(color: Color, color_kind: ColorKind, f: &mut Formatter<'_>) -> Result {
        match query_color_support_override() {
            ColorSupportOverride::Ansi256 => {
                // ANSI 256 color mode.
                let color = color.as_ansi256();
                let index = color.index;
                write!(
                    f,
                    "{}",
                    match color_kind {
                        ColorKind::Foreground => SgrCode::ForegroundAnsi256(index),
                        ColorKind::Background => SgrCode::BackgroundAnsi256(index),
                    }
                )
            }
            _ => {
                // True color mode.
                let color = color.as_rgb();
                let RgbColor { red, green, blue } = color;
                write!(
                    f,
                    "{}",
                    match color_kind {
                        ColorKind::Foreground => SgrCode::ForegroundRGB(red, green, blue),
                        ColorKind::Background => SgrCode::BackgroundRGB(red, green, blue),
                    }
                )
            }
        }
    }

    impl Display for Style {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Style::Foreground(color) => fmt_color(*color, ColorKind::Foreground, f),
                Style::Background(color) => fmt_color(*color, ColorKind::Background, f),
                Style::Bold => write!(f, "{}", SgrCode::Bold),
                Style::Dim => write!(f, "{}", SgrCode::Dim),
                Style::Italic => write!(f, "{}", SgrCode::Italic),
                Style::Underline => write!(f, "{}", SgrCode::Underline),
                Style::Blink => write!(f, "{}", SgrCode::Blink),
                Style::Invert => write!(f, "{}", SgrCode::Invert),
                Style::Hidden => write!(f, "{}", SgrCode::Hidden),
                Style::Strike => write!(f, "{}", SgrCode::Strike),
            }
        }
    }
}

mod display_trait_impl {
    use crate::{AnsiStyledText, SgrCode};
    use std::fmt::{Display, Formatter, Result};

    // https://doc.rust-lang.org/std/fmt/trait.Display.html
    impl Display for AnsiStyledText<'_> {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
            let mut style_string_vec = vec![];
            for style_item in self.style {
                style_string_vec.push(style_item.to_string());
            }
            style_string_vec.push(self.text.to_string());
            style_string_vec.push(SgrCode::Reset.to_string());
            write!(formatter, "{}", style_string_vec.join(""))
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn test_formatted_string_creation_ansi256() -> Result<(), String> {
            color_support_override_set(ColorSupportOverride::Ansi256);
            let eg_1 = AnsiStyledText {
                text: "Hello",
                style: &[
                    Style::Bold,
                    Style::Foreground(Color::Rgb(0, 0, 0)),
                    Style::Background(Color::Rgb(1, 1, 1)),
                ],
            };

            assert_eq!(
                format!("{0}", eg_1),
                "\x1b[1m\x1b[38;5;16m\x1b[48;5;16mHello\x1b[0m".to_string()
            );

            let eg_2 = AnsiStyledText {
                text: "World",
                style: &[
                    Style::Bold,
                    Style::Foreground(Color::Ansi256(150)),
                    Style::Background(Color::Rgb(1, 1, 1)),
                ],
            };

            assert_eq!(
                format!("{0}", eg_2),
                "\x1b[1m\x1b[38;5;150m\x1b[48;5;16mWorld\x1b[0m".to_string()
            );

            Ok(())
        }

        #[test]
        fn test_formatted_string_creation_truecolor() -> Result<(), String> {
            color_support_override_set(ColorSupportOverride::Truecolor);
            let eg_1 = AnsiStyledText {
                text: "Hello",
                style: &[
                    Style::Bold,
                    Style::Foreground(Color::Rgb(0, 0, 0)),
                    Style::Background(Color::Rgb(1, 1, 1)),
                ],
            };

            assert_eq!(
                format!("{0}", eg_1),
                "\x1b[1m\x1b[38;2;0;0;0m\x1b[48;2;1;1;1mHello\x1b[0m".to_string()
            );

            let eg_2 = AnsiStyledText {
                text: "World",
                style: &[
                    Style::Bold,
                    Style::Foreground(Color::Ansi256(150)),
                    Style::Background(Color::Rgb(1, 1, 1)),
                ],
            };

            assert_eq!(
                format!("{0}", eg_2),
                "\x1b[1m\x1b[38;2;175;215;135m\x1b[48;2;1;1;1mWorld\x1b[0m".to_string()
            );

            Ok(())
        }
    }
}
