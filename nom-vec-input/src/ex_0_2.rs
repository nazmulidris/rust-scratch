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
use crate::ex_0_1::OwnedStrings;
use nom::IResult;
use nom::bytes::complete::take;
use nom::error::{Error, ErrorKind};
use std::iter::{Enumerate, Map};
use std::slice::Iter;

/// A slice of string slices.
///
/// ```text
/// "Slice": &[
///     [0]: "2f", <- "StringSlices"
///     [1]: "14", <- "StringSlices"
///     [2]: "d",  <- "StringSlices"
/// ]
/// ```
///
/// A reference to an array of string slices, commonly used to represent a sequence of
/// lines or tokens in parsing tasks.
///
/// `SliceOfStringSlices<'a>` is a borrowed reference to a slice (`&[]`) of string slices
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
/// `SliceOfStringSlices` is just a type alias (syntactic sugar) that translates to `&[&str]`.
/// However, they are free to return an owned struct, or slices, or combinations of them;
/// again this is defined in the generic signature of the parser function itself.
///
/// When you use `SliceOfStringSlices`, due to the reason above, the [str::lines] method,
/// followed by [Iterator::collect], is called outside a parser function that receives
/// this as an argument. And a reference to this slice is passed into this parser
/// function.
#[derive(Debug, Clone, PartialEq)]
pub struct SliceOfStringSlices<'a>(pub &'a [&'a str]);

impl<'a> nom::Input for SliceOfStringSlices<'a> {
    type Item = &'a str;
    /// ### Why This Type Is Needed
    ///
    /// - You have a collection of string references: `&[&str]`
    /// - When you iterate through this collection normally, you get references to each
    ///   item: `&&str`
    /// - But you need to return just the original string references: `&str`
    /// - The `Map` iterator allows you to "unwrap" one level of reference, converting
    ///   `&&str` to `&str`
    ///
    /// We can't use `Iter<'a, &'a str>`, since it yields `&&'a str`.
    ///
    /// The issue addressed by this custom iterator relates to reference levels:
    /// 1. The `SSArray<'a>` struct wraps a `&'a [&'a str]` (a slice of string slices).
    /// 2. When we iterate over this slice using `.iter()`, we get an iterator that yields
    ///    `&&'a str` elements (a reference to each element in the slice).
    /// 3. However, the `nom::Input` trait expects an iterator that yields `&'a str`
    ///    elements.
    ///
    /// ### Components of this type
    ///
    /// It with [Self::iter_elements()] and defines a custom iterator type that performs a
    /// transformation on the elements yielded by another iterator. This type yields `&'a
    /// str` (not `&&'a str`).
    ///
    /// 1. `Map<T, F>` - This is a standard Rust iterator adapter that applies a function
    ///    to each element of an underlying iterator.
    /// 2. `Iter<'a, &'a str>` - This is the underlying iterator, which iterates over a
    ///    slice of `&'a str` elements. When you call `.iter()` on a slice, this is the type
    ///    you get.
    /// 3. `fn(&&'a str) -> &'a str` - This is a function pointer type that takes a `&&'a
    ///    str` (a reference to a reference to a string slice) and returns a `&'a str` (a
    ///    reference to a string slice).
    type Iter = Map<
        /* underlying iter */ Iter<'a, &'a str>,
        /* map function that derefs each item */fn(&&'a str) -> &'a str
    >; // Yield: &'a str, not &&'a str.
    type IterIndices = Enumerate<Self::Iter>; // Yields: (usize, &'a str).

    fn input_len(&self) -> usize {
        self.0.len()
    }

    fn take(&self, count: usize) -> Self {
        SliceOfStringSlices(&self.0[..count.min(self.0.len())])
    }

    fn take_from(&self, count: usize) -> Self {
        if count >= self.0.len() {
            SliceOfStringSlices(&[])
        } else {
            SliceOfStringSlices(&self.0[count..])
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let count = count.min(self.0.len());
        (SliceOfStringSlices(&self.0[..count]), SliceOfStringSlices(&self.0[count..]))
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0.iter().position(|&item| predicate(item))
    }

    /// Modified the `iter_elements` function to use `map` to transform `&&'a str` to `&'a
    /// str`. The [Map] iterator adapter is used to transform each `&&'a str` into a `&'a
    /// str` by dereferencing one level.
    fn iter_elements(&self) -> Self::Iter {
        self.0.iter().map(|&s| s)
    }

    fn iter_indices(&self) -> Self::IterIndices {
        self.iter_elements().enumerate()
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.0.len() >= count {
            Ok(count)
        } else {
            Err(nom::Needed::new(count - self.0.len()))
        }
    }
}

pub fn parse_hex_color<'a>(input: SliceOfStringSlices<'a>) -> IResult<SliceOfStringSlices<'a>, (u8, u8, u8)> {
    // Take 3 hex strings. Error if there are not enough items.
    let it: (SliceOfStringSlices<'a>, SliceOfStringSlices<'a>) = take(/* 3 items */ 3u8)(input)?;
    let (rem, hex_strings) = it;

    let iter_for_underlying_vec = &mut hex_strings.0.iter();
    let red_item = iter_for_underlying_vec.next().unwrap();
    let green_item = iter_for_underlying_vec.next().unwrap();
    let blue_item = iter_for_underlying_vec.next().unwrap();

    match (
        u8::from_str_radix(red_item, 16),
        u8::from_str_radix(green_item, 16),
        u8::from_str_radix(blue_item, 16),
    ) {
        (Ok(r), Ok(g), Ok(b)) => {
            // All conversions succeeded.
            return Ok((rem, (r, g, b)));
        }
        _ => {
            return Err(nom::Err::Error(Error::new(hex_strings, ErrorKind::MapRes)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::bytes::complete::take;

    #[test]
    fn test_ss_ray_with_take() {
        let input = "2f\n14\ndf";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();

        let ss_ray = SliceOfStringSlices(lines_vec_slice);
        assert_eq!(ss_ray.0, lines_vec_slice);

        let (rem, output) =
            take::<_, SliceOfStringSlices<'_>, Error<SliceOfStringSlices<'_>>>(3u8)(ss_ray).unwrap();
        assert_eq!(rem.0, &[] as &[&str]);
        assert_eq!(output.0, &["2f", "14", "df"]);
    }

    #[test]
    fn test_parse_hex_color() {
        let input = "2f\n14\ndf";
        let lines_vec: Vec<&str> = input.lines().collect();
        let lines_vec_slice = lines_vec.as_slice();

        let ss_ray = SliceOfStringSlices(lines_vec_slice);
        assert_eq!(ss_ray.0, lines_vec_slice);

        let (rem, output) = parse_hex_color(ss_ray).unwrap();
        assert_eq!(rem.0, &[] as &[&str]);
        assert_eq!(output, (0x2f, 0x14, 0xdf));
    }
}
