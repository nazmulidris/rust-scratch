/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

//! video: <https://youtu.be/q6paRBbLgNw>
//! book: <https://danielkeep.github.io/tlborm/book/pim-README.html>
//! `vecmac.rs` example: <https://gist.github.com/jonhoo/ec57882a976a2d2a92b3138ea25cd45a>

/// Block: Wrap the matched pattern in an extra set of "{}" to generate code in a block.
#[test]
fn test_macro_by_eg_generate_lambda() {
  macro_rules! make_fn {
    ($arg:ident) => {{
      let foo = || {
        return format!("{}", $arg);
      };
      foo
    }};
  }

  let var_1 = "foo";

  // Create the `lambda_1` (capturing) lambda.
  let lambda_1 = make_fn!(var_1);
  assert_eq!(lambda_1(), "foo");

  println!("{}", var_1);
}

/// Inline: Without wrapping the matched pattern in an extra set of "{}" it will generate
/// the code "inline".
#[test]
fn test_macro_by_eg_generate_lambda_inline() {
  macro_rules! make_fn {
    ($name:ident, $arg:ident) => {
      let $name = || {
        return format!("{}", $arg);
      };
    };
  }

  let var_1 = "foo";

  // Create the `lambda_1` (capturing) lambda.
  make_fn!(lambda_1, var_1);
  assert_eq!(lambda_1(), "foo");

  println!("{}", var_1);
}
