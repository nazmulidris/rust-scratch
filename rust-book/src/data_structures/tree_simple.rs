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

//! <https://gist.github.com/aidanhs/5ac9088ca0f6bdd4a370>

pub fn run() {}

struct Node {
  value: i32,
  left: Option<Box<Node>>,
  right: Option<Box<Node>>,
}

impl Node {
  fn new(value: i32) -> Node {
    Node {
      value: value,
      left: None,
      right: None,
    }
  }

  fn insert(&mut self, value: i32) {
    let new_node = Some(Box::new(Node::new(value)));
    if value < self.value {
      match self.left.as_mut() {
        None => self.left = new_node,
        Some(left) => left.insert(value),
      }
    } else {
      match self.right.as_mut() {
        None => self.right = new_node,
        Some(right) => right.insert(value),
      }
    }
  }

  fn search(&self, target: i32) -> Option<i32> {
    match self.value {
      value if target == value => Some(value),
      value if target < value => self.left.as_ref()?.search(target),
      value if target > value => self.right.as_ref()?.search(target),
      _ => None,
    }
  }
}
