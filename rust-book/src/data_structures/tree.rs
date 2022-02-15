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

//! # File and Folder data structure
//! - Rust book use of enums that are struct-like: <https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#:~:text=this%20one%20has%20a%20wide%20variety%20of%20types%20embedded%20in%20its%20variants>
//! - Examples of enums that are struct-like: <https://stackoverflow.com/q/29088633/2085356>
//!   - Approach 1: <https://stackoverflow.com/q/29088633/2085356>
//!   - Approach 2: <https://stackoverflow.com/a/29101091/2085356>
//! - Easy Rust book: <https://fongyoong.github.io/easy_rust/Chapter_25.html>
//! - `From` trait: <https://stackoverflow.com/a/42278050/2085356>
//! - Don't try to write Java in Rust: <https://users.rust-lang.org/t/passing-self-as-a-parameter/18069>
//!
//! # Visualization
//! ![](../../docs/weak-ref.svg)
use core::fmt::Debug;
use std::{
  borrow::{Borrow, BorrowMut},
  cell::RefCell,
  rc::{Rc, Weak},
};

pub fn run() {}

// Use type aliases to enhance readability.
type Parent = RefCell<WeakNodeRef>;
type Children = RefCell<Vec<NodeRef>>;
type WeakNodeRef = Weak<dyn NodeIF>;
type NodeRef = Rc<dyn NodeIF>;

struct FileNode {
  name: String,
}

struct FolderNode {
  name: String,
  parent: Parent,
  children: Children,
}

trait NodeIF {
  fn get_name(self: &Self) -> &str;
  fn get_parent(self: &Self) -> Parent;
  fn get_children(self: &Self) -> Children;
}

impl NodeIF for FileNode {
  fn get_name(self: &Self) -> &str {
    &self.name
  }

  fn get_parent(self: &Self) -> Parent {
    RefCell::new(Weak::<FileNode>::new())
  }

  fn get_children(self: &Self) -> Children {
    RefCell::new(Vec::<NodeRef>::new())
  }
}

impl FileNode {
  fn new(name: &str) -> NodeRef {
    Rc::new(Self {
      name: name.to_string(),
    })
  }
}

impl NodeIF for FolderNode {
  fn get_name(self: &Self) -> &str {
    &self.name
  }

  fn get_parent(self: &Self) -> Parent {
    self.parent.clone()
  }

  fn get_children(self: &Self) -> Children {
    self.children.clone()
  }
}

impl FolderNode {
  fn new(name: &str) -> NodeRef {
    let parent = RefCell::new(Weak::<FolderNode>::new());
    let children = RefCell::new(vec![]);
    Rc::new(Self {
      name: name.to_string(),
      parent,
      children,
    })
  }

  fn add_child(self: &Self, child: &NodeRef) {
    self.children.borrow_mut().push(child.clone());
  }
}

impl Debug for dyn NodeIF {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "\n  NodeIF -> {{ name: {}, parent: {:?}, children: {:?} }}",
      self.get_name(),
      self.get_parent().borrow().weak_count(),
      self.get_children().borrow().len()
    )
  }
}

#[test]
fn test_can_create_file() {
  let f_1: NodeRef = FileNode::new("file");
  assert_eq!(f_1.get_name(), "file");
  assert!(f_1.get_parent().borrow().upgrade().is_none());
  assert!(f_1.get_children().borrow().is_empty());
}

#[test]
fn test_can_create_folder() {
  let f_2: NodeRef = FolderNode::new("folder");
  assert_eq!(f_2.get_name(), "folder");
  assert!(f_2.get_parent().borrow().upgrade().is_none());
  assert!(f_2.get_children().borrow().is_empty());
}

#[test]
fn test_can_manipulate_tree() {
  let root_dir_rc: NodeRef = FolderNode::new("root");
  let user_dir_rc: NodeRef = FolderNode::new("user");

  {
    let file1_rc: NodeRef = FileNode::new("root_dir_file_1");
    let file2_rc: NodeRef = FileNode::new("root_dir_file_2");

    let file3_rc: NodeRef = FileNode::new("user_dir_file3");
    let file4_rc: NodeRef = FileNode::new("user_dir_file4");

    // Put 2 files in root dir.
    {
      Tree::add_child(&file1_rc, &root_dir_rc);
      Tree::add_child(&file2_rc, &root_dir_rc);
    }

    // Put 2 files in user dir.
    {
      Tree::add_child(&file3_rc, &user_dir_rc);
      Tree::add_child(&file4_rc, &user_dir_rc);
    }
    // Put user dir in root dir.
    {
      Tree::add_child(&user_dir_rc, &root_dir_rc);
    }

    // Assertions.
    assert!(file1_rc.get_parent().borrow().upgrade().is_none());
    assert!(file2_rc.get_parent().borrow().upgrade().is_none());
    assert!(file3_rc.get_parent().borrow().upgrade().is_none());
    assert!(file4_rc.get_parent().borrow().upgrade().is_none());
  }

  // Assertions.
  assert_eq!(root_dir_rc.get_children().borrow().len(), 3);
  assert_eq!(user_dir_rc.get_children().borrow().len(), 2);

  assert!(root_dir_rc.get_parent().borrow().upgrade().is_none());

  assert!(user_dir_rc.get_parent().borrow().upgrade().is_some());
  assert!(
    user_dir_rc
      .get_parent()
      .borrow()
      .upgrade()
      .as_ref()
      .unwrap()
      .get_name()
      == "root"
  );
}

// TODO: The following struct might be used to make a cleaner API.
pub struct Tree {
  root: FolderNode,
}

impl Tree {
  fn add_child(child_rc: &NodeRef, parent_rc: &NodeRef) {
    parent_rc.get_children().borrow_mut().push(child_rc.clone());
    // child.set_parent(parent)
    *child_rc.get_parent().borrow_mut() = Rc::downgrade(parent_rc);
  }
}
