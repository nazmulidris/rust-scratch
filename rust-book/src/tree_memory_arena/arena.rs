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

//! Data structure to store & manipulate a (non-binary) tree of data in memory. The `Arena` can be
//! used to implement a pletohora of different data structures. The non-binary tree is just an
//! example.
//!
//! Wikipedia: <https://en.wikipedia.org/wiki/Region-based_memory_management>
//! rustc lint warnings: <https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html>

use std::{
  collections::HashMap,
  sync::{atomic::AtomicUsize, Arc, RwLock, Weak},
};

use super::id::{HasId, Uid};

// Node.
#[derive(Debug)]
pub struct Node<T> {
  id: usize,
  parent: Option<Uid>,
  children: Option<Vec<Uid>>,
  pub data: T,
}

impl<T> HasId for Node<T> {
  fn get_id(&self) -> usize {
    self.id
  }

  fn get_copy_of_id(&self) -> Uid {
    Uid::new(self.id)
  }
}

// Type aliases for readability.
type Id = dyn HasId;
type NodeRef<T> = Arc<RwLock<Node<T>>>;
type WeakNodeRef<T> = Weak<RwLock<Node<T>>>;

// Arena.
#[derive(Debug)]
pub struct Arena<T> {
  map: RwLock<HashMap<usize, NodeRef<T>>>,
  atomic_counter: AtomicUsize,
}

// TODO: implement search / filter given lambda.
// TODO: delete node (and its children) given id.
impl<T> Arena<T> {
  /// DFS graph walking: <https://developerlife.com/2018/08/16/algorithms-in-kotlin-5/>
  /// DFS tree walking: <https://stephenweiss.dev/algorithms-depth-first-search-dfs#handling-non-binary-trees>
  pub fn tree_walk_dfs(&self, node_id: &Id) -> Option<Vec<Uid>> {
    let mut result: Vec<Uid> = vec![];
    let mut stack: Vec<Uid> = vec![];
    stack.push(node_id.get_copy_of_id());

    while let Some(node_id) = stack.pop() {
      let node_ref = self.get_node_ref_strong(&node_id)?;
      let node = node_ref.read().unwrap();

      result.push(node.get_copy_of_id());

      if let Some(children) = node.children.as_ref() {
        children
          .into_iter()
          .for_each(|child_id| stack.push(child_id.get_copy_of_id()));
      }
    } // End while stack not empty.

    match result.len() {
      0 => None,
      _ => Some(result),
    }
  }

  pub fn get_node_ref_weak(&self, node_id: &Id) -> Option<WeakNodeRef<T>> {
    let id = node_id.get_id();
    let map = self.map.read().unwrap();
    let node_ref = map.get(&id)?;
    Some(Arc::downgrade(&node_ref))
  }

  pub fn get_node_ref_strong(&self, node_id: &Id) -> Option<NodeRef<T>> {
    let id = node_id.get_id();
    let map = self.map.read().unwrap();
    let node_ref = map.get(&id)?;
    Some(Arc::clone(&node_ref))
  }

  pub fn new_node(&mut self, data: T, parent: Option<&Id>) -> impl HasId {
    let new_node_id = self.generate_uid();

    // Push a new node into arena.
    let push_new_node_into_arena = || {
      // Push the node into the arena.
      let id = new_node_id.get_id();
      self.map.write().unwrap().insert(
        id,
        Arc::new(RwLock::new(Node {
          id,
          parent: None,
          children: None,
          data,
        })),
      );
    };
    push_new_node_into_arena();

    // Has parent?
    if let Some(parent_id) = parent {
      let parent_node_ref = self.get_node_ref_strong(parent_id).unwrap();
      let mut parent_node = parent_node_ref.write().unwrap();
      let mut children = parent_node.children.take().unwrap_or_default();
      children.push(new_node_id.get_copy_of_id());
      parent_node.children = Some(children);
    }

    // Return the node identifier.
    return new_node_id;
  }

  fn generate_uid(&self) -> impl HasId {
    Uid::new(
      self
        .atomic_counter
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
    )
  }

  pub fn new() -> Self {
    Arena {
      map: RwLock::new(HashMap::new()),
      atomic_counter: AtomicUsize::new(0),
    }
  }
}
