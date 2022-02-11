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

//! Rust book: <https://doc.rust-lang.org/book/ch08-01-vectors.html>

pub fn run() {}

#[test]
fn test_vector_numbers_simple() {
  // Mutable vector.
  let mut number_vec_1: Vec<i64> = Vec::new();
  number_vec_1.push(1);

  assert_eq!(number_vec_1.len(), 1);
  assert_eq!(number_vec_1[0], 1);
  assert_eq!(number_vec_1.get(0), Some(&1));
  assert_eq!(number_vec_1.get(2), None);

  number_vec_1.push(2);
  number_vec_1[0] = 3;

  assert_eq!(number_vec_1.len(), 2);
  assert_eq!(number_vec_1[0], 3);
  assert_eq!(number_vec_1[1], 2);

  // Immutable vector created from macro.
  let number_vec_2: Vec<i64> = vec![1, 2, 3];
  assert_eq!(number_vec_2.len(), 3);
}

#[test]
fn test_vector_strings_simple() {
  let mut string_vec_1: Vec<String> = Vec::new();
  string_vec_1.push("Hello".to_string());
  assert_eq!(string_vec_1.len(), 1);

  let item_1: &String = &string_vec_1[0];
  assert!(item_1.contains("Hello"));

  let item_1_option: Option<&String> = string_vec_1.get(0);
  assert!(item_1_option.is_some());
  assert_eq!(item_1_option, Some(&"Hello".to_string()));
  assert_eq!(item_1_option.unwrap(), "Hello");
}

#[test]
fn test_vector_borrow_checker() {
  let mut string_vec_1: Vec<String> = vec!["one".to_string(), "two".to_string()];
  string_vec_1.push("three".to_string());
  let item_1: &String = &string_vec_1[0];
  assert!(item_1.contains("one"));

  // 🧨 The following lines will cause a compiler error.
  // string_vec_1.push("four".to_string());
  // println!("{}", item_1);
  //
  // 🤔 Why should a reference to the first element care about what changes at the end of the
  // vector? This error is due to the way vectors work: adding a new element onto the end of the
  // vector might require allocating new memory and copying the old elements to the new space, if
  // there isn’t enough room to put all the elements next to each other where the vector currently
  // is. In that case, the reference to the first element would be pointing to deallocated memory.
  // The borrowing rules prevent programs from ending up in that situation.
}

#[test]
fn test_iterate_and_mutate_vector() {
  let mut string_vec_1: Vec<String> = vec!["one".to_string(), "two".to_string()];
  string_vec_1.push("three".to_string());
  for item in &string_vec_1 {
    assert!(item.contains("one") | item.contains("two") | item.contains("three"));
  }

  string_vec_1.iter_mut().for_each(|it| {
    it.push_str("!");
  });

  string_vec_1.iter().for_each(|it| {
    assert!(it.contains("one!") | it.contains("two!") | it.contains("three!"));
  });
}

#[test]
fn test_store_multiple_types_in_vector() {
  // Enum that can be stored in a vector.
  enum TrixCell {
    Empty,
    Int(i64),
    Float(f64),
    Text(String),
  }

  // Row represents a row in a spreadsheet w/ 6 columns.
  let row_1 = vec![
    TrixCell::Empty,
    TrixCell::Int(1),
    TrixCell::Float(1.0),
    TrixCell::Text("one".to_string()),
    TrixCell::Int(1),
    TrixCell::Float(1.0),
  ];

  let row_2 = vec![
    TrixCell::Empty,
    TrixCell::Int(2),
    TrixCell::Float(2.0),
    TrixCell::Text("two".to_string()),
    TrixCell::Int(2),
    TrixCell::Float(2.0),
  ];

  assert_eq!(row_1.len(), 6);
  assert_eq!(row_2.len(), 6);

  // https://stackoverflow.com/questions/9109872/how-do-you-access-enum-values-in-rust
  let row_1_col_1: &TrixCell = row_1.get(1).unwrap();
  if let TrixCell::Int(value) = row_1_col_1 {
    assert_eq!(value, &1);
  }

  let row_2_col_2: &TrixCell = row_2.get(2).unwrap();
  if let TrixCell::Float(value) = row_2_col_2 {
    assert_eq!(value, &2.0);
  }
}
