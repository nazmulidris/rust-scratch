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

//! Rust book: <https://doc.rust-lang.org/book/ch15-01-box.html>
//!
//! [Box, Rc, Arc, RefCell](https://tekshinobi.com/rust-tips-box-rc-arc-cell-refcell-mutex):
//! 1. `Box` is for single ownership. A great use case is to use this when we want to store primitive
//!    types (stored on stack) on heap.
//! 2. `Rc` is for multiple ownership.
//! 3. `Arc` is for multiple ownership, but threadsafe.
//! 4. `RefCell` is for “interior mutability”; that is, when you need to mutate something behind a &T.
//! 5. `Cell` is for “interior mutability” for Copy types; that is, when you need to mutate something
//!    behind a `&T`. `Cell`, is similar to `RefCell` except that instead of giving references to the
//!    inner value, the value is copied in and out of the `Cell`.
//! 6. `Mutex`, which offers interior mutability that’s safe to use across threads
use std::{
  array,
  cell::RefCell,
  fmt::Display,
  ops::Deref,
  rc::Rc,
  sync::{Arc, Mutex, RwLock, Weak},
};

pub fn run() {}

#[test]
fn test_use_box() {
  let my_box: Box<i32> = Box::new(5);
  assert_eq!(*my_box, 5);
  assert_eq!(my_box, Box::new(5));

  do_something_with_my_box_borrow(&my_box);

  fn do_something_with_my_box_borrow(my_box: &Box<i32>) {
    // Approach 1.
    assert_eq!(**my_box, 5);

    // Approach 2.
    let foo: &i32 = my_box.deref();
    assert_eq!(*foo, 5);

    // Approach 3.
    assert_eq!(*(my_box.deref()), 5);

    // Approach 4.
    assert_eq!(my_box.deref(), &5);
  }
}

#[test]
fn test_use_deref_trait() {
  // This isn't a real `Box`, the content isn't allocated on the stack (not heap). This is just to
  // see how the `Deref` trait works.
  struct MyBox<T> {
    content: T,
  }

  impl<T> MyBox<T> {
    fn new(content: T) -> MyBox<T> {
      MyBox { content }
    }
  }

  impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(self: &Self) -> &T {
      &(self.content)
    }
  }

  let x = 5;
  let my_box = MyBox::new(x);
  assert_eq!(*(my_box.deref()), 5);
  assert_eq!(*my_box, 5);

  // Implicit `deref` coercion with functions and methods.
  //
  // Here we’re calling the `hello` function with the argument `&m`, which is a reference to a
  // `MyBox<String>` value. Because we implemented the `Deref` trait on `MyBox<T>`, Rust can turn
  // `&MyBox<String>` into `&String` by calling `deref`. The standard library provides an
  // implementation of `Deref` on `String` that returns a string slice, and this is in the API
  // documentation for `Deref`. Rust calls `deref` again to turn the `&String` into `&str`, which
  // matches the `hello` function’s definition.

  let box_containing_string = MyBox::new(String::from("Hello"));
  fn_that_accepts_str_slice(&box_containing_string);

  fn fn_that_accepts_str_slice(str_slice: &str) {
    assert_eq!(format!("{}", str_slice), str_slice);
  }
}

#[test]
fn test_use_drop_trait() {
  struct MyBoxDroppable<T>
  where
    T: Display,
  {
    content: T,
  }

  impl<T> MyBoxDroppable<T>
  where
    T: Display,
  {
    fn new(content: T) -> MyBoxDroppable<T> {
      MyBoxDroppable { content }
    }
  }

  impl<T> Drop for MyBoxDroppable<T>
  where
    T: Display,
  {
    fn drop(self: &mut Self) {
      println!("=> Dropping MyBoxDroppable with content: {}", self.content);
    }
  }

  // Force a drop of `my_box_droppable` at the end of this block.
  {
    let my_box_droppable = MyBoxDroppable::new(5);
    println!("=> Created MyBoxDroppable {}", my_box_droppable.content);
  }

  // Another way to force a drop by calling `drop` explicitly.
  {
    let my_box_droppable = MyBoxDroppable::new(5);
    drop(my_box_droppable);
  }
}

/// `Rc` (aka reference count) owns the data inside of it.
/// 1. Can pass it around without losing ownership.
/// 2. No concurrency, paralellism, or mutation.
#[test]
fn test_use_rc_no_concurrency_or_parallelism_or_mutation() {
  let ref_1 = Rc::new(5);
  assert_eq!(*ref_1, 5);
  {
    let ref_2 = ref_1.clone(); // aka `Rc::clone(&ref_1)`.
    assert_eq!(*ref_2, 5);
    {
      let ref_3 = ref_2.clone(); // aka `Rc::clone(&ref_2)`.
      assert_eq!(*ref_3, 5);
      assert_eq!(Rc::strong_count(&ref_1), 3);
    } // `ref_3` is dropped here.
    assert_eq!(Rc::strong_count(&ref_1), 2);
  } // `ref_2` is dropped here.
  assert_eq!(Rc::strong_count(&ref_1), 1);
}

