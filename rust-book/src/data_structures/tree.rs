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

use std::{
  borrow::BorrowMut,
  cell::RefCell,
  rc::{Rc, Weak},
};

pub fn run() {}

/// 📂📄 File and Folder data structure.
/// - Rust book use of enums that are struct-like: <https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#:~:text=this%20one%20has%20a%20wide%20variety%20of%20types%20embedded%20in%20its%20variants>
/// - Examples of enums that are struct-like: <https://stackoverflow.com/q/29088633/2085356>
///   - Approach 1: <https://stackoverflow.com/q/29088633/2085356>
///   - Approach 2: <https://stackoverflow.com/a/29101091/2085356>
/// - Easy Rust book: <https://fongyoong.github.io/easy_rust/Chapter_25.html>
/// - `From` trait: <https://stackoverflow.com/a/42278050/2085356>

#[derive(Debug)]
enum Node {
  File {
    name: String,
  },
  Folder {
    name: String,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
  },
}

impl From<&str> for Node {
  fn from(name: &str) -> Self {
    Node::File {
      name: name.to_string(),
    }
  }
}

impl From<(&str, RefCell<Weak<Node>>, RefCell<Vec<Rc<Node>>>)> for Node {
  fn from((name, parent, children): (&str, RefCell<Weak<Node>>, RefCell<Vec<Rc<Node>>>)) -> Self {
    Node::Folder {
      name: name.to_string(),
      parent,
      children,
    }
  }
}

impl Node {
  fn to_rc_ptr(self: Self) -> Rc<Self> {
    Rc::new(self)
  }

  fn new_file(name: &str) -> Rc<Self> {
    Node::from(name).to_rc_ptr()
    // Equivalent to:
    // let new_file = name.into();
    // new_file.to_rc_ptr()
  }

  fn new_folder(name: &str) -> Rc<Self> {
    let parent = RefCell::new(Weak::new());
    let children = RefCell::new(vec![]);
    Node::from((name, parent, children)).to_rc_ptr()
    // Equivalent to:
    // let new_folder = (name, parent, children).into();
    // new_folder.to_rc_ptr()
  }

  fn get_name(self: &Self) -> &String {
    match self {
      Node::File { name } => &name,
      Node::Folder { name, .. } => &name,
    }
  }

  fn get_parent(self: &Self) -> Option<Rc<Node>> {
    match self {
      Node::Folder { parent, .. } => parent.borrow().upgrade(),
      Node::File { .. } => None,
    }
  }

  fn get_children(self: &Self) -> Option<Vec<Rc<Node>>> {
    match self {
      Node::Folder { children, .. } => Some(children.borrow().clone()),
      Node::File { .. } => None,
    }
  }

  fn add_child(self: &Self, child: &Rc<Node>) {
    match self {
      Node::Folder {
        children,
        parent,
        name,
      } => {
        children.borrow_mut().push(child.clone());
        // *child.parent.borrow_mut() = Rc::downgrade(&self);
      }
      Node::File { .. } => {}
    }
  }
}

#[test]
fn test_can_create_file_variant_using_into() {
  let file: Node = "file".into();

  assert!(matches!(&file, Node::File { .. }));

  match &file {
    Node::File { name } => assert_eq!(name, "file"),
    _ => panic!("Expected file variant"),
  }

  if let Node::File { name } = &file {
    assert_eq!(name, "file");
  } else {
    panic!("Expected file variant");
  }
}

#[test]
fn test_can_create_folder_variant_using_into() {
  let folder = ("folder", RefCell::new(Weak::new()), RefCell::new(vec![])).into();

  assert!(matches!(folder, Node::Folder { .. }));

  match &folder {
    Node::Folder {
      name,
      parent,
      children,
    } => {
      assert_eq!(name, "folder");
      assert_eq!(parent.borrow().weak_count(), 0);
      assert_eq!(parent.borrow().strong_count(), 0);
      assert_eq!(children.borrow().len(), 0);
    }
    _ => panic!("Expected folder variant"),
  }

  if let Node::Folder { name, .. } = &folder {
    assert_eq!(name, "folder")
  } else {
    panic!("Expected folder variant")
  }
}

#[test]
fn test_can_convert_both_variants_to_rc() {
  {
    let file_rc_ptr = Node::new_file("file");
    // Equivalent to:
    // let file: Node = "file".into();
    // let file_rc_ptr: Rc<Node> = file.to_rc_ptr();

    assert!(matches!(*file_rc_ptr, Node::File { .. }));
    assert_eq!(file_rc_ptr.get_name(), "file");
    assert_eq!(*file_rc_ptr.get_name(), "file");
  }

  {
    let folder_rc_ptr = Node::new_folder("folder");
    // Equivalent to:
    // let folder: Node = ("folder", RefCell::new(Weak::new()), RefCell::new(vec![])).into();
    // let folder_rc_ptr: Rc<Node> = folder.to_rc_ptr();

    assert!(matches!(*folder_rc_ptr, Node::Folder { .. }));
    assert_eq!(folder_rc_ptr.get_name(), "folder");
    assert_eq!(*folder_rc_ptr.get_name(), "folder");
  }
}

#[test]
fn test_can_modify_children_of_folder() {
  let root_dir = Node::new_folder("root");
  let root_dir_file_1 = Node::new_file("root_dir_file_1");
  let root_dir_file_2 = Node::new_file("root_dir_file_2");

  let user_dir = Node::new_folder("user");
  let user_dir_file_1 = Node::new_file("user_dir_file1");
  let user_dir_file_2 = Node::new_file("user_dir_file2");

  // Put 2 files in root dir.
  root_dir.add_child(&root_dir_file_1);
  root_dir.add_child(&root_dir_file_2);

  // Put 2 files in user dir.
  user_dir.add_child(&user_dir_file_1);
  user_dir.add_child(&user_dir_file_2);

  // Put user dir in root dir.
  root_dir.add_child(&user_dir);

  // Check that root dir has 2 files and user dir has 2 files.
  assert_eq!(root_dir.get_children().unwrap().len(), 3);
  assert_eq!(user_dir.get_children().unwrap().len(), 2);

  // TODO: this is not working.
  assert_eq!(user_dir.get_parent().unwrap().get_name(), "root");
}
