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

//! This module contains the code to parse natural language sentences.
//!
//! The sentences are in the form:
//! ```text
//! Hello, [my name is <name>] and [i am <age> years old] and [i like <language>]
//! ```
//! - The `and` is optional; it can be omitted or replaced w/ a `,`.
//! - The age, name, and language may appear in any order.
//! - You must have all 3 fields.
//!
//! Sentences of this form need to be parsed into a struct, one for each sentence. This example is
//! inspired by this [video](https://gist.github.com/lmammino/0c3e7e6dbaf41303d1059c6a279c59d1).

use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, sequence::*, IResult,
};

#[allow(dead_code)]
/// Hold the parsed data from a sentence.
mod output_structs {
    #[derive(Debug)]
    pub struct Sentence<'a> {
        pub name: &'a str,
        pub age: u8,
        pub language: &'a str,
    }
}
pub use output_structs::*;

/// Functions that can be composed to parse the sentence.
#[allow(dead_code)]
mod parse_sentence {
    use super::*;

    /// Sentence starts w/ "hello". Then optional whitespace. Then optional ",". Then optional
    /// whitespace.
    pub fn root(input: &str) -> IResult</* remainder */ &str, Sentence> {
        let (rem, _) = tuple((
            tag_no_case("hello"),
            multispace0,
            opt(tag(",")), /* cut() also works instead of opt() */
            multispace0,
        ))(input)?;

        // Name, age, and language show up next in any order.
        let (rem, (name, age, language)) = permutation((name, age, language))(rem)?;

        Ok((
            rem,
            Sentence {
                name,
                age,
                language,
            },
        ))
    }

    /// Optional whitespace. Then optional "and" or optional ",". Then optional whitespace.
    #[rustfmt::skip]
    pub fn optional_connector_phrase(input: &str) -> IResult<&str, ()> {
        let (rem, _spaces) = multispace0(input)?;
        let (rem, _prefix) = opt(
            alt((
                tag_no_case("and"),
                tag(",")
            ))
        )(rem)?;
        let (rem, _spaces) = multispace0(rem)?;
        Ok((rem, ()))
    }

    /// Age starts w/ optional "and" or ",". Then "i am". Then optional whitespace. Then
    /// age (number).
    #[rustfmt::skip]
    pub fn age(input: &str) -> IResult<&str, u8> {
        let (rem, _prefix) = optional_connector_phrase(input)?;
        let (rem, _i_am_with_spaces) = tuple((
            tag_no_case("i am"),
            multispace0
        ))(rem)?;
        let (rem, age) = map_res(digit1, |age: &str| age.parse::<u8>())(rem)?;
        let (rem, _suffix) = tag_no_case(" years old")(rem)?;
        Ok((rem, age))
    }

    /// Name starts w/ optional "and" or ",". Then "my name is". Then optional whitespace. Then
    /// name.
    #[rustfmt::skip]
    pub fn name(input: &str) -> IResult<&str, &str> {
        let (rem, _prefix) = optional_connector_phrase(input)?;
        let (rem, _my_name_is_with_spaces) = tuple((
            multispace0,
            tag_no_case("my name is"),
            multispace0
        ))(rem)?;
        let (rem, name) = alpha1(rem)?;
        Ok((rem, name))
    }

    /// Language starts w/ optional "and" or ",". Then "i like". Then optional whitespace. Then
    /// language.
    #[rustfmt::skip]
    pub fn language(input: &str) -> IResult<&str, &str> {
        let (rem, _prefix) = optional_connector_phrase(input)?;
        let (rem, _i_like_with_spaces) = tuple((
            tag_no_case("i like"),
            multispace0
        ))(rem)?;
        let (rem, language) = alpha1(rem)?;
        Ok((rem, language))
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_natural_lang::parse_sentence;

    #[test]
    fn parse_sentences() {
        let test_data = [
            "Hello, my name is Tommaso and i am 32 years old and I like Rust",
            "Hello, my name is Roberto and i like Python, I am 44 years old",
            "Hello, I like JavaScript my name is Luciano i am 35 years old",
        ];
        for line in test_data.iter() {
            let (_, output) = parse_sentence::root(line).unwrap();
            println!("{line} -> {output:#?}");
        }
    }
}
