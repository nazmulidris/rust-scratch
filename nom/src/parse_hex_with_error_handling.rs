/*
 *   Copyright (c) 2024 Nazmul Idris
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

//! More info:
//! - <https://github.com/rust-bakery/nom/blob/main/examples/json.rs#L209-L286>
//! - <https://tfpk.github.io/nominomicon/chapter_7.html>
//! - <https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md>
//! - <https://developerlife.com/2023/02/20/guide-to-nom-parsing/#getting-to-know-nom-using-a-simple-example>
#[cfg(test)]
mod tests_3 {
    use nom::{
        bytes::complete::{tag, take_while_m_n},
        combinator::map_res,
        error::{context, convert_error},
        sequence::Tuple,
        IResult, Parser,
    };

    /// `nom` is used to parse the hex digits from string. Then [u8::from_str_radix] is
    /// used to convert the hex string into a number. This can't fail, even though in the
    /// function signature, that may return a [core::num::ParseIntError], which never
    /// happens. Note the use of [nom::error::VerboseError] to get more detailed error
    /// messages that are passed to [nom::error::convert_error].
    ///
    /// Even if [core::num::ParseIntError] were to be thrown, it would be consumed, and
    /// a higher level `nom` error would be returned for the `map_res` combinator.
    pub fn parse_hex_seg(input: &str) -> IResult<&str, u8, nom::error::VerboseError<&str>> {
        map_res(
            take_while_m_n(2, 2, |it: char| it.is_ascii_hexdigit()),
            |it| u8::from_str_radix(it, 16),
        )
        .parse(input)
    }

    /// Note the use of [nom::error::VerboseError] to get more detailed error messages
    /// that are passed to [nom::error::convert_error].
    pub fn root(input: &str) -> IResult<&str, (&str, u8, u8, u8), nom::error::VerboseError<&str>> {
        let (remainder, (_, red, green, blue)) = (
            context("start of hex color", tag("#")),
            context("hex seg 1", parse_hex_seg),
            context("hex seg 2", parse_hex_seg),
            context("hex seg 3", parse_hex_seg),
        )
            .parse(input)?;

        Ok((remainder, ("", red, green, blue)))
    }

    #[test]
    fn test_root_1() {
        let input = "x#FF0000";
        let result = root(input);
        println!("{:?}", result);
        assert!(result.is_err());

        match result {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("Could not parse because ... {}", convert_error(input, e));
            }
            _ => { /* do nothing for nom::Err::Incomplete(_) */ }
        }
    }

    #[test]
    fn test_root_2() {
        let input = "#FF_0000";
        let result = root(input);
        println!("{:?}", result);
        assert!(result.is_err());

        match result {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("Could not parse because ... {}", convert_error(input, e));
            }
            _ => { /* do nothing for nom::Err::Incomplete(_) */ }
        }
    }

    #[test]
    fn test_root_3() {
        let input = "#FF0000";
        let result = root(input);
        println!("{:?}", result);
        assert!(result.is_ok());

        match result {
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                println!("Could not parse because ... {}", convert_error(input, e));
            }
            _ => { /* do nothing for nom::Err::Incomplete(_) */ }
        }
    }
}
