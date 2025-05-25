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

use std::{iter::Enumerate, vec::IntoIter};

use nom::{
    Err, IResult, Input,
    bytes::complete::take,
    error::{Error, ErrorKind},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedStrings(pub Vec<String>);

mod impl_input_for_owned_strings {
    use super::*;

    impl Input for OwnedStrings {
        type Item = String;
        type Iter = IntoIter<Self::Item>;
        type IterIndices = Enumerate<IntoIter<String>>;

        fn input_len(&self) -> usize {
            self.0.len()
        }

        fn iter_indices(&self) -> Self::IterIndices {
            self.0.clone().into_iter().enumerate()
        }

        fn iter_elements(&self) -> Self::Iter {
            self.0.clone().into_iter()
        }

        fn position<P>(&self, predicate: P) -> Option<usize>
        where
            P: Fn(Self::Item) -> bool,
        {
            self.0.iter().position(|item| predicate(item.clone()))
        }

        fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
            if self.0.len() >= count {
                Ok(count)
            } else {
                Err(nom::Needed::new(count - self.0.len()))
            }
        }

        fn take(&self, count: usize) -> Self {
            OwnedStrings(self.0[..count].to_vec())
        }

        fn take_from(&self, count: usize) -> Self {
            OwnedStrings(self.0[count..].to_vec())
        }

        fn take_split(&self, count: usize) -> (Self, Self) {
            let taken = self.take(count);
            let remaining = self.take_from(count);
            (taken, remaining)
        }
    }
}

pub fn parse_hex_color(input: OwnedStrings) -> IResult<OwnedStrings, (u8, u8, u8)> {
    // Take 3 hex strings. Error if there are not enough items.
    let it: (OwnedStrings, OwnedStrings) = take(/* 3 items */ 3u8)(input)?;
    let (rem, hex_strings) = it;

    let iter_for_underlying_vec = &mut hex_strings.0.iter();
    let red_item = &iter_for_underlying_vec.next().unwrap();
    let green_item = &iter_for_underlying_vec.next().unwrap();
    let blue_item = &iter_for_underlying_vec.next().unwrap();

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
            return Err(Err::Error(Error::new(
                hex_strings.clone(),
                ErrorKind::MapRes,
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This is the behavior we get when using `take` with a `&str` input.
    /// Compare this to the `take` with `OwnedStrings` input.
    #[test]
    fn nom_take_for_str_slice() {
        let input = "2F14DF";
        let it: IResult<&str, &str> = take(3u8)(input);
        if let Ok((_rem, color_parts)) = it {
            assert_eq!(color_parts, "2F1");
            assert_eq!(_rem, "4DF");
        } else {
            panic!("Failed to parse color parts from string input");
        }
    }

    #[test]
    fn test_parse_hex_color() {
        let input_owned = OwnedStrings(vec![
            /* r */ "2F".to_string(),
            /* g */ "14".to_string(),
            /* b */ "DF".to_string(),
        ]);

        let expected_output = (
            0x2F, // r
            0x14, // g
            0xDF, // b
        );

        let output_result = parse_hex_color(input_owned);
        let (remainder, actual_output) = output_result.unwrap();
        assert_eq!(actual_output, expected_output);
        assert_eq!(remainder.0, vec![] as Vec<String>);
    }
}
