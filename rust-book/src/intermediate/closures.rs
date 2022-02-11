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

//! - Rust book: <https://doc.rust-lang.org/book/ch13-00-functional-features.html>
//! - SO: <https://stackoverflow.com/a/29191208/2085356>

use std::{
  collections::HashMap,
  hash::Hash,
  ops::Add,
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst},
    Arc,
  },
};

pub fn run() {}

#[test]
fn test_simple_closure() {
  // Don't specify the type of:
  // 1. variable holding the closure.
  // 2. arguments and return type of the closure itself.
  let add_fn_1 = |value| value + 1;
  assert_eq!(add_fn_1(1), 2);

  // Can't specify the type of the variable holding the closure!
  // But can specify it for closures that don't capture an environment using a function pointer.
  // For closures that capture an environment, use the traits `Fn`, `FnMut`, and `FnOnce`.
  let add_fn_2: fn(i32) -> i32 = |value| value + 1;
  assert_eq!(add_fn_2(1), 2);

  // Can specify just the type of the arguments and return type of the closure itself.
  let add_fn_3 = |value: i32| -> i32 { value + 1 };
  assert_eq!(add_fn_3(1), 2);

  // Can do both (when closure doesn't capture envirnoment).
  let add_fn_4: fn(i32) -> i32 = |value: i32| -> i32 { value + 1 };
  assert_eq!(add_fn_4(1), 2);
}

#[test]
fn test_closure_with_shape() {
  // 1. Fn that accepts a closure (that receives the `arg`) and calls it.
  // Can't specify the type of the variable holding the closure! But can restrict its shape via
  // `Fn` trait: https://stackoverflow.com/a/29191208/2085356
  fn call_closure<F, T>(receiver_fn: F, arg: T) -> T
  where
    F: Fn(T) -> T,
  {
    receiver_fn(arg)
  }
  assert_eq!(call_closure(|it| it + 1, 1), 2);

  // 2. Fn that returns a sophisticated closure bound by an enum.
  #[derive(Debug, Clone, PartialEq)]
  enum Number {
    U64(u64),
    I64(i64),
  }
  impl Number {
    fn unwrap(&self) -> i64 {
      match self {
        Number::I64(it) => *it,
        Number::U64(it) => *it as i64,
      }
    }
    fn from(arg: i64) -> Self {
      if arg >= 0 {
        Number::U64(arg as u64)
      } else {
        Number::I64(arg)
      }
    }
  }
  impl Add for Number {
    type Output = Number;

    fn add(self, other: Self::Output) -> Self::Output {
      Number::from(self.unwrap() + other.unwrap())
    }
  }
  fn make_add_closure() -> impl Fn(Number) -> Number {
    |it| -> Number { it + Number::U64(1) }
  }
  assert_eq!(make_add_closure()(Number::from(-2)).unwrap(), -1);

  // 3. Combine both.
  assert_eq!(
    call_closure(make_add_closure(), Number::from(1)).unwrap(),
    2
  );
}

#[test]
fn test_memoize_closure() {
  // Struct for UseMemo.
  #[derive(Debug)]
  struct LazyMemoValues<F, T, V>
  where
    F: FnMut(&T) -> V,
    T: Clone + Eq + Hash,
    V: Copy,
  {
    create_value_fn: F,
    value_map: HashMap<T, Option<V>>,
  }

  // Methods for UseMemo.
  impl<F, T, V> LazyMemoValues<F, T, V>
  where
    F: FnMut(&T) -> V,
    T: Clone + Eq + Hash,
    V: Copy,
  {
    fn new(create_value_fn: F) -> Self {
      LazyMemoValues {
        create_value_fn,
        value_map: HashMap::new(),
      }
    }
    fn get_as_ref(&mut self, arg: &T) -> &V {
      if !self.value_map.contains_key(arg) {
        let value = (self.create_value_fn)(arg);
        self.value_map.insert(arg.clone(), Some(value));
      }
      self.value_map.get(arg).unwrap().as_ref().unwrap()
    }
    fn get_copy(&mut self, arg: &T) -> V {
      self.get_as_ref(arg).clone()
    }
  }

  // https://doc.rust-lang.org/rust-by-example/std/arc.html
  let arc_flag = Arc::new(AtomicUsize::new(0));

  let mut a_variable = 123; // This will be captured in the closure below.
  let mut a_flag = false; // This will be captured in the closure below.

  let mut memo_1 = LazyMemoValues::new(|it| {
    arc_flag.fetch_add(1, SeqCst);
    a_variable = 12;
    a_flag = true;
    a_variable + it
  });

  assert_eq!(arc_flag.load(SeqCst), 0);
  assert_eq!(memo_1.get_as_ref(&1), &13);
  assert_eq!(arc_flag.load(SeqCst), 1);
  assert_eq!(memo_1.get_as_ref(&1), &13); // Won't regenerate the value.
  assert_eq!(arc_flag.load(SeqCst), 1); // Doesn't change.

  assert_eq!(memo_1.get_as_ref(&2), &14);
  assert_eq!(arc_flag.load(SeqCst), 2);
  assert_eq!(memo_1.get_as_ref(&2), &14);
  assert_eq!(memo_1.get_copy(&2), 14);

  assert_eq!(a_variable, 12);
  assert_eq!(a_flag, true);
}
