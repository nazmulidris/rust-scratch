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

//! Rust Book, module re-exports:
//! - <https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#documentation-comments-as-tests>

pub mod arena;
pub mod arena_types;
pub mod mt_arena;

// Re-export the following modules:
pub use arena::*; // Arena.
pub use arena_types::*; // HasId, Node, NodeRef, WeakNodeRef, ReadGuarded, WriteGuarded, ArenaMap, FilterFn, ResultUidList
pub use mt_arena::*; // MTArena.
