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

//! This module contains the parser for the markdown text. Test input is provided in
//! `test_input_md.txt` file.
//!
//! This is different from a standard MD parser in the following ways:
//! - There is metadata at the top of the file, in the form of `@<key>: [<value1>, <value2>, .. ]`
//!   lines.
//! - It handles multi-line text differently eg: when there's bold text, it can span multiple lines.
//! - This module does not have a full fledged parser, but it has enough functionality to be the
//!   starting point for one.
//! - The output of the parser is a `Vec<Vec<(Token, String)>>`
//!    - `(Token, String)` represents a single span of text where the `Token` is an enum that can be
//!       used to determine the semantic meaning of the text (eg: heading, bold, list item, etc).
//!       This is used for syntax highlighting, so that a stylesheet or theme can be applied to the
//!       text.
//!    - Vec<(Token, String)> represents a single line of text.
//!    - Multiple lines are represented by a `Vec<Vec<(Token, String)>>`.

#[cfg(test)]
mod tests {
    use nom::{
        branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*,
        sequence::*, IResult,
    };

    mod output_structs {
        use super::*;

        // TODO: define Token enum, and other structs
        #[derive(Debug)]
        pub enum Token {
            Bold,
            Heading,
            List,
            Title,
            Tag,
        }
    }
    use output_structs::*;

    mod parsers {
        use super::*;

        pub const BOLD: &str = "**";
        pub const NEW_LINE: &str = "\n";

        // TODO: define parsers for bold, meta, heading, etc
        pub fn parse_vec_lines(input: &str) -> IResult<&str, Vec<Vec<(Token, &str)>>> {
            let mut it = vec![];

            let (input, bold_vec) = parse_bold(input)?;
            it.push(bold_vec);

            // FIXME: this breaks it ... parser only works if bold is at start of the input
            let (input, bold_vec) = parse_bold(input)?;
            it.push(bold_vec);

            Ok((input, it))
        }

        /// More info: <https://stackoverflow.com/questions/67275710/iterating-over-multiple-lines-using-the-rust-nom-parsing-library>
        pub fn parse_bold(input: &str) -> IResult<&str, Vec<(Token, &str)>> {
            // Parse multiple lines of bold text. `bold_str` does not include `**`.
            let (input, bold_str) =
                delimited(tag_no_case(BOLD), take_until(BOLD), tag_no_case(BOLD))(input)?;

            if bold_str.contains(NEW_LINE) {
                // Split by newline. But leaves the newline in.
                let split_by_newlines = separated_list0(tag(NEW_LINE), take_until(NEW_LINE));
                // Terminated by newline, remove it.
                let terminator = tag(NEW_LINE);
                let (remainder, mut vec_bold_str) =
                    terminated(split_by_newlines, terminator)(bold_str)?;

                // Fold the remainder (if any) into the `Vec<&str>`.
                if !remainder.is_empty() {
                    vec_bold_str.push(remainder);
                }

                // Turn the `Vec<&str>` into a `Vec<(Token::Bold, &str)>`.
                Ok((
                    input,
                    vec_bold_str.iter().map(|it| (Token::Bold, *it)).collect(),
                ))
            } else {
                // Just return the single line of bold text.
                Ok((input, vec![(Token::Bold, bold_str)]))
            }
        }
    }
    use parsers::*;

    mod input_data {
        // Load the `input_md.txt` file into a `Vec<String>` mimicking editor component buffer.
        pub fn get_input_md_lines() -> Vec<String> {
            include_str!("input_md.txt")
                .lines()
                .map(|s| s.to_string())
                .collect()
        }
    }
    use input_data::*;

    #[test]
    fn test_parse_input_md() {
        let input_vec_lines = get_input_md_lines();
        println!("{input_vec_lines:#?}");

        let binding = input_vec_lines.join(NEW_LINE);
        let result = parse_vec_lines(&binding);
        println!("{result:#?}");

        // TODO: parse the `input_md.txt` file into a `Vec<Vec<(Token, String)>>` & make assertions
    }
}
