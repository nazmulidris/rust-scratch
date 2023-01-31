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

#[cfg(test)]
mod tests {
    use std::num::ParseIntError;

    use nom::{
        error::{FromExternalError, ParseError},
        sequence::tuple,
        Parser,
    };

    use nom::{
        bytes::complete::{tag, take_while_m_n},
        combinator::map_res,
        IResult,
    };

    #[derive(Debug, PartialEq)]
    pub struct Color {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
    }

    impl Color {
        pub fn new(red: u8, green: u8, blue: u8) -> Self {
            Self { red, green, blue }
        }
    }

    /// Helper functions to match and parse hex digits. These are not [Parser] implementations.
    mod helper_fns {
        use super::*;

        /// This function is used by [map_res] and it returns a [Result], not [IResult].
        pub fn parse_str_to_hex_num(input: &str) -> Result<u8, std::num::ParseIntError> {
            u8::from_str_radix(input, 16)
        }

        /// This function is used by [take_while_m_n] and as long as it returns `true` items will be
        /// taken from the input.
        pub fn match_is_hex_digit(c: char) -> bool {
            c.is_ascii_hexdigit()
        }

        pub fn parse_hex_seg(input: &str) -> IResult<&str, u8> {
            map_res(
                take_while_m_n(2, 2, match_is_hex_digit),
                parse_str_to_hex_num,
            )(input)
        }
    }

    /// These are [Parser] implementations that are used by [hex_color_no_alpha].
    mod intermediate_parsers {
        use super::*;

        /// Call this to return function that implements the [Parser] trait.
        pub fn gen_hex_seg_parser_fn<'input, E>() -> impl Parser<&'input str, u8, E>
        where
            E: FromExternalError<&'input str, ParseIntError> + ParseError<&'input str>,
        {
            map_res(
                take_while_m_n(2, 2, helper_fns::match_is_hex_digit),
                helper_fns::parse_str_to_hex_num,
            )
        }
    }

    /// This is the "main" function that is called by the tests.
    fn hex_color_no_alpha(input: &str) -> IResult<&str, Color> {
        // This tuple contains 3 ways to do the same thing.
        let it = (
            helper_fns::parse_hex_seg, // This is preferred.
            intermediate_parsers::gen_hex_seg_parser_fn(),
            map_res(
                take_while_m_n(2, 2, helper_fns::match_is_hex_digit),
                helper_fns::parse_str_to_hex_num,
            ),
        );
        let (input, _) = tag("#")(input)?;
        let (input, (red, green, blue)) = tuple(it)(input)?; // same as `it.parse(input)?`
        Ok((input, Color { red, green, blue }))
    }

    #[test]
    fn parse_valid_color() {
        let mut input = String::new();
        input.push_str("#2F14DF");
        input.push('🔅');

        let result = dbg!(hex_color_no_alpha(&input));

        let Ok((remainder, color)) = result else { panic!(); };
        assert_eq!(remainder, "🔅");
        assert_eq!(color, Color::new(47, 20, 223));
    }

    #[test]
    fn parse_invalid_color() {
        let result = dbg!(hex_color_no_alpha("🔅#2F14DF"));
        assert!(result.is_err());
    }
}
