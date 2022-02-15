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

//! Code: <https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=9444cbeadcfdbef32c664ae2946e636a>
//! SO answer: <https://stackoverflow.com/a/65179837/2085356>

pub fn run() {}

use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub enum Colour {
  Red,
  Black,
}

type Child<T> = Option<Rc<RefCell<Node<T>>>>;
type Parent<T> = Option<Weak<RefCell<Node<T>>>>;

pub struct RBTree<T: Ord> {
  root: Child<T>,
}

impl<T: Ord> RBTree<T> {
  pub fn new() -> Self {
    Self { root: None }
  }

  pub fn insert(&mut self, value: T) {
    fn insert<T: Ord>(child: &mut Child<T>, mut new_node: Node<T>) {
      let child = child.as_ref().unwrap();
      let mut child_mut_borrow = child.borrow_mut();

      if child_mut_borrow.value == new_node.value {
        return;
      }

      let leaf = if child_mut_borrow.value > new_node.value {
        &mut child_mut_borrow.left
      } else {
        &mut child_mut_borrow.right
      };

      match leaf {
        Some(_) => {
          insert(leaf, new_node);
        }
        None => {
          new_node.parent = Some(Rc::downgrade(&child));
          *leaf = Some(Rc::new(RefCell::new(new_node)));
        }
      };
    }

    let mut new_node = Node::new(value);

    if self.root.is_none() {
      new_node.parent = None;
      self.root = Some(Rc::new(RefCell::new(new_node)));
    } else {
      // We ensure that a `None` is never sent to insert()
      insert(&mut self.root, new_node);
    }
  }
}

struct Node<T: Ord> {
  value: T,
  colour: Colour,
  parent: Parent<T>,
  left: Child<T>,
  right: Child<T>,
}

impl<T: Ord> Node<T> {
  fn new(value: T) -> Node<T> {
    Node {
      value: value,
      colour: Colour::Red,
      parent: None,
      left: None,
      right: None,
    }
  }
}

fn main() {
  let mut rbtree = RBTree::<i32>::new();
  rbtree.insert(55);
  rbtree.insert(5);
  rbtree.insert(27);
  rbtree.insert(100);
  rbtree.insert(23);

  assert_eq!(format!("{}", rbtree), "5 23 27 55 100 ");
}

//
// BELOW IS JUST STUFF FOR PRINTING THE TREE, IGNORE
//

use std::fmt::{Display, Formatter};

impl<T: Ord + Display> Display for RBTree<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    RBTree::<T>::traverse(&self.root, f)?;
    Ok(())
  }
}

impl<T: Ord + Display> RBTree<T> {
  fn traverse(root: &Child<T>, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    match root {
      Some(ref node) => {
        let node_borrow = node.borrow();
        RBTree::<T>::traverse(&node_borrow.left, f)?;
        write!(f, "{} ", node_borrow.value)?;
        RBTree::<T>::traverse(&node_borrow.right, f)?;
        Ok(())
      }
      None => Ok(()),
    }
  }
}
