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

//! Rust book:
//! 1. <https://doc.rust-lang.org/book/ch10-02-traits.html>
//! 2. <https://doc.rust-lang.org/book/ch19-03-advanced-traits.html>

use chrono::{DateTime, Utc};
use r3bl_rs_utils::utils::style_primary;
use std::{
  fmt::{Display, Formatter, Result},
  ops::Add,
};

pub fn run() {}

#[test]
fn test_traits_comprehensively() {
  // Interface.
  trait Stringable {
    fn to_string(self: &Self) -> String;
    fn length(self: &Self) -> usize;

    fn get_shortened_string(
      item: &String,
      max_length: usize,
    ) -> String
    where
      Self: Sized,
    {
      let mut shortened_string = item.chars().take(max_length).collect::<String>();
      let needs_ellipsis = item.len() > max_length;
      if needs_ellipsis {
        shortened_string.push_str("...");
      }
      shortened_string
    }
  }

  // Structs.
  #[derive(Debug)]
  struct NewsArticle {
    title: String,
    author: String,
    content: String,
    timestamp: DateTime<Utc>,
  }

  #[derive(Debug)]
  struct Tweet {
    content: String,
    username: String,
    timestamp: DateTime<Utc>,
  }

  // Implement the Stringable trait for the structs.
  impl Stringable for NewsArticle {
    fn to_string(self: &Self) -> String {
      format!(
        "ts: {}, author: {}, title: {}, content: {}",
        self.timestamp.format("%Y-%m-%d %H:%M:%S"),
        self.author,
        Self::get_shortened_string(&self.title, 5),
        Self::get_shortened_string(&self.content, 7)
      )
    }

    fn length(self: &Self) -> usize {
      Stringable::to_string(self).len()
    }
  }

  impl Stringable for Tweet {
    fn to_string(self: &Self) -> String {
      format!(
        "ts: {}, content: {}, username: {}",
        self.timestamp.format("%Y-%m-%d %H:%M:%S"),
        Self::get_shortened_string(&self.content, 5),
        Self::get_shortened_string(&self.username, 25)
      )
    }

    fn length(self: &Self) -> usize {
      Stringable::to_string(self).len()
    }
  }

  let article_1 = NewsArticle {
    title: "Rust".to_string(),
    author: "Nazmul".to_string(),
    content: "Rust is the future".to_string(),
    timestamp: Utc::now(),
  };

  let tweet_1 = Tweet {
    content: "Rust-z-futr".to_string(),
    username: "Nazmul".to_string(),
    timestamp: Utc::now(),
  };

  // Both the trait and structs are in scope, so the `to_string` and `length` methods can be used
  // below.

  // Erase the types & access via trait using 2 different techniques.
  let stringable_1_with_type_erasure = &article_1 as &dyn Stringable;
  assert!(dbg!(stringable_1_with_type_erasure.to_string())
    .contains("author: Nazmul, title: Rust, content: Rust is..."));
  assert_eq!(article_1.length(), stringable_1_with_type_erasure.length());

  let stringable_2_with_type_erasure = Box::new(&tweet_1);
  assert!(dbg!(stringable_2_with_type_erasure.to_string())
    .contains("content: Rust-..., username: Nazmul"));
  assert_eq!(tweet_1.length(), stringable_2_with_type_erasure.length());

  // Implement Display trait for structs.
  impl Display for Tweet {
    fn fmt(
      &self,
      f: &mut Formatter<'_>,
    ) -> Result {
      write!(f, "Tweet: {}", Stringable::to_string(self))
    }
  }

  impl Display for NewsArticle {
    fn fmt(
      &self,
      f: &mut Formatter<'_>,
    ) -> Result {
      write!(f, "NewsArticle: {}", Stringable::to_string(self))
    }
  }

  // Functions that takes a reference to a Stringable via 3 different techniques.
  fn use_stringable(item: &dyn Stringable) -> String {
    format!("{}", item.to_string())
  }

  fn use_stringable2(item: &impl Stringable) -> String {
    format!("{}", item.to_string())
  }

  fn use_stringable3<T: Stringable + Display>(item: &T) -> String {
    format!("{}", item.to_string())
  }

  fn use_stringable4<T>(item: &T) -> String
  where
    T: Stringable + Display,
  {
    format!("{}", item.to_string())
  }

  assert_eq!(article_1.length(), use_stringable(&article_1).len());
  assert_eq!(article_1.length(), use_stringable2(&article_1).len());
  assert_eq!(article_1.length(), use_stringable3(&article_1).len());
  assert_eq!(article_1.length(), use_stringable4(&article_1).len());

  assert_eq!(tweet_1.length(), use_stringable(&tweet_1).len());
  assert_eq!(tweet_1.length(), use_stringable2(&tweet_1).len());
  assert_eq!(tweet_1.length(), use_stringable3(&tweet_1).len());
  assert_eq!(tweet_1.length(), use_stringable4(&tweet_1).len());

  // Change the type of the reference to a different type.
  fn borrow_as_stringable_and_display<T: Stringable + Display>(item: &T) -> &T {
    item
  }

  assert_eq!(
    article_1.length(),
    borrow_as_stringable_and_display(&article_1).length()
  );
}

