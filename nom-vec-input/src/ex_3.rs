/*
 *   Copyright (c) 2025 Nazmul Idris
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

//! This parser is different from [mod@crate::ex_2] in the following ways:
//! 1. The input is not provided as [&str], rather a [OwnedStringArray].
//! 2. This is meant to simulate the output from [str::lines] [Iterator::collect].
//! 3. [str::lines] strips any trailing `\n` chars.
//!
//! Given that [str::lines] strips newline characters, the parsing strategy will be:
//!
//! 1. Iterate through the input [Vec]<[String]>.
//! 2. For each non-empty string (line content), parse it into [Sentence::FULL] and
//!    [Sentence::PARTIAL] sentences. We'll need a helper nom parser for this that doesn't
//!    expect or produce [Sentence::EOL] from newlines within the content itself.
//! 3. If a string in the input [Vec]<[String]> is empty, it represents an empty line from
//!    the original input, so it will be treated as an [Sentence::EOL].
//! 4. An [Sentence::EOL] will be inserted to separate the content derived from one string
//!    in the input [Vec]<[String]> from the content of the next, effectively representing
//!    the newline that was originally between them.

use nom::{
    IResult, Parser as _,
    branch::alt,
    bytes::complete::{tag, take_till, take_while1},
    character::complete::{char, line_ending},
    combinator::{map, recognize, verify},
    multi::many0,
};

use crate::common::{OwnedStringArray, Sentence, Sentences};

/// New version of [crate::ex_2::parse_sentences()] that takes a slice of [String]s and
/// not a [&str]. Each [String] is expected to be a line from the original input, with
/// newlines already stripped (e.g., from [str::lines] [Iterator::collect].
pub fn parse_sentences<'a>(input: OwnedStringArray<'a>) -> Sentences<'a> {
    let mut result = Vec::new();

    for (idx, line_str_obj) in input.iter().enumerate() {
        let line_content: &str = line_str_obj.as_str();

        if line_content.is_empty() {
            // An empty string in the vec means an empty line from original input.
            result.push(Sentence::EOL);
        } else {
            // Parse the content of the non-empty line.
            result.append(&mut parse_line(line_content));
        }

        // If this is not the last line in the input vector,
        // it means there was a newline separating it from the next line.
        if idx < input.len() - 1 {
            result.push(Sentence::EOL);
        }
    }

    result
}

/// Parses the entire content of a single line string into a [Vec]<[Sentence]>. The input
/// `line_content` is assumed to have no newline characters. This function uses
/// [parse_sentence_within_line] to extract sentences from the line.
fn parse_line(line_content: &str) -> Vec<Sentence<'_>> {
    if line_content.is_empty() {
        return Vec::new();
    }

    // Use many0 to apply parse_element_within_line repeatedly.
    match many0(parse_sentence_within_line).parse(line_content) {
        Ok((remainder, mut sentences)) => {
            // If there's any unparsed remainder, add it as a PARTIAL sentence.
            // This should ideally not happen if parse_element_within_line is comprehensive.
            if !remainder.is_empty() {
                sentences.push(Sentence::PARTIAL(remainder));
            }
            sentences
        }
        Err(_) => {
            // If parsing fails (e.g., many0 itself errors, though unlikely for many0),
            // treat the entire line_content as a single PARTIAL sentence.
            vec![Sentence::PARTIAL(line_content)]
        }
    }
}

/// Parses a [Sentence::FULL] or [Sentence::PARTIAL] from a line's content. Assumes no
/// newlines are present in the input string.
fn parse_sentence_within_line(input: &str) -> IResult<&str, Sentence<'_>> {
    alt((
        // Full sentence: text + period.
        map(
            recognize((
                take_till(|c: char| c == '.'), // Consume text up to a period
                char('.'),                     // Consume the period
            )),
            Sentence::FULL,
        ),
        // PARTIAL: Consumes text that is NOT a period and NOT just whitespace if whitespace is handled separately.
        // For "bar ", this should consume "bar ".
        // For " ", this should consume " ".
        map(
            // Consume one or more characters that are not a period.
            // This will take "bar " or " "
            verify(
                take_till(|c: char| c == '.'),
                |s: &str| !s.is_empty(), // Must consume something
            ),
            Sentence::PARTIAL,
        ),
        // This rule might be redundant if the above PARTIAL is greedy enough.
        // Let's remove it for now and see if the above PARTIAL handles " " correctly.
        // map(
        //     take_while1(|c: char| c.is_whitespace()),
        //     Sentence::PARTIAL,
        // ),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests_for_vec_input {
    use super::*;

    #[test]
    fn test_vec_parse_sentences_no_trailing_eol() {
        let input_str = "foo. \nbar ";
        let lines_vec: Vec<String> = input_str.lines().map(String::from).collect();
        let lines_slice = lines_vec.as_slice();
        let output = parse_sentences(lines_slice);
        assert_eq!(
            vec![
                Sentence::FULL("foo."),
                Sentence::PARTIAL(" "),
                Sentence::EOL,
                Sentence::PARTIAL("bar "),
            ],
            output
        );
    }

    #[test]
    fn test_vec_parse_sentences_with_empty_lines() {
        let input_str = "\nfoo.\n\nbar\n";
        let lines_vec: Vec<String> = input_str.lines().map(String::from).collect();
        // lines_vec will be: ["", "foo.", "", "bar"]
        let lines_slice = lines_vec.as_slice();

        let output = parse_sentences(lines_slice);

        // The expected output for this case needs to be what parse_sentences_from_lines
        // would logically produce from ["", "foo.", "", "bar"].
        // 1. line "": EOL. Separator EOL. -> [EOL, EOL]
        // 2. line "foo.": FULL("foo."). Separator EOL. -> [EOL, EOL, FULL("foo."), EOL]
        // 3. line "": EOL. Separator EOL. -> [EOL, EOL, FULL("foo."), EOL, EOL, EOL]
        // 4. line "bar": PARTIAL("bar"). No separator. -> [EOL, EOL, FULL("foo."), EOL, EOL, EOL, PARTIAL("bar")]
        let expected_output_for_vec_logic = vec![
            Sentence::EOL, // From first empty string ""
            Sentence::EOL, // Separator after first line
            Sentence::FULL("foo."),
            Sentence::EOL, // Separator after "foo."
            Sentence::EOL, // From third empty string ""
            Sentence::EOL, // Separator after third line
            Sentence::PARTIAL("bar"),
        ];
        assert_eq!(expected_output_for_vec_logic, output);

        // Note: The original test's expected output for this case was:
        // vec![EOL, FULL("foo."), EOL, EOL, PARTIAL("bar"), EOL]
        // This original expectation relies on parsing the raw "&str" and how it handles
        // the final newline. The Vec<String> from .lines() loses some of this info,
        // specifically about a single trailing newline on the whole input.
    }

    #[test]
    fn test_vec_simple_line() {
        let input_str = "hello world.";
        let lines_vec: Vec<String> = input_str.lines().map(String::from).collect();
        let lines_slice = lines_vec.as_slice();
        let output = parse_sentences(lines_slice);
        assert_eq!(vec![Sentence::FULL("hello world.")], output);
    }

    #[test]
    fn test_vec_two_lines() {
        let input_str = "first line.\nsecond part";
        let lines_vec: Vec<String> = input_str.lines().map(String::from).collect();
        let lines_slice = lines_vec.as_slice();
        let output = parse_sentences(lines_slice);
        assert_eq!(
            vec![
                Sentence::FULL("first line."),
                Sentence::EOL,
                Sentence::PARTIAL("second part"),
            ],
            output
        );
    }
}
