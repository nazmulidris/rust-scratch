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
  fmt::Debug,
  sync::{atomic::AtomicUsize, Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

use super::id::{HasId, Uid};

// Node.
#[derive(Debug)]
pub struct Node<T>
where
  T: Debug,
{
  pub id: usize,
  pub parent: Option<Uid>,
  pub children: Vec<Uid>,
  pub payload: T,
}

impl<T> HasId for Node<T>
where
  T: Debug,
{
  fn get_id(&self) -> usize {
    self.id
  }

  fn get_uid(&self) -> Uid {
    Uid::new(self.id)
  }
}

// Type aliases for readability.
pub type Id = dyn HasId;
pub type NodeRef<T> = Arc<RwLock<Node<T>>>;
pub type WeakNodeRef<T> = Weak<RwLock<Node<T>>>;
pub type ReadGuarded<'a, T> = RwLockReadGuard<'a, T>;
pub type WriteGuarded<'a, T> = RwLockWriteGuard<'a, T>;
pub type ArenaMap<T> = HashMap<usize, NodeRef<T>>;

// Arena.
#[derive(Debug)]
pub struct Arena<T>
where
  T: Debug,
{
  map: RwLock<ArenaMap<T>>,
  atomic_counter: AtomicUsize,
}

// TODO: create parallel tree_walk_dfs that returns a handle.
// TODO: implement search / filter given lambda.
impl<T> Arena<T>
where
  T: Debug,
{
  pub fn get_children_of(&self, node_id: &Id) -> Vec<Uid> {
    if let Some(node_to_lookup) = self.get_arc_to_node(node_id) {
      let node_to_lookup: ReadGuarded<Node<T>> = node_to_lookup.read().unwrap(); // Safe to call unwrap.
      let children_uids = &node_to_lookup.children;
      return children_uids.clone();
    }
    Vec::new()
  }

  pub fn get_parent_of(&self, node_id: &Id) -> Option<Uid> {
    if let Some(node_to_lookup) = self.get_arc_to_node(node_id) {
      let node_to_lookup: ReadGuarded<Node<T>> = node_to_lookup.read().unwrap(); // Safe to call unwrap.
      let parent_uid_opt = &node_to_lookup.parent;
      if parent_uid_opt.is_some() {
        return parent_uid_opt.clone();
      }
    }
    None
  }

  pub fn delete_node(&self, node_id: &Id) -> Vec<Uid> {
    let deletion_list = self.tree_walk_dfs(node_id).unwrap_or(vec![]);

    if let Some(parent_uid) = self.get_parent_of(node_id) {
      let parent_node = self.get_arc_to_node(&parent_uid);
      if let Some(parent_node) = parent_node {
        let mut writeable_parent_node: WriteGuarded<Node<T>> = parent_node.write().unwrap(); // Safe to call unwrap.
        writeable_parent_node
          .children
          .retain(|child_id| child_id.get_id() != node_id.get_id());
      }
    }

    let mut map: WriteGuarded<ArenaMap<T>> = self.map.write().unwrap(); // Safe to unwrap.
    deletion_list.iter().for_each(|id| {
      map.remove(&id.get_id());
    });

    deletion_list.clone()
  }

  /// DFS graph walking: <https://developerlife.com/2018/08/16/algorithms-in-kotlin-5/>
  /// DFS tree walking: <https://stephenweiss.dev/algorithms-depth-first-search-dfs#handling-non-binary-trees>
  pub fn tree_walk_dfs(&self, node_id: &Id) -> Option<Vec<Uid>> {
    let mut collected_nodes: Vec<Uid> = vec![];
    let mut stack: Vec<Uid> = vec![node_id.get_uid()];

    while let Some(node_id) = stack.pop() {
      // Question mark operator works below, since it returns a `Option<T>` to `while let ...`.
      // Basically early return if `node_id` can't be found.
      let node_ref = self.get_arc_to_node(&node_id)?;
      node_ref.read().ok().map(|node: ReadGuarded<Node<T>>| {
        collected_nodes.push(node.get_uid());
        stack.extend(node.children.iter().cloned());
      });
    }

    match collected_nodes.len() {
      0 => None,
      _ => Some(collected_nodes),
    }
  }

  /// If `node_id` can't be found, returns `None`.
  pub fn get_weak_ref_to_node(&self, node_id: &Id) -> Option<WeakNodeRef<T>> {
    let id = node_id.get_id();
    let map: ReadGuarded<ArenaMap<T>> = self.map.read().ok()?;
    let node_ref = map.get(&id)?;
    Some(Arc::downgrade(&node_ref))
  }

  /// If `node_id` can't be found, returns `None`.
  pub fn get_arc_to_node(&self, node_id: &Id) -> Option<NodeRef<T>> {
    let id = node_id.get_id();
    let map: ReadGuarded<ArenaMap<T>> = self.map.read().ok()?;
    let node_ref = map.get(&id)?;
    Some(Arc::clone(&node_ref))
  }

  pub fn add_new_node(&mut self, data: T, parent_id: Option<&Id>) -> impl HasId {
    let new_node_id = self.generate_uid();

    let push_new_node_into_arena = || {
      let id = new_node_id.get_id();
      let mut map: WriteGuarded<ArenaMap<T>> = self.map.write().unwrap(); // Safe to unwrap.
      map.insert(
        id,
        Arc::new(RwLock::new(Node {
          id,
          parent: match parent_id {
            Some(parent_id) => Some(parent_id.get_uid()),
            None => None,
          },
          children: vec![],
          payload: data,
        })),
      );
    };
    push_new_node_into_arena();

    if let Some(parent_id) = parent_id {
      if let Some(parent_node_ref) = self.get_arc_to_node(parent_id) {
        let mut parent_node: WriteGuarded<Node<T>> = parent_node_ref.write().unwrap(); // Safe to unwrap.
        parent_node.children.push(new_node_id.get_uid());
      }
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
