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
  sync::{atomic::AtomicUsize, Arc, RwLock},
};

use super::{
  arena_types::HasId, ArenaMap, FilterFn, NodeRef, ReadGuarded, ResultUidList,
  WeakNodeRef, WriteGuarded,
};

// Node.
#[derive(Debug)]
pub struct Node<T>
where
  T: Debug + Clone,
{
  pub id: usize,
  pub parent: Option<usize>,
  pub children: Vec<usize>,
  pub payload: T,
}

impl<T> HasId for Node<T>
where
  T: Debug + Clone,
{
  fn get_id(&self) -> usize {
    self.id
  }
}

// Arena.
#[derive(Debug)]
pub struct Arena<T>
where
  T: Debug + Clone,
{
  map: RwLock<ArenaMap<T>>,
  atomic_counter: AtomicUsize,
}

impl<T> Arena<T>
where
  T: Debug + Clone,
{
  /// If no matching nodes can be found returns `None`.
  pub fn filter_all_nodes_by(
    &self,
    filter_fn: &FilterFn<T>,
  ) -> ResultUidList {
    let map: ReadGuarded<ArenaMap<T>> = self.map.read().unwrap();
    let filtered_map = map
      .iter()
      .filter(|(id, node_ref)| filter_fn(**id, node_ref.read().unwrap().payload.clone()))
      .map(|(id, _node_ref)| *id)
      .collect::<Vec<usize>>();
    match filtered_map.len() {
      0 => None,
      _ => Some(filtered_map),
    }
  }

  /// If `node_id` can't be found, returns `None`.
  pub fn get_children_of(
    &self,
    node_id: usize,
  ) -> ResultUidList {
    // Early return if `node_id` can't be found.
    let node_to_lookup = self.get_arc_to_node(node_id)?;
    let node_to_lookup: ReadGuarded<Node<T>> = node_to_lookup.read().unwrap(); // Safe to call unwrap.
    let children_uids = &node_to_lookup.children;
    Some(children_uids.clone())
  }

  /// If `node_id` can't be found, returns `None`.
  pub fn get_parent_of(
    &self,
    node_id: usize,
  ) -> Option<usize> {
    // Early return if `node_id` can't be found.
    let node_to_lookup = self.get_arc_to_node(node_id)?;
    let node_to_lookup: ReadGuarded<Node<T>> = node_to_lookup.read().unwrap(); // Safe to call unwrap.
    return node_to_lookup.parent.clone();
  }

  /// If `node_id` can't be found, returns `None`.
  pub fn delete_node(
    &self,
    node_id: usize,
  ) -> ResultUidList {
    // Early return if `node_id` can't be found.
    let deletion_list = self.tree_walk_dfs(node_id)?;

    // If `node_id` has a parent, remove `node_id` its children, otherwise skip this step.
    if let Some(parent_uid) = self.get_parent_of(node_id) {
      let parent_node = self.get_arc_to_node(parent_uid);
      if let Some(parent_node) = parent_node {
        let mut writeable_parent_node: WriteGuarded<Node<T>> =
          parent_node.write().unwrap(); // Safe to call unwrap.
        writeable_parent_node
          .children
          .retain(|child_id| *child_id != node_id);
      }
    }

    let mut map: WriteGuarded<ArenaMap<T>> = self.map.write().unwrap(); // Safe to unwrap.
    deletion_list.iter().for_each(|id| {
      map.remove(id);
    });

    Some(deletion_list.clone())
  }

  /// DFS graph walking: <https://developerlife.com/2018/08/16/algorithms-in-kotlin-5/>
  /// DFS tree walking: <https://stephenweiss.dev/algorithms-depth-first-search-dfs#handling-non-binary-trees>
  pub fn tree_walk_dfs(
    &self,
    node_id: usize,
  ) -> ResultUidList {
    let mut collected_nodes: Vec<usize> = vec![];
    let mut stack: Vec<usize> = vec![node_id];

    while let Some(node_id) = stack.pop() {
      // Question mark operator works below, since it returns a `Option<T>` to `while let ...`.
      // Basically skip to the next item in the `stack` if `node_id` can't be found.
      let node_ref = self.get_arc_to_node(node_id)?;
      node_ref.read().ok().map(|node: ReadGuarded<Node<T>>| {
        collected_nodes.push(node.get_id());
        stack.extend(node.children.iter().cloned());
      });
    }

    match collected_nodes.len() {
      0 => None,
      _ => Some(collected_nodes),
    }
  }

  /// If `node_id` can't be found, returns `None`.
  pub fn get_weak_ref_to_node(
    &self,
    node_id: usize,
  ) -> Option<WeakNodeRef<T>> {
    let id = node_id;
    let map: ReadGuarded<ArenaMap<T>> = self.map.read().ok()?;
    let node_ref = map.get(&id)?;
    Some(Arc::downgrade(&node_ref))
  }

  /// If `node_id` can't be found, returns `None`.
  pub fn get_arc_to_node(
    &self,
    node_id: usize,
  ) -> Option<NodeRef<T>> {
    let id = node_id;
    let map: ReadGuarded<ArenaMap<T>> = self.map.read().ok()?;
    let node_ref = map.get(&id)?;
    Some(Arc::clone(&node_ref))
  }

  pub fn add_new_node(
    &mut self,
    data: T,
    parent_id: Option<usize>,
  ) -> usize {
    let new_node_id = self.generate_uid();

    let push_new_node_into_arena = || {
      let id = new_node_id;
      let mut map: WriteGuarded<ArenaMap<T>> = self.map.write().unwrap(); // Safe to unwrap.
      map.insert(
        id,
        Arc::new(RwLock::new(Node {
          id,
          parent: match parent_id {
            Some(parent_id) => Some(parent_id),
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
        parent_node.children.push(new_node_id);
      }
    }

    // Return the node identifier.
    return new_node_id;
  }

  fn generate_uid(&self) -> usize {
    self
      .atomic_counter
      .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
  }

  pub fn new() -> Self {
    Arena {
      map: RwLock::new(HashMap::new()),
      atomic_counter: AtomicUsize::new(0),
    }
  }
}