/// `RefCell` gives interior mutability. Combining with `Rc` means allowing having multiple owners
/// of mutable data.
/// 1. `RefCell` gives the appearance of immutable data to some, while allowing mutating the data.
/// 2. No paralelleism, or concurrecy.
/// <https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#having-multiple-owners-of-mutable-data-by-combining-rct-and-refcellt>
#[test]
fn test_use_rc_with_refcell_for_multiple_owners_of_mutable_data_no_concurrency_or_paralellism() {
  let ref_1 = Rc::new(RefCell::new(5));
  assert_eq!(*ref_1.borrow(), 5);
  {
    let ref_2 = ref_1.clone();
    assert_eq!(*ref_2.borrow(), 5);
    {
      let ref_3 = ref_2.clone();
      assert_eq!(*ref_3.borrow(), 5);
      *ref_3.borrow_mut() = 10;
      assert_eq!(*ref_3.borrow(), 10);
      assert_eq!(*ref_1.borrow(), 10);
    }
    assert_eq!(*ref_2.borrow(), 10);
    assert_eq!(*ref_1.borrow(), 10);
  }
  assert_eq!(*ref_1.borrow(), 10);
}

/// <https://doc.rust-lang.org/book/ch15-06-reference-cycles.html#adding-a-reference-from-a-child-to-its-parent>
/// Thinking about the relationships another way, a parent node should own its children: if a parent
/// node is dropped, its child nodes should be dropped as well. However, a child should not own its
/// parent: if we drop a child node, the parent should still exist. This is a case for weak
/// references!
#[test]
fn test_weak_refs() {
  // TODO: move this into tree.rs *delete tree-fail.rs*
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

/// `Arc` with `Mutex` is the parallel and concurrent version of the `Rc` and `RefCell` test (above
/// somewhere). It allows thread safe interior mutability.
/// 1. <https://aeshirey.github.io/code/2020/12/23/arc-mutex-in-rust.html>
/// 2. <https://fongyoong.github.io/easy_rust/Chapter_43.html>
#[test]
fn test_use_arc_mutex_for_concurrency_or_paralellism() {
  fn wrap_my_data<T>(arg: &[T]) -> Arc<Mutex<Vec<T>>>
  where
    T: Clone + Sized,
  {
    let my_data = arg.to_vec();
    let my_data = Mutex::new(my_data);
    let my_data = Arc::new(my_data);
    my_data
  }

  fn modify_my_data_1<T>(arg: Arc<Mutex<Vec<T>>>, value: T)
  where
    T: Clone + Sized,
  {
    if let Ok(mut my_data) = arg.lock() {
      my_data.push(value);
    }
  }

  fn modify_my_data_2<T>(arg: Arc<Mutex<Vec<T>>>, value: T)
  where
    T: Clone + Sized,
  {
    if let Ok(mut my_data) = arg.lock() {
      my_data.push(value);
    }
  }

  let ref_to_my_data = wrap_my_data(&[1, 2, 3]);
  assert_eq!(ref_to_my_data.lock().unwrap().len(), 3);

  modify_my_data_1(ref_to_my_data.clone(), 20);
  modify_my_data_1(ref_to_my_data.clone(), 30);

  assert_eq!(ref_to_my_data.lock().unwrap().len(), 5);
  assert_eq!(ref_to_my_data.lock().unwrap()[0], 1);
  assert_eq!(*ref_to_my_data.lock().unwrap(), vec![1, 2, 3, 20, 30]);
}

/// `Arc` w/ `RwLock` is even better than using `Arc` w/ `Mutex`. It allows fine grained locking and
/// interior mutability.
/// 1. <https://fongyoong.github.io/easy_rust/Chapter_44.html>
#[test]
fn test_use_arc_rwlock_for_concurrency_or_paralellism() {
  fn wrap_my_data<T>(arg: &[T]) -> Arc<RwLock<Vec<T>>>
  where
    T: Clone + Sized,
  {
    let my_data = arg.to_vec();
    let my_data = RwLock::new(my_data);
    let my_data = Arc::new(my_data);
    my_data
  }

  fn modify_my_data_1<T>(arg: Arc<RwLock<Vec<T>>>, value: T)
  where
    T: Clone + Sized,
  {
    if let Ok(mut my_data) = arg.write() {
      my_data.push(value);
    }
  }

  fn modify_my_data_2<T>(arg: Arc<RwLock<Vec<T>>>, value: T)
  where
    T: Clone + Sized,
  {
    if let Ok(mut my_data) = arg.write() {
      my_data.push(value);
    }
  }

  let ref_to_my_data = wrap_my_data(&[1, 2, 3]);
  assert_eq!(ref_to_my_data.read().unwrap().len(), 3);

  modify_my_data_1(ref_to_my_data.clone(), 20);
  modify_my_data_1(ref_to_my_data.clone(), 30);

  assert_eq!(ref_to_my_data.read().unwrap().len(), 5);
  assert_eq!(ref_to_my_data.read().unwrap()[0], 1);
  assert_eq!(*ref_to_my_data.read().unwrap(), vec![1, 2, 3, 20, 30]);
}
