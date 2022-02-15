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

//! Article: <https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/>
//! Crate: <https://github.com/saschagrunert/indextree>

pub fn run() {}

pub struct Arena<T> {
  nodes: Vec<Node<T>>,
}

pub struct Node<T> {
  parent: Option<NodeId>,
  previous_sibling: Option<NodeId>,
  next_sibling: Option<NodeId>,
  first_child: Option<NodeId>,
  last_child: Option<NodeId>,

  /// The actual data which will be stored within the tree
  pub data: T,
}

pub struct NodeId {
  index: usize,
}

impl<T> Arena<T> {
  pub fn new_node(&mut self, data: T) -> NodeId {
    // Get the next free index
    let next_index = self.nodes.len();

    // Push the node into the arena
    self.nodes.push(Node {
      parent: None,
      first_child: None,
      last_child: None,
      previous_sibling: None,
      next_sibling: None,
      data: data,
    });

    // Return the node identifier
    NodeId { index: next_index }
  }
}