#[test]
fn test_simple_generic_constraints() {
  fn largest_using_slices<T>(list: &[T]) -> &T
  where
    T: PartialOrd,
  {
    let mut largest: &T = &list[0]; // 🚫🧑‍🤝‍🧑 No Copy is made.

    for item in list.iter() {
      if item > largest {
        largest = item;
      }
    }

    &largest
  }

  let number_list = vec![25, 100, 75, 1000];
  let result = largest_using_slices(&number_list);
  assert_eq!(&1000, result);

  let char_list = vec!['y', 'm', 'a', 'q'];
  let result = largest_using_slices(&char_list);
  assert_eq!(&'y', result);

  fn largest_using_copy<T>(list: &[T]) -> T
  where
    T: PartialOrd + Copy,
  {
    let mut largest: T = list[0]; // 🧑‍🤝‍🧑 This is where the Copy occurs.

    for &item in list.iter() {
      if item > largest {
        largest = item;
      }
    }

    largest
  }
  assert_eq!(1000, largest_using_copy(&number_list));
  assert_eq!('y', largest_using_copy(&char_list));
}

#[test]
fn test_trait_bounds_for_conditional_method_impl() {
  // Struct.
  struct Pair<T> {
    first: T,
    second: T,
  }

  // Attach method to Pair struct.
  impl<T> Pair<T> {
    fn new(
      first: T,
      second: T,
    ) -> Self {
      Self { first, second }
    }
  }

  // ⚔️ Attach method to a specific kind of Pair. The following method is only implemented for pairs
  // of a type that implement both PartialOrd + Display.
  impl<T: Display + PartialOrd> Pair<T> {
    fn print_pair_comparison(&self) -> String {
      if self.first >= self.second {
        format!("The largest member is {}", self.first)
      } else {
        format!("The largest member is {}", self.second)
      }
    }
  }

  // i64 is a type that implements both PartialOrd + Display.
  let pair_1 = Pair::<i64>::new(1, 2);
  let result = pair_1.print_pair_comparison();
  assert_eq!(result, "The largest member is 2");

  // &str is a type that implements both PartialOrd + Display.
  let pair_2 = Pair::<&str>::new("1.0", "2.0");
  let result = pair_2.print_pair_comparison();
  assert_eq!(result, "The largest member is 2.0");
}

#[test]
fn test_blanket_implementations_for_trait() {
  // Traits (interfaces).
  trait StringableIF {
    fn to_string(self: &Self) -> String;
  }

  trait ConsoleLoggableIF {
    fn log(self: &Self);
  }

  /// "Blanket implementation" of the trait `ConsoleLoggableIF` for a type `T` that implements the
  /// `StringableIF` trait. This method is available for all types that implement `StringableIF`
  /// trait. Kind of like a Kotlin extension method attached to `T` but with generic constraints.
  impl<T> ConsoleLoggableIF for T
  where
    T: StringableIF,
  {
    fn log(self: &Self) {
      println!("{}", self.to_string());
    }
  }

  // Struct.
  struct Contact {
    name: String,
    email: String,
  }

  // Only implement StringableIF for Contact struct.
  impl StringableIF for Contact {
    fn to_string(&self) -> String {
      format!("{} <{}>", self.name, self.email)
    }
  }

  let contact_1 = Contact {
    name: "John".to_string(),
    email: "j@gmail".to_string(),
  };

  // Becuase `Contact` struct implements `StringableIF`, the "blanket implementation" from
  // `ConsoleLoggableIF` is available.
  contact_1.log();
}

