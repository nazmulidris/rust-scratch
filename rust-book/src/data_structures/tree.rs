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
//!
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
//!
//! - Diagram
//!   - <https://github.com/nazmulidris/rust_scratch/blob/main/rust-book/docs/weak-ref.svg>
//!   - [SVG file](../../docs/weak-ref.svg)
//! - <https://doc.rust-lang.org/book/ch15-06-reference-cycles.html#adding-a-reference-from-a-child-to-its-parent>
//! - Thinking about the relationships another way, a parent node should own its children: if a
//!   parent node is dropped, its child nodes should be dropped as well. However, a child should not
//!   own its parent: if we drop a child node, the parent should still exist. This is a case for weak
//!   references!
//!
//! # Other implementations
//!
//! 1. RBTree
//!   - Code:
//!     <https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=9444cbeadcfdbef32c664ae2946e636a>
//!   - SO answer: <https://stackoverflow.com/a/65179837/2085356>
//! 2. Simple: <https://gist.github.com/aidanhs/5ac9088ca0f6bdd4a370>
//!

use core::fmt::Debug;
use std::{
  borrow::{Borrow, BorrowMut},
  cell::RefCell,
  sync::{Arc, Weak},
};

pub fn run() {}

// TODO: impl tree walking, find w/ comparator lambda, and print out the tree.
// TODO: impl delete, easy insert.
// TODO: impl nodelist (find multiple nodes) & return iterator.
// TODO: impl add siblings to node.

// TODO: convert RefCell -> RwLock
type NodeRef<T> = Arc<Node<T>>;
type Parent<T> = RefCell<Weak<Node<T>>>; // not `RefCell<<Rc<Node>>>` which would cause memory leak.
type Children<T> = RefCell<Vec<NodeRef<T>>>;

#[derive(Debug)]
struct Node<T> {
  value: T,
  parent: Parent<T>,
  children: Children<T>,
}

// TODO: start add Tree w/ root & methods.
struct Tree<T> {
  root: NodeRef<T>,
}

impl<T> Tree<T> {
  fn new(root: NodeRef<T>) -> Tree<T> {
    Tree { root }
  }
}
// TODO: end add Tree w/ root & methods.

/// `child_node.parent` is set to weak reference to `parent_node`.
fn set_parent<T>(child: &NodeRef<T>, parent: &NodeRef<T>) {
  *child.parent.borrow_mut() = Arc::downgrade(&parent);
}

fn add_child<T>(child: &NodeRef<T>, parent: &NodeRef<T>) {
  parent.children.borrow_mut().push(child.clone());
}

fn create_node<T>(value: T) -> NodeRef<T> {
  let node = Node {
    value,
    parent: RefCell::new(Weak::new()),  // Basically None.
    children: RefCell::new(Vec::new()), // Basically [].
  };
  let node_ref = Arc::new(node);
  node_ref
}

#[test]
fn test_tree() {
  let child_node: NodeRef<i32> = create_node(3);

  {
    let parent_node: NodeRef<i32> = create_node(5);
    add_child(&child_node, &parent_node);
    set_parent(&child_node, &parent_node);

    assert_eq!(Arc::strong_count(&child_node), 2); // `child_node` has 2 strong references.
    assert_eq!(Arc::weak_count(&child_node), 0);

    assert_eq!(Arc::strong_count(&parent_node), 1); // `parent_node` has 1 strong reference.
    assert_eq!(Arc::weak_count(&parent_node), 1); // `parent_node` also has 1 weak reference.

    assert!(child_node.parent.borrow().upgrade().is_some());
    assert_eq!(child_node.parent.borrow().upgrade().unwrap().value, 5);
  } // `parent_node` is dropped here.

  // `child_node`'s parent is now `None`.
  assert!(child_node.parent.borrow().upgrade().is_none());
  assert_eq!(child_node.value, 3);

  assert_eq!(Arc::strong_count(&child_node), 1); // `child_node` has 1 strong references.
  assert_eq!(Arc::weak_count(&child_node), 0); // `child_node` still has no weak references.
}
