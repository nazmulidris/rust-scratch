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
//!
//! Sentences of this form need to be parsed into a struct, one for each sentence. This example is
//! inspired by this [video](https://gist.github.com/lmammino/0c3e7e6dbaf41303d1059c6a279c59d1).

#[cfg(test)]
mod tests {
    use nom::{
        branch::*, bytes::complete::*, character::complete::*, combinator::*, sequence::*, IResult,
    };

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
    mod parsers {
        use super::*;

        /// Optional whitespace. Then optional "and" or optional ",". Then optional whitespace.
        pub fn parse_optional_and_or_comma(input: &str) -> IResult<&str, ()> {
            let (input, _) = multispace0(input)?;
            let (input, _) = alt((opt(tag_no_case("and")), opt(tag_no_case(","))))(input)?;
            let (input, _) = multispace0(input)?;
            Ok((input, ()))
        }

        /// Age starts w/ optional "and" or ",". Then "i am". Then optional whitespace. Then age
        /// (number).
        pub fn parse_age(input: &str) -> IResult<&str, u8> {
            let (input, _) = parse_optional_and_or_comma(input)?;
            let (input, _) = tuple((tag_no_case("i am"), multispace0))(input)?;
            let (input, age) = map_res(digit1, |age: &str| age.parse::<u8>())(input)?;
            let (input, _) = tag_no_case(" years old")(input)?;
            Ok((input, age))
        }

        /// Name starts w/ optional "and" or ",". Then "my name is". Then optional whitespace. Then
        /// name.
        pub fn parse_name(input: &str) -> IResult<&str, &str> {
            let (input, _) = parse_optional_and_or_comma(input)?;
            let (input, _) = tuple((multispace0, tag_no_case("my name is"), multispace0))(input)?;
            let (input, name) = alpha1(input)?;
            Ok((input, name))
        }

        /// Language starts w/ optional "and" or ",". Then "i like". Then optional whitespace. Then
        /// language.
        pub fn parse_language(input: &str) -> IResult<&str, &str> {
            let (input, _) = parse_optional_and_or_comma(input)?;
            let (input, _) = tuple((tag_no_case("i like"), multispace0))(input)?;
            let (input, language) = alpha1(input)?;
            Ok((input, language))
        }

        /// Sentence starts w/ "hello". Then optional whitespace. Then optional ",". Then optional
        /// whitespace.
        pub fn parse_sentence(input: &str) -> IResult<&str, Sentence> {
            let (input, _) = tuple((
                tag_no_case("hello"),
                multispace0,
                opt(tag(",")),
                multispace0,
            ))(input)?;

            // Name, age, and language show up next in any order.
            let (input, (name, age, language)) =
                permutation((parse_name, parse_age, parse_language))(input)?;

            Ok((
                input,
                Sentence {
                    name,
                    age,
                    language,
                },
            ))
        }
    }
    pub use parsers::*;

    #[test]
    fn parse_sentences() {
        let test_data = [
            "Hello, my name is Tommaso and i am 32 years old and I like Rust",
            "Hello, my name is Roberto and i like Python and I am 44 years old",
            "Hello, I like JavaScript my name is Luciano i am 35 years old",
        ];
        for line in test_data.iter() {
            let (_, output) = parse_sentence(line).unwrap();
            println!("{line} -> {output:#?}");
        }
    }
}
