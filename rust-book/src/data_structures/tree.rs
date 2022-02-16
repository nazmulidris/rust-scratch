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

//! # Tree data structure
//! ----------------------------------------------------------------------------
//! - Rust book use of enums that are struct-like:
//!   <https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#:~:text=this%20one%20has%20a%20wide%20variety%20of%20types%20embedded%20in%20its%20variants>
//! - Examples of enums that are struct-like: <https://stackoverflow.com/q/29088633/2085356>
//!   - Approach 1: <https://stackoverflow.com/q/29088633/2085356>
//!   - Approach 2: <https://stackoverflow.com/a/29101091/2085356>
//! - Easy Rust book: <https://fongyoong.github.io/easy_rust/Chapter_25.html>
//! - `From` trait: <https://stackoverflow.com/a/42278050/2085356>
//! - Don't try to write Java in Rust:
//!   <https://users.rust-lang.org/t/passing-self-as-a-parameter/18069>
//!
//! # Weak refs for child's parent (ownership edge vs non-ownership edge)
//! ----------------------------------------------------------------------------
//! - Diagram
//!   - <https://github.com/nazmulidris/rust_scratch/blob/main/rust-book/docs/weak-ref.svg>
//!   - [SVG file](../../docs/weak-ref.svg)
//! - <https://doc.rust-lang.org/book/ch15-06-reference-cycles.html#adding-a-reference-from-a-child-to-its-parent>
//! - Thinking about the relationships another way, a parent node should own its children: if a
//!   parent node is dropped, its child nodes should be dropped as well. However, a child should not
//!   own its parent: if we drop a child node, the parent should still exist. This is a case for weak
//!   references!
//!
//! # RwLock
//! ----------------------------------------------------------------------------
//! - <https://doc.rust-lang.org/std/sync/struct.RwLock.html>
//!
//! # Other implementations
//! ----------------------------------------------------------------------------
//! 1. RBTree
//!   - Code:
//!     <https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=9444cbeadcfdbef32c664ae2946e636a>
//!   - SO answer: <https://stackoverflow.com/a/65179837/2085356>
//! 2. Simple: <https://gist.github.com/aidanhs/5ac9088ca0f6bdd4a370>
//!

// TODO: impl tree walking, find w/ comparator lambda, and print out the tree.
// TODO: impl delete, easy insert.
// TODO: impl nodelist (find multiple nodes) & return iterator.
// TODO: impl add siblings to node.

use core::fmt::Debug;
use rust_book_lib::utils::{print_header, style_dimmed, style_error, style_primary, style_prompt};
use std::{
  borrow::{Borrow, BorrowMut},
  cell::RefCell,
  fmt::{self, Display},
  sync::{Arc, RwLock, Weak},
};

pub fn run() {}

type NodeArcRef<T> = Arc<Node<T>>;
type Children<T> = RwLock<Vec<NodeArcRef<T>>>;
type Parent<T> = RwLock<Weak<Node<T>>>; // not `RwLock<<Rc<Node<T>>>>` which would cause memory leak.

/// This struct is wrapped in an [`Arc`] to allow for multiple owners of the same data. It is never
/// used by itself.
/// See also: [`create_node`](fn@create_node)
struct Node<T>
where
  T: Display,
{
  value: T,
  parent: Parent<T>,
  children: Children<T>,
}

impl<T> fmt::Debug for Node<T>
where
  T: Debug + Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut parent_msg = String::new();
    if let Some(parent) = self.parent.read().unwrap().upgrade() {
      parent_msg.push_str(format!("📦 {}", parent.value).as_str());
    } else {
      parent_msg.push_str("🚫 None");
    }
    f.debug_struct("Node")
      .field("value", &self.value)
      // .field("parent", &self.parent)
      .field("parent", &parent_msg)
      .field("children", &self.children)
      .finish()
  }
}

/// `child.parent` = weak reference to `parent`.
/// 🔏 Note - this acquires a write lock.
fn set_parent<T: Display>(this: &NodeArcRef<T>, parent: &NodeArcRef<T>) {
  {
    let mut childs_parent = this.parent.write().unwrap();
    *childs_parent = Arc::downgrade(&parent);
  } // `childs_parent` guard dropped.
}

/// `parent.children`.push(strong reference to `child`).
/// 🔏 Note - this acquires a write lock.
fn add_child<T: Display>(this: &NodeArcRef<T>, child: &NodeArcRef<T>) {
  {
    let parents_children = &mut this.children.write().unwrap();
    parents_children.push(child.clone());
  } // `parents_children` guard dropped.
  set_parent(&child, &this);
}

fn create_node<T: Display>(value: T) -> NodeArcRef<T> {
  let new_node = Node {
    value,
    parent: RwLock::new(Weak::new()),  // Basically None.
    children: RwLock::new(Vec::new()), // Basically [].
  };
  let node_arc_ref = Arc::new(new_node);
  node_arc_ref
}

/// The parent is backed by a `Weak` ref.
fn has_parent<T: Display>(node: &NodeArcRef<T>) -> bool {
  get_parent(node).is_some()
}

/// The parent is backed by a `Weak` ref.
/// 🔒 Note - this acquires a read lock.
fn get_parent<T: Display>(node: &NodeArcRef<T>) -> Option<NodeArcRef<T>> {
  {
    let parent_weak = node.parent.read().unwrap();
    if let Some(parent) = parent_weak.upgrade() {
      Some(parent)
    } else {
      None
    }
  } // `parent_weak` guard dropped.
}

#[test]
fn test_tree() {
  let child_node = create_node(3);

  {
    let parent_node = create_node(5);
    add_child(&parent_node, &child_node);

    println!("{}: {:#?}", style_primary("parent_node"), parent_node); // Pretty print.
    println!("{}: {:#?}", style_primary("child_node"), child_node); // Pretty print.

    assert_eq!(Arc::strong_count(&child_node), 2); // `child_node` has 2 strong references.
    assert_eq!(Arc::weak_count(&child_node), 0);

    assert_eq!(Arc::strong_count(&parent_node), 1); // `parent_node` has 1 strong reference.
    assert_eq!(Arc::weak_count(&parent_node), 1); // `parent_node` also has 1 weak reference.

    assert!(has_parent(&child_node));
    assert_eq!(get_parent(&child_node).unwrap().value, 5);
  } // `parent_node` is dropped here.

  // `child_node`'s parent is now `None`, its an orphan.
  assert!(!has_parent(&child_node));
  assert_eq!(child_node.value, 3);

  assert_eq!(Arc::strong_count(&child_node), 1); // `child_node` has 1 strong references.
  assert_eq!(Arc::weak_count(&child_node), 0); // `child_node` still has no weak references.
}

// TODO: Create add Tree w/ root & methods.
struct Tree<T>
where
  T: Display,
{
  root: NodeArcRef<T>,
}

impl<T> Tree<T>
where
  T: Display,
{
  fn new(root: NodeArcRef<T>) -> Tree<T> {
    Tree { root }
  }
}
