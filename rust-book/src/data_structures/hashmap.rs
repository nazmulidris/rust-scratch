/*
 Copyright 2022 Nazmul Idris

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

      https://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
*/

//! Rust book: <https://doc.rust-lang.org/book/ch08-03-hash-maps.html>

use std::collections::HashMap;

pub fn run() {}

#[test]
fn test_simple_map_creation() {
  let mut map: HashMap<String, u64> = HashMap::new();
  let moved_palo_alto_string = "palo alto".to_string();
  map.insert(moved_palo_alto_string, 94028);
  map.insert("mountain view".to_string(), 94041);
  map.insert("menlo park".to_string(), 94025);

  assert_eq!(map.len(), 3);

  let value_option: Option<&u64> = map.get("palo alto");
  let value: &u64 = value_option.unwrap();
  assert_eq!(*value, 94028);

  assert_eq!(map.get("mountain view"), Some(&94041));

  if let Some(value) = map.get("menlo park") {
    assert_eq!(*value, 94025);
  }
}

#[test]
fn test_zip_and_collect_map_creation() {
  let moved_key_array: Vec<&str> = vec!["palo alto", "mountain view", "menlo park"];
  let moved_zip_code_array: Vec<u64> = vec![94028, 94041, 94025];

  let map: HashMap<&str, u64> = moved_key_array
    .into_iter()
    .zip(moved_zip_code_array.into_iter())
    .collect();

  assert_eq!(map.len(), 3);
  assert_eq!(map.get("palo alto"), Some(&94028));
  assert_eq!(map.get("mountain view"), Some(&94041));
  assert_eq!(map.get("menlo park"), Some(&94025));
}

#[test]
fn test_iterate_map_values() {
  let mut map: HashMap<String, u64> = HashMap::new();
  map.insert("palo alto".to_string(), 94028);
  map.insert("mountain view".to_string(), 94041);
  map.insert("menlo park".to_string(), 94025);

  let place_array: Vec<&String> = map.keys().collect();
  let zip_code_array: Vec<&u64> = map.values().collect();

  for (key, value) in &map {
    assert!(place_array.contains(&key));
    assert!(zip_code_array.contains(&value));
  }
}

#[test]
fn test_insert_into_map_if_key_missing() {
  let mut map: HashMap<String, u64> = HashMap::new();

  let zip_code: &u64 = map.entry("palo alto".to_string()).or_insert(94028);
  assert_eq!(*zip_code, 94028);

  let zip_code: &u64 = map.entry("mountain view".to_string()).or_insert(94041);
  assert_eq!(*zip_code, 94041);

  let zip_code: &u64 = map.entry("menlo park".to_string()).or_insert(94025);
  assert_eq!(*zip_code, 94025);
}

#[test]
fn test_map_update_value_based_on_old_value() {
  let sentence = "hello world wonderful world";
  let mut map: HashMap<String, u64> = HashMap::new();

  for word in sentence.split_whitespace() {
    let map_entry = map.entry(word.to_string());
    let word_count = map_entry.or_insert(0);
    *word_count += 1;
  }

  let map_to_string = format!(
    "{{ world:{}, hello:{}, wonderful:{} }}",
    &map["world"], &map["hello"], &map["wonderful"]
  );
  assert_eq!(map_to_string, "{ world:2, hello:1, wonderful:1 }");
}
