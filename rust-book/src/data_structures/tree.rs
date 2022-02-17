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

use core::fmt::Debug;
use rust_book_lib::utils::{print_header, style_dimmed, style_error, style_primary, style_prompt};
use std::{
  borrow::{Borrow, BorrowMut},
  cell::RefCell,
  fmt::{self, Display},
  sync::{Arc, RwLock, Weak},
};

pub fn run() {}

type NodeRef<T> = Arc<Node<T>>;
type WeakNodeRef<T> = Weak<Node<T>>;
type Children<T> = RwLock<Vec<NodeRef<T>>>;
type Parent<T> = RwLock<WeakNodeRef<T>>; // not `RwLock<<Rc<Node<T>>>>` which would cause memory leak.

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

impl<T> Node<T>
where
  T: Display,
{
  fn new(value: T) -> Self {
    Node {
      value,
      parent: RwLock::new(Weak::new()),
      children: RwLock::new(Vec::new()),
    }
  }

  /// 🔏 Note - this acquires a write lock.
  fn set_parent(self: &Self, parent: &NodeRef<T>) {
    let mut my_parent = self.parent.write().unwrap();
    *my_parent = Arc::downgrade(parent);
  } // `my_parent` guard dropped.

  /// 🔏 Note - this acquires a write lock.
  fn add_child(self: &Self, child: &NodeRef<T>) {
    let mut my_children = self.children.write().unwrap();
    my_children.push(Arc::clone(child));
  } // `my_children` guard dropped.

  /// The parent is backed by a `Weak` ref.
  /// 🔒 Note - this acquires a read lock.
  fn get_parent(self: &Self) -> Option<NodeRef<T>> {
    let my_parent_weak = self.parent.read().unwrap();
    my_parent_weak.upgrade()
  } // `my_parent_weak` guard dropped.

  fn has_parent(self: &Self) -> bool {
    self.get_parent().is_some()
  }
}

#[derive(Debug)]
struct NodeRefWrapper<T: Display> {
  my_ref: NodeRef<T>,
}
impl<T> NodeRefWrapper<T>
where
  T: Display,
{
  fn new(value: T) -> NodeRefWrapper<T> {
    let new_node = Node::new(value);
    let node_arc_ref = Arc::new(new_node);
    NodeRefWrapper {
      my_ref: node_arc_ref,
    }
  }

  /// This can't be moved into a (non-static) method of `Node` because it needs a `&NodeRef<T>`
  /// which isn't available in `Node`.
  fn add_child_and_update_its_parent(self: &Self, child: &NodeRefWrapper<T>) {
    self.my_ref.add_child(child.my_ref.borrow());
    child.my_ref.set_parent(self.my_ref.borrow());
  }
}

#[test]
fn test_tree_low_level_node_manipulation() {
  let child_node = NodeRefWrapper::new(3);

  {
    let parent_node = NodeRefWrapper::new(5);
    parent_node.add_child_and_update_its_parent(&child_node);
    // NodeRefWrapper::add_child_and_update_its_parent(&parent_node.my_ref, &child_node.my_ref);

    println!("{}: {:#?}", style_primary("parent_node"), parent_node); // Pretty print.
    println!("{}: {:#?}", style_primary("child_node"), child_node); // Pretty print.

    assert_eq!(Arc::strong_count(&child_node.my_ref), 2); // `child_node` has 2 strong references.
    assert_eq!(Arc::weak_count(&child_node.my_ref), 0);

    assert_eq!(Arc::strong_count(&parent_node.my_ref), 1); // `parent_node` has 1 strong reference.
    assert_eq!(Arc::weak_count(&parent_node.my_ref), 1); // `parent_node` also has 1 weak reference.

    assert!(child_node.my_ref.has_parent());
    assert_eq!(child_node.my_ref.get_parent().unwrap().value, 5);
  } // `parent_node` is dropped here.

  // `child_node`'s parent is now `None`, its an orphan.
  assert!(!child_node.my_ref.has_parent());
  assert_eq!(child_node.my_ref.value, 3);

  assert_eq!(Arc::strong_count(&child_node.my_ref), 1); // `child_node` has 1 strong references.
  assert_eq!(Arc::weak_count(&child_node.my_ref), 0); // `child_node` still has no weak references.
}

// TODO: impl tree walking, find w/ comparator lambda, and print out the tree.
// TODO: impl delete, easy insert.
// TODO: impl nodelist (find multiple nodes) & return iterator.
// TODO: impl add siblings to node.

#[derive(Debug)]
struct Tree<T>
where
  T: Display,
{
  root: NodeRefWrapper<T>,
}

impl<T> Tree<T>
where
  T: Display,
{
  fn new(value: T) -> Tree<T> {
    Tree {
      root: NodeRefWrapper::new(value),
    }
  }
  fn add_child(&self, value: T) -> NodeRef<T> {
    let child_node = NodeRefWrapper::new(value);
    self.root.add_child_and_update_its_parent(&child_node);
    child_node.my_ref.clone()
  }
}

#[test]
fn test_tree_simple_api() {
  let tree = Tree::new(5);
  let child_node = tree.add_child(3);
  assert_eq!(child_node.value, 3);
  assert_eq!(tree.root.my_ref.value, 5);
  assert_eq!(tree.root.my_ref.children.read().unwrap().len(), 1);
  assert_eq!(
    child_node.value,
    tree.root.my_ref.children.read().unwrap()[0].value
  );
  println!("{}: {:#?}", style_primary("tree"), tree); // Pretty print.
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
