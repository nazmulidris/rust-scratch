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

//! This code does not work and is included here as a pedagogical example. Trying to write Java code
//! in Rust only leads to [misery](https://users.rust-lang.org/t/passing-self-as-a-parameter/18069).

use std::{
  borrow::{Borrow, BorrowMut},
  cell::RefCell,
  rc::{Rc, Weak},
};

pub fn run() {}

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
  /// Returns `Rc` that owns `self`. This `Rc` can be cloned for shared ownership.
  fn new_file(name: &str) -> Rc<Self> {
    Node::from(name).move_self_to_rc()
    // Same as: `name.into().move_to_rc();`
  }

  /// Returns `Rc` that owns `self`. This `Rc` can be cloned for shared ownership.
  fn new_folder(name: &str) -> Rc<Self> {
    let parent = RefCell::new(Weak::new());
    let children = RefCell::new(vec![]);
    Node::from((name, parent, children)).move_self_to_rc()
    // Same as: `(name, parent, children).into().move_to_rc();`
  }

  fn move_self_to_rc(self: Self) -> Rc<Self> {
    Rc::new(self)
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
        // WARNING: the following does not work! Need a `Rc` to `self`!
        // *child.parent.borrow_mut() = Rc::downgrade(&self);
        // WARNING: the following does not work! Need a `Rc` to `self`!
        if let Some(child_parent) = child.get_parent().borrow() {
          // *child_parent.borrow_mut() = Rc::downgrade(self);
        }
      }
      Node::File { .. } => {}
    }
  }
}

#[test]
fn test_can_modify_children_of_folder() {
  let root_dir_rc = Node::new_folder("root");
  let user_dir_rc = Node::new_folder("user");

  {
    let file1_rc = Node::new_file("root_dir_file_1");
    let file2_rc = Node::new_file("root_dir_file_2");

    let file3_rc = Node::new_file("user_dir_file1");
    let file4_rc = Node::new_file("user_dir_file2");

    // Put 2 files in root dir.
    root_dir_rc.add_child(&file1_rc);
    root_dir_rc.add_child(&file2_rc);

    // Put 2 files in user dir.
    user_dir_rc.add_child(&file3_rc);
    user_dir_rc.add_child(&file4_rc);

    // Put user dir in root dir.
    root_dir_rc.add_child(&user_dir_rc);
  }

  // Check that root dir has 3 files and user dir has 2 files.
  assert_eq!(root_dir_rc.get_children().unwrap().len(), 3);
  assert_eq!(user_dir_rc.get_children().unwrap().len(), 2);

  // WARNING: the following fails!
  // assert_eq!(user_dir_rc.get_parent().unwrap().get_name(), "root",);
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
