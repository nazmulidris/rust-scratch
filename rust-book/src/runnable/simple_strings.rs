/*
 * Copyright (c) 2022 Nazmul Idris. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! <https://www.ameyalokare.com/rust/2017/10/12/rust-str-vs-String.html>
//!
//! 1. `String`   -> growable & heap allocated like Java `StringBuffer`
//! 2. `&String`  -> borrowed `String`; can be coerced into `&str`
//! 3. `str`      -> immutable like Java `String`
//! 4. `&str`     -> borrowed `str`; usually the "go to" type for strings in Rust.

use ansi_term::Colour::Green;
use rust_book_lib::utils::print_header;

pub fn run() {
  print_header("strings");
  string_buffer();
  string_coercion();
}

/// String is equivalent to Java StringBuffer.
fn string_buffer() {
  let mut string_buffer: String =
    String::from("String (like Java StringBuffer) is growable & heap allocated");
  string_buffer += ".";
  println!("string_buffer:String = '{}'", Green.paint(&string_buffer));
  println!(
    "string_buffer:String has capacity(): {}",
    string_buffer.capacity()
  );
  println!("string_buffer:String has len(): {}", string_buffer.len());

  let string: &str = "Immutable string";
  println!("string:&str = '{}'", Green.paint(string));
  println!("string:&str has no capacity() since its immutable");
  println!("string:&str has len(): {}", string.len());
}

/// &String can be coerced into &str.
fn string_coercion() {
  let string_buffer: String = String::from("&String - StringBuffer is mutable");
  let string_immutable: &str = "&str - Immutable string";
  do_something_with_amp_str(&string_buffer); // &String is coerced into &str.
  do_something_with_amp_str(&string_immutable);
}

fn do_something_with_amp_str(text: &str) {
  println!(
    "I accept &str; can coerce &String to &str: arg passed in {}",
    text
  );
}
