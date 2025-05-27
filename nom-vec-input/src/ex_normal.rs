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

use nom::{
    IResult, Parser as _,
    branch::alt,
    bytes::complete::{tag, take_till, take_while1},
    character::complete::{char, line_ending},
    combinator::{map, recognize, verify},
    multi::many0,
};

#[derive(Debug, PartialEq)]
enum Sentence<'a> {
    FULL(&'a str),
    PARTIAL(&'a str),
    EOL,
}

type Sentences<'a> = Vec<Sentence<'a>>;

fn parse_sentences(input: &str) -> Sentences {
    // Use many0 to apply parse_single_sentence repeatedly.
    match many0(parse_single_sentence).parse(input) {
        Ok((remainder, mut sentences)) => {
            // If there's any unparsed remainder, add it as a PARTIAL sentence.
            if !remainder.is_empty() {
                sentences.push(Sentence::PARTIAL(remainder));
            }
            sentences
        }
        Err(nom::Err::Error(_e)) | Err(nom::Err::Failure(_e)) => {
            // More specific error matching
            // If parsing fails at the very beginning or in an unrecoverable way,
            // treat the entire input as a single PARTIAL sentence if it's not empty.
            // Optionally log _e here for debugging.
            if !input.is_empty() {
                vec![Sentence::PARTIAL(input)]
            } else {
                Vec::new()
            }
        }
        Err(nom::Err::Incomplete(_)) => {
            // This case is unlikely with complete string inputs but handled for completeness.
            if !input.is_empty() {
                vec![Sentence::PARTIAL(input)] // Or handle as an error.
            } else {
                Vec::new()
            }
        }
    }
}

/// Parse a single sentence from the input string.
fn parse_single_sentence(input: &str) -> IResult<&str, Sentence<'_>> {
    // Do not reorder, the ordering of these branches matters for parsing.
    alt((
        // EOL: just a newline.
        map(line_ending, |_| Sentence::EOL),
        // Full sentence: text + period.
        map(
            recognize((
                // Consumes characters until a period.
                take_till(|c| c == '.'),
                // Consumes the period.
                char('.'),
            )),
            Sentence::FULL,
        ),
        // Whitespace that forms its own partial segment (e.g., after a period). Must
        // consume at least one whitespace character.
        //
        // take_while1 is used for the whitespace-only partial sentence to ensure it
        // consumes at least one character.
        map(
            take_while1(|c: char| c.is_whitespace() && c != '\n'),
            Sentence::PARTIAL,
        ),
        // Any other text until a period or newline. This must consume input to be valid
        // for many0, this is why we use verify (for many0 compatibility).
        //
        // We use `verify` to ensure that `take_till` actually consumes something. The
        // last branch uses take_till(|c| c == '.' || c == '\n') to consume text up to a
        // sentence terminator. This is wrapped in verify(..., |s: &str| !s.is_empty()).
        //
        // The verify combinator ensures that the inner parser (take_till) succeeds and
        // its output (s) is non-empty. If take_till produces an empty string (which
        // happens if the input is empty or starts with one of the delimiters ., \n),
        // verify will cause this branch of alt to fail. This is crucial for many0 to
        // terminate correctly and prevent infinite loops on empty matches.
        map(
            verify(
                // Consumes e.g., "bar ".
                take_till(|c| c == '.' || c == '\n'),
                // Ensures that s is not empty; fails if take_till matched nothing.
                |s: &str| !s.is_empty(),
            ),
            Sentence::PARTIAL,
        ),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sentences_no_trailing_eol() {
        let input = "foo. \nbar ";
        let output = parse_sentences(input);
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
    fn test_parse_sentences_with_empty_lines() {
        let input = "\nfoo.\n\nbar\n";
        let output = parse_sentences(input);
        assert_eq!(
            vec![
                Sentence::EOL,
                Sentence::FULL("foo."),
                Sentence::EOL,
                Sentence::EOL,
                Sentence::PARTIAL("bar"),
                Sentence::EOL,
            ],
            output
        );
    }

    #[test]
    fn test_lines_unexpected_newline_behavior() {
        // lines() unexpectedly removes trailing newlines!
        {
            let input = "foo.\nbar\nbaz.\n";
            let lines = input.lines().collect::<Vec<&str>>();
            assert_eq!(lines, vec!["foo.", "bar", "baz."]);
        }

        // Using split_inclusive to preserve trailing newlines.
        {
            let input = "foo.\nbar\nbaz.\n";
            let lines: Vec<&str> = input.split_inclusive('\n').collect();
            assert_eq!(lines, vec!["foo.\n", "bar\n", "baz.\n"]);
        }
    }
}
