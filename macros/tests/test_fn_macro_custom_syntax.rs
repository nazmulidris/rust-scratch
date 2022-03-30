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

//! # Watch macro expansion
//!
//! To watch for changes run this script:
//! `./cargo-watch-macro-expand-one-test.fish test_fn_macro_custom_syntax`
//!
//! # Watch test output
//!
//! To watch for test output run this script:
//! `./cargo-watch-one-test.fish test_fn_macro_custom_syntax`

use my_proc_macros_lib::fn_macro_custom_syntax;

#[test]
fn test_fn_macro_custom_syntax_full() {
  fn_macro_custom_syntax! {
    ThingManager<K, V>
    where K: Send + Sync + Default + 'static, V: Send + Sync + Default + 'static
    for std::collections::HashMap<K, V>
  }

  let mut thing_manager = ThingManager::<String, String> {
    wrapped_thing: std::collections::HashMap::new(),
  };
  thing_manager.wrapped_thing.insert(
    "key".to_string(),
    "value".to_string(),
  );

  assert_eq!(
    thing_manager
      .wrapped_thing
      .get("key"),
    Some(&"value".to_string())
  );
  assert_eq!(
    thing_manager
      .wrapped_thing
      .get("key2"),
    None
  );
}

#[test]
fn test_fn_macro_custom_syntax_no_where_clause() {
  fn_macro_custom_syntax! {
    ThingManager<K, V>
    for std::collections::HashMap<K, V>
  }

  let mut thing_manager = ThingManager::<String, String> {
    wrapped_thing: std::collections::HashMap::new(),
  };
  thing_manager.wrapped_thing.insert(
    "key".to_string(),
    "value".to_string(),
  );

  assert_eq!(
    thing_manager
      .wrapped_thing
      .get("key"),
    Some(&"value".to_string())
  );
  assert_eq!(
    thing_manager
      .wrapped_thing
      .get("key2"),
    None
  );
}
