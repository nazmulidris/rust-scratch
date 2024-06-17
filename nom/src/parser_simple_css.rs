/*
 *   Copyright (c) 2023 Nazmul Idris
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

//! This module contains a parser that parses simple CSS like styling for hex colors.
//! The hex color string can be in the following formats:
//! 1. `#RRGGBB`, eg: `#FF0000` for red.
//! 2. `#RRGGBBAA`, eg: `#FF0000FF` for red with alpha.
//!
//! Here are some examples of valid input strings:
//! ```
//! style = {
//!     fg_color: #FF0000;
//!     bg_color: #FF0000FF;
//! }
//! ```
//! 1. The `fg_color` and `bg_color` are both optional.
//! 2. The `style = {` and `}` are required.

#[cfg(test)]
mod tests {
    use nom::{
        branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*,
        sequence::*, IResult,
    };
    use std::collections::HashMap;

    /// Output structs to hold color w/ and w/out alpha.
    mod output_structs {
        #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
        pub enum ColorKind {
            FgColor,
            BgColor,
            None,
        }

        impl From<ColorKind> for &str {
            fn from(color_kind: ColorKind) -> Self {
                match color_kind {
                    ColorKind::FgColor => "fg_color",
                    ColorKind::BgColor => "bg_color",
                    ColorKind::None => "_",
                }
            }
        }

        impl From<&str> for ColorKind {
            fn from(s: &str) -> Self {
                match s {
                    "fg_color" => Self::FgColor,
                    "bg_color" => Self::BgColor,
                    _ => Self::None,
                }
            }
        }

        #[derive(Debug, Copy, Clone)]
        pub enum Color {
            NoAlpha(ColorNoAlpha),
            WithAlpha(ColorWithAlpha),
        }

        #[derive(Debug, Copy, Clone)]
        pub struct ColorNoAlpha {
            pub r: u8,
            pub g: u8,
            pub b: u8,
        }

        impl ColorNoAlpha {
            pub fn new(red: u8, green: u8, blue: u8) -> Self {
                Self {
                    r: red,
                    g: green,
                    b: blue,
                }
            }
        }

        #[derive(Debug, Copy, Clone)]
        pub struct ColorWithAlpha {
            pub r: u8,
            pub g: u8,
            pub b: u8,
            pub a: u8,
        }

        impl ColorWithAlpha {
            pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
                Self {
                    r: red,
                    g: green,
                    b: blue,
                    a: alpha,
                }
            }
        }
    }
    pub use output_structs::*;

    /// Type alias for [nom::error::VerboseError] to make the code more readable.
    type VError<'input> = nom::error::VerboseError<&'input str>;

    /// Parser functions for a single hex segment & `#RRGGBB` & `#RRGGBBAA`.
    mod hex_color_parser_helper_fns {
        use super::*;

        pub fn parse_single_hex_segment(input: &str) -> IResult<&str, u8, VError> {
            map_res(
                take_while_m_n(2, 2, |it: char| it.is_ascii_hexdigit()),
                |it: &str| u8::from_str_radix(it, 16),
            )(input)
        }

        pub fn parse_hex_color_no_alpha(input: &str) -> IResult<&str, Color, VError> {
            let (input, _) = tag("#")(input)?;
            let (input, (red, green, blue)) = tuple((
                parse_single_hex_segment,
                parse_single_hex_segment,
                parse_single_hex_segment,
            ))(input)?;

            Ok((input, Color::NoAlpha(ColorNoAlpha::new(red, green, blue))))
        }

        pub fn parse_hex_color_with_alpha(input: &str) -> IResult<&str, Color, VError> {
            let (input, _) = tag("#")(input)?;
            let (input, (red, green, blue, alpha)) = tuple((
                parse_single_hex_segment,
                parse_single_hex_segment,
                parse_single_hex_segment,
                parse_single_hex_segment,
            ))(input)?;

            Ok((
                input,
                Color::WithAlpha(ColorWithAlpha::new(red, green, blue, alpha)),
            ))
        }
    }
    pub use hex_color_parser_helper_fns::*;

    /// Parser functions for a style multiline string.
    mod style_parser_helper_fns {
        use super::*;

        /// Parse `style = { bg_color: .. , fg_color: .. }` parser.
        pub fn parse_style(
            input: &str,
        ) -> IResult<&str, Option<HashMap<ColorKind, Color>>, VError> {
            // Parse `style = {`.
            let (input, _) = tuple((
                tag("style"),
                multispace0,
                char('='),
                multispace0,
                tag("{"),
                multispace0,
            ))(input)?;

            // Parse `bg_color: ..` or `fg_color: ..`.
            let (input, output) = many0(parse_color_key_value)(input)?;

            // Parse `}`.
            let (input, _) = tuple((multispace0, tag("}"), multispace0))(input)?;

            let output = {
                let mut it: HashMap<ColorKind, Color> = HashMap::new();
                for (color_kind, color) in output.iter() {
                    it.insert(*color_kind, *color);
                }
                it
            };

            Ok((input, Some(output)))
        }

        /// Parse `<key> : <val> ;`, where:
        /// 1. `<key>` can be `fg_color` or `bg_color`.
        /// 2. `<val>` can be `#RRGGBB` or `#RRGGBBAA`.
        pub fn parse_color_key_value(input: &str) -> IResult<&str, (ColorKind, Color), VError> {
            // Parse `fg_color` or `bg_color`.
            let (input, key_str) = alt((tag("fg_color"), tag("bg_color")))(input)?;

            // Parse `: #RRGGBBAA;` or `: #RRGGBB;`.
            let (input, (_, _, _, output, _, _, _)) = tuple((
                multispace0,
                tag(":"),
                multispace0,
                // The order in which these functions are called matters: first, try to parse
                // `#RRGGBBAA` & if it fails then try to parse `#RRGGBB`.
                alt((parse_hex_color_with_alpha, parse_hex_color_no_alpha)),
                multispace0,
                tag(";"),
                multispace0,
            ))(input)?;

            Ok((input, (ColorKind::from(key_str), output)))
        }
    }
    pub use style_parser_helper_fns::*;

    /// The input here is a Vec<String> and not &str. This is similar to getting input from the TUI
    /// editor component (which holds an editor buffer in a Vec<String>, where each String is a
    /// separate line).
    #[test]
    fn parse_vec_string() {
        let input_vec: Vec<String> = [
            "style = {",
            "    fg_color: #FF0000;",
            "    bg_color: #00FF00FF;",
            "}",
        ]
        .iter()
        .map(|it| it.to_string())
        .collect();

        let input_vec_converted: String = input_vec.join("\n");

        let actual = parse_style(&input_vec_converted).unwrap().1.unwrap();

        assert!(matches!(
            actual.get(&ColorKind::from("fg_color")).unwrap(),
            Color::NoAlpha(ColorNoAlpha { r: 255, g: 0, b: 0 })
        ));
        assert!(matches!(
            actual.get(&ColorKind::from("bg_color")).unwrap(),
            Color::WithAlpha(ColorWithAlpha {
                r: 0,
                g: 255,
                b: 0,
                a: 255
            })
        ));
    }

    #[test]
    fn test_parse_valid_hex_color_string_no_alpha() {
        let input = "#FF0000";
        let actual = parse_hex_color_no_alpha(input).unwrap().1;
        let matches = matches!(actual, Color::NoAlpha(ColorNoAlpha { r: 255, g: 0, b: 0 }));
        assert!(matches);
    }

    #[test]
    fn test_parse_valid_hex_color_string_with_alpha() {
        let input = "#FFFFFFFF";
        let actual = parse_hex_color_with_alpha(input).unwrap().1;
        assert!(matches!(
            actual,
            Color::WithAlpha(ColorWithAlpha {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            })
        ));
    }

    /// Test `<key>: #<val>`, where:
    /// 1. `<key>` can be `fg_color` or `bg_color`.
    /// 2. `<val>` can be `#RRGGBB` or `#RRGGBBAA`.
    #[test]
    fn test_parse_color_key_value() {
        // "fg_color: #FF0000;"
        {
            let input = "fg_color: #FF0000;";
            let actual = parse_color_key_value(input).unwrap().1;
            assert!(matches!(actual.0, ColorKind::FgColor));
            assert!(matches!(
                actual.1,
                Color::NoAlpha(ColorNoAlpha { r: 255, g: 0, b: 0 })
            ));
        }

        // "bg_color: #FF0000;"
        {
            let input = "bg_color: #FF0000;";
            let actual = parse_color_key_value(input).unwrap().1;
            assert!(matches!(actual.0, ColorKind::BgColor));
            assert!(matches!(
                actual.1,
                Color::NoAlpha(ColorNoAlpha { r: 255, g: 0, b: 0 })
            ));
        }

        // "fg_color: #FFFFFFFF;"
        {
            let input = "fg_color: #FFFFFFFF;";
            let actual = parse_color_key_value(input).unwrap().1;
            assert!(matches!(actual.0, ColorKind::FgColor));
            assert!(matches!(
                actual.1,
                Color::WithAlpha(ColorWithAlpha {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255
                })
            ));
        }

        // "bg_color: #FFFFFFFF;"
        {
            let input = "bg_color: #FFFFFFFF;";
            let actual = parse_color_key_value(input).unwrap().1;
            assert!(matches!(actual.0, ColorKind::BgColor));
            assert!(matches!(
                actual.1,
                Color::WithAlpha(ColorWithAlpha {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255
                })
            ));
        }
    }

    #[test]
    fn test_parse_valid_style_string() {
        let input = r#"style = {
    fg_color: #FF0000;
    bg_color: #00FF00FF;
}"#;
        let actual = parse_style(input).unwrap().1.unwrap();
        assert!(matches!(
            actual.get(&ColorKind::from("fg_color")).unwrap(),
            Color::NoAlpha(ColorNoAlpha { r: 255, g: 0, b: 0 })
        ));
        assert!(matches!(
            actual.get(&ColorKind::from("bg_color")).unwrap(),
            Color::WithAlpha(ColorWithAlpha {
                r: 0,
                g: 255,
                b: 0,
                a: 255
            })
        ));
    }
}
