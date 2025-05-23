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
///
/// Here's an example:
///
/// ```
/// use nom::IResult;
/// use nom_vec_input::common::StrSliceArray;
/// pub fn main() {
///     let input = "2f\n14\ndf";
///     let lines_vec: Vec<&str> = input.lines().collect();
///     let lines_vec_slice = lines_vec.as_slice();
///     _ = your_parse_function(lines_vec_slice);
/// }
///
/// pub fn your_parse_function<'a>(input: StrSliceArray<'a>)
/// -> IResult<StrSliceArray<'a>, u8>
/// { Ok((input, 0)) }
/// ```
pub type StrSliceArray<'a> = &'a [&'a str];
