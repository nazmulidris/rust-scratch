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
    error::{Error, ErrorKind},
};

/// A slice of string slices.
///
/// A reference to an array of string slices, commonly used to represent a sequence of
/// lines or tokens in parsing tasks.
///
/// `StrSliceArray<'a>` is a borrowed reference to a slice (`&[]`) of string slices
/// (`&str`), all with the same lifetime `'a`.
///
/// ```text
/// ┌────────────────────────────┐
/// │      &'a [&'a str]         │ // Reference to a slice of string slices
/// └────────────────────────────┘
///               │
///               ▼
///    ┌──────────────────────┐
///    │ [ "2f", "14", "df" ] │ // Slice (array) of string slices
///    └────┬─────┬─────┬─────┘
///         │     │     │
///         ▼     ▼     ▼
///         "2f"  "14"  "df" // Each element is a &str (string slice)
/// ```
///
/// Parser functions can receive an input argument of whatever type owned or borrowed.
/// This is defined in the generic signature of the parser function itself. However, for
/// parser functions that use this struct, they must receive a reference, since this
/// `StrSliceArray` is just a type alias (syntactic sugar) that translates to `&[&str]`.
/// However, they are free to return an owned struct, or slices, or combinations of them;
/// again this is defined in the generic signature of the parser function itself.
///
/// When you use `StrSliceArray`, due to the reason above, the [str::lines] method,
/// followed by [Iterator::collect], is called outside a parser function that receives
/// this as an argument. And a reference to this slice is passed into this parser
/// function.
type StrSliceArray<'a> = &'a [&'a str];

#[test]
fn test_str_slice_array_type() {
    use StrSliceArray;
    use nom::IResult;
    pub fn main() {
        let input = "2f\n14\ndf";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();
        _ = your_parse_function(lines_vec_slice);
    }

    pub fn your_parse_function<'a>(input: StrSliceArray<'a>) -> IResult<StrSliceArray<'a>, u8> {
        Ok((input, 0))
    }
}

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// This parser function can't receive an owned struct due to the generic types that are
/// declared here. The [StrSliceArray] type is a reference, and doesn't own the data.
///
/// For this reason the [str::lines] method, followed by [Iterator::collect], is called
/// outside this function. And a reference to this slice is passed into this parser
/// function by its caller.
///
/// However, this function is free to return an owned struct, or slices, or combinations
/// of them.
pub fn parse_color<'a>(input: StrSliceArray<'a>) -> IResult<StrSliceArray<'a>, Color> {
    let res = (parse_hex, parse_hex, parse_hex).parse(input);
    res.map(|(rem, (red, green, blue))| (rem, Color { red, green, blue }))
}

/// Try to consume the first item of the input array here and parse it as a hex number.
/// - If that fails then return an error. Don't consume the first item.
/// - If it succeeds then return the rest of the input array and the parsed value. With
///   the first item consumed.
///
/// The [Slice::split_first()] is equivalent to the following code:
/// ```
/// fn split_first<T>(slice: &[T]) -> Option<(&T, &[T])> {
///     if slice.is_empty() {
///         None
///     } else {
///         Some((&slice[0], &slice[1..]))
///     }
/// }
/// ```
fn parse_hex<'a>(input: StrSliceArray<'a>) -> IResult<StrSliceArray<'a>, u8> {
    let Some((first, rest)) = input.split_first() else {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    };

    let try_parse_first_to_u8 = u8::from_str_radix(first, 16);

    try_parse_first_to_u8
        .map(|val| (rest, val))
        .map_err(|_| nom::Err::Error(Error::new(rest, ErrorKind::HexDigit)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_ok_1() {
        let input = "2f\n14\ndf";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();
        assert_eq!(
            parse_color(lines_vec_slice),
            Ok((
                [].as_slice(), // No rem.
                Color {
                    red: 47,
                    green: 20,
                    blue: 223
                }
            ))
        );
    }

    #[test]
    fn test_parser_ok_2() {
        let input = vec!["2f", "14", "df"];
        let lines_vec_slice = input.as_slice();
        assert_eq!(
            parse_color(lines_vec_slice),
            Ok((
                [].as_slice(), // No rem.
                Color {
                    red: 47,
                    green: 20,
                    blue: 223
                }
            ))
        );
    }

    #[test]
    fn test_parser_with_invalid_hex_1() {
        let input = "2f\nzz\ndf";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();
        let res = parse_color(lines_vec_slice);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            nom::Err::Error(Error::new(
                ["df"].as_slice(), // "zz" is invalid hex, "df" is the rem.
                ErrorKind::HexDigit
            ))
        );
    }

    #[test]
    fn test_parser_with_invalid_hex_2() {
        let input = "2fx\n14\ndf\nzz";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();
        let res = parse_color(lines_vec_slice);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            nom::Err::Error(Error::new(
                ["14", "df", "zz"].as_slice(), // "2fx" is invalid hex, "14", "df", and "zz" are the rem.
                ErrorKind::HexDigit
            ))
        );
    }

    #[test]
    fn test_parser_with_too_few_lines() {
        let input = "2f\n14";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();
        let res = parse_color(lines_vec_slice);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            nom::Err::Error(Error::new([].as_slice(), ErrorKind::Eof))
        );
    }

    #[test]
    fn test_parser_with_extra_lines() {
        let input = "2f\n14\ndf\nff";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();
        assert_eq!(
            parse_color(lines_vec_slice),
            Ok((
                ["ff"].as_slice(), // "ff" is left as rem.
                Color {
                    red: 47,
                    green: 20,
                    blue: 223
                }
            ))
        );
    }

    #[test]
    fn test_parser_with_empty_input() {
        let input = "";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();
        let res = parse_color(lines_vec_slice);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            nom::Err::Error(Error::new([].as_slice(), ErrorKind::Eof))
        );
    }

    #[test]
    fn test_parser_with_non_hex_characters() {
        let input = "gg\n14\ndf";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();
        let res = parse_color(lines_vec_slice);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            nom::Err::Error(Error::new(
                ["14", "df"].as_slice(), // "gg" is invalid hex, "14" and "df" are the rem.
                ErrorKind::HexDigit
            ))
        );
    }
}
