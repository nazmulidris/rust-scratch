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

use super::arena::Arena;
use super::{Node, ReadGuarded, ResultUidList, ShreableArena, WalkerFn};
use std::fmt::Debug;
use std::marker::{Send, Sync};
use std::sync::{Arc, RwLock};
use std::thread::{self, spawn};

#[derive(Debug)]
pub struct MTArena<T: Debug> {
  arena_arc: ShreableArena<T>,
}

impl<T: 'static + Debug + Send + Sync> MTArena<T> {
  pub fn new() -> Self {
    MTArena {
      arena_arc: Arc::new(RwLock::new(Arena::new())),
    }
  }

  pub fn get_arena_arc(&self) -> ShreableArena<T> {
    self.arena_arc.clone()
  }

  /// Note that the `walker_fn` does not capture any variables. It is a function pointer and not a
  /// lambda. Also note that this function executes in a new thread in parallel.
  pub fn tree_walk_parallel(
    &self,
    node_id: usize,
    walker_fn: WalkerFn<T>,
  ) -> thread::JoinHandle<ResultUidList> {
    let arc = self.get_arena_arc();
    spawn(move || {
      let read_guard: ReadGuarded<Arena<T>> = arc.read().unwrap();
      let return_value = read_guard.tree_walk_dfs(node_id);

      // While walking the tree, in a separate thread, call the `walker_fn` for each node.
      if let Some(result_list) = return_value.clone() {
        result_list.clone().into_iter().for_each(|uid| {
          let node_arc_opt = read_guard.get_arc_to_node(uid);
          if let Some(node_arc) = node_arc_opt {
            let node_ref: ReadGuarded<Node<T>> = node_arc.read().unwrap();
            walker_fn(uid, node_ref);
          }
        });
      }

      return_value
    })
  }
}