/// https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
#[test]
fn test_associated_types_in_traits_instead_of_generics() {
  // MyString "class".
  #[derive(Debug, Clone, PartialEq)]
  struct MyString {
    contents: String,
  }

  impl MyString {
    fn new(arg: &str) -> Self {
      Self {
        contents: arg.to_string(),
      }
    }
  }

  // Operator overloading for `+` where we will see associated types instead of generics.
  impl Add for MyString {
    type Output = MyString; // Associated type of Add trait determines the type returned by `add()`.

    fn add(
      self,
      other: Self::Output,
    ) -> Self::Output {
      Self::Output {
        contents: self.contents + &other.contents,
      }
    }
  }

  assert_eq!(
    MyString::new("hello") + MyString::new("world"),
    MyString::new("helloworld")
  );
}

#[test]
fn test_more_associated_types_in_traits() {
  // Trait.
  trait Pair {
    // Associated types aka output types.
    type First;
    type Second;

    fn new(
      first: Self::First,
      second: Self::Second,
    ) -> Self;
    fn first(&self) -> &Self::First;
    fn second(&self) -> &Self::Second;
  }

  // Struct that impls trait.
  #[derive(Debug, Clone, PartialEq)]
  struct PairImpl<T, U>
  where
    T: Clone + PartialEq + Display,
    U: Clone + PartialEq + Display,
  {
    first: T,
    second: U,
  }

  impl<T, U> Display for PairImpl<T, U>
  where
    T: Clone + PartialEq + Display,
    U: Clone + PartialEq + Display,
  {
    fn fmt(
      self: &Self,
      f: &mut Formatter<'_>,
    ) -> Result {
      write!(f, "({}, {})", self.first, self.second)
    }
  }

  impl<T, U> Pair for PairImpl<T, U>
  where
    T: Clone + PartialEq + Display,
    U: Clone + PartialEq + Display,
  {
    type First = T;
    type Second = U;

    fn new(
      first: Self::First,
      second: Self::Second,
    ) -> Self {
      Self { first, second }
    }

    fn first(&self) -> &Self::First {
      &self.first
    }

    fn second(&self) -> &Self::Second {
      &self.second
    }
  }

  let pair_1 = PairImpl::<i32, i32>::new(1, 2);
  assert_eq!(pair_1.first(), &1);
  assert_eq!(pair_1.second(), &2);
}

#[test]
fn test_different_styles_of_passing_args_via_trait_impl() {
  trait HasId {
    type Id;
    fn id(&self) -> &Self::Id;
  }

  struct Node {
    id: i32,
    payload: String,
    children: Vec<i32>,
  }

  impl HasId for Node {
    type Id = i32;

    fn id(&self) -> &Self::Id {
      &self.id
    }
  }

  impl HasId for i32 {
    type Id = i32;

    fn id(&self) -> &Self::Id {
      self
    }
  }

  /// This accepts a borrowed `Node` object.
  fn fun_0(node: &Node) {
    println!("{}: {}", style_primary("fun_0:"), node.id());
  }

  /// This accepts a borrowed object that implements `HasId`.
  fn fun_1(node: &dyn HasId<Id = i32>) {
    println!("{}: {}", style_primary("fun_1:"), node.id());
  }

  /// This takes an object that implements `HasId`.
  fn fun_2_own(node: impl HasId<Id = i32>) {
    println!("{}: {}", style_primary("fun_2:"), node.id());
  }

  fn fun_2(node: &impl HasId<Id = i32>) {
    println!("{}: {}", style_primary("fun_2:"), node.id());
  }

  /// This takes an `i32` which also implements `HasId`.
  fn fun_3_own(node: i32) {
    println!("{}: {}", style_primary("fun_3:"), &node.id());
  }

  fn fun_3(node: &i32) {
    println!("{}: {}", style_primary("fun_3:"), &node.id());
  }

  /// This takes a `Node` object that's in a `Box` reference.
  fn fun_4(node: Box<dyn HasId<Id = i32>>) {
    println!("{}: {}", style_primary("fun_4:"), node.id());
  }

  // Here is what it looks like to use these various forms.
  let my_node = Node {
    id: 1,
    payload: "payload".to_string(),
    children: vec![2, 3, 4],
  };
  let my_i32_id = 1;

  fun_0(&my_node);

  fun_1(&my_node);
  fun_1(&my_i32_id);

  fun_2(&my_node);
  fun_2(&my_i32_id);

  fun_3(&my_i32_id);

  fun_4(Box::new(my_node)); // `my_node` is moved into `fun_4`.
  fun_4(Box::new(my_i32_id)); // `my_i32_id` is moved into `fun_4`.
}
