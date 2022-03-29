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

/// Experiment - generate a simple struct w/ given field declarations.
macro_rules! make_struct {
  ($name: ident, $($field_name: ident: $ty: ty),*) => {
    #[derive(Debug)]
    struct $name { $($field_name: $ty),* }
  }
}

#[test]
fn test_make_struct() {
  make_struct!(Thing, field_1: i32, field_2: String);
  let instance = Thing {
    field_1: 12,
    field_2: "abc".to_string(),
  };
  assert_eq!(instance.field_1, 12);
  assert_eq!(instance.field_2, "abc");
}
