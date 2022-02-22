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
use super::{ReadGuarded, ResultUidList, ShreableArena};
use std::fmt::Debug;
use std::marker::{Send, Sync};
use std::sync::{Arc, RwLock};
use std::thread;

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

  pub fn tree_walk_parallel(
    &self,
    node_id: usize,
  ) -> thread::JoinHandle<ResultUidList> {
    let arc = self.get_arena_arc();
    thread::spawn(move || {
      let read_guard: ReadGuarded<Arena<T>> = arc.read().unwrap();
      read_guard.tree_walk_dfs(node_id)
    })
  }
}
