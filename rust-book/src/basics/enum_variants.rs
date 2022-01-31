use rust_example_lib::utils::type_utils::type_utils::type_of;

/*
 * Copyright (c) 2022 Nazmul Idris. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


/// Enums and variants are like TypeScript discriminated unions and Kotlin sealed classes. They're
/// **not** like Java enums.
/// Rust book: https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
/// Rust book: https://doc.rust-lang.org/book/ch06-02-match.html
/// Rust book: https://doc.rust-lang.org/book/ch06-03-if-let.html
pub fn run() {}

#[test]
fn test_simple_enum() {
  // Enum w/ 2 variants.
  enum IpAddress {
    Version4(u8, u8, u8, u8),
    Version6(String),
  }
  impl IpAddress {
    fn get_route(self: &Self) -> RouteResult {
      match self {
        IpAddress::Version4(a1, a2, a3, a4) => {
          RouteResult {
            kind: "v4".to_string(),
            address: format!("{}.{}.{}.{}", a1, a2, a3, a4),
          }
        }
        IpAddress::Version6(address) => {
          RouteResult {
            kind: "v6".to_string(),
            address: address.clone(),
          }
        }
      }
    }
  }
  struct RouteResult {
    kind: String,
    address: String,
  }

  let four = IpAddress::Version4(127, 0, 0, 1);
  let six = IpAddress::Version6(String::from("::1"));

  assert_eq!(four.get_route().kind, "v4");
  assert_eq!(four.get_route().address, "127.0.0.1");
  assert_eq!(six.get_route().kind, "v6");
  assert_eq!(six.get_route().address, "::1");
}

#[test]
fn test_enum_representing_redux_action() {
  enum CounterAction {
    Increment,
    Decrement,
    IncrementBy(u32),
    DecrementBy(u32),
  }
  impl CounterAction {
    fn get_type(self: &Self) -> String {
      match self {
        CounterAction::Increment => "INCREMENT".to_string(),
        CounterAction::Decrement => "DECREMENT".to_string(),
        CounterAction::IncrementBy(_) => "INCREMENT_BY".to_string(),
        CounterAction::DecrementBy(_) => "DECREMENT_BY".to_string(),
      }
    }
  }

  let increment_action = CounterAction::Increment;
  let decrement_action = CounterAction::Decrement;
  let increment_by_action = CounterAction::IncrementBy(5);
  let decrement_by_action = CounterAction::DecrementBy(10);

  assert!(type_of(&increment_action).contains("CounterAction"));
  assert!(type_of(&decrement_action).contains("CounterAction"));
  assert!(type_of(&decrement_by_action).contains("CounterAction"));
  assert!(type_of(&increment_by_action).contains("CounterAction"));

  assert_eq!(increment_action.get_type(), "INCREMENT");
  assert_eq!(decrement_action.get_type(), "DECREMENT");
  assert_eq!(increment_by_action.get_type(), "INCREMENT_BY");
  assert_eq!(decrement_by_action.get_type(), "DECREMENT_BY");

  let is_increment_action = match increment_action {
    CounterAction::Increment => { true }
    _ => { false }
  };
  assert!(is_increment_action);

  let is_decrement_action = match decrement_action {
    CounterAction::Decrement => { true }
    _ => { false }
  };
  assert!(is_decrement_action);

  let count = match increment_by_action {
    CounterAction::IncrementBy(count) => { count }
    _ => { 0 }
  };
  assert_eq!(count, 5);

  let count = match decrement_by_action {
    CounterAction::DecrementBy(count) => { count }
    _ => { 0 }
  };
  assert_eq!(count, 10);
}

#[test]
fn test_enum_option_some_none() {
  let some = Some(5);
  assert_eq!(some.unwrap(), 5);

  let none = None;
  assert_eq!(none.unwrap_or_else(|| false), false);

  let empty_option = Option::None;
  assert_eq!(empty_option.unwrap_or_else(|| false), false);

  let number_option: Option<i8> = Some(5);
  assert_eq!(number_option.unwrap(), 5);
}

/// https://doc.rust-lang.org/book/ch06-02-match.html
#[test]
fn test_enum_and_pattern_match() {
  enum PriceKind { Regular, Special }
  enum Porsche { GT3, GT3RS, GT4, GT4RS, CarreraGT(PriceKind) }
  impl Porsche {
    fn get_base_price(self: &Self) -> u32 {
      match self {
        Porsche::GT3 => { 160_000 } // Curly braces are optional. No comma needed.
        Porsche::GT3RS => { 200_000 }
        Porsche::GT4 => 120_000, // Simply returns value w/out curly braces. Needs comma.
        Porsche::GT4RS => 160_000,
        Porsche::CarreraGT(price) => {
          match price {
            PriceKind::Regular => { 500_000 }
            PriceKind::Special => { 1_000_000 }
          }
        }
      }
    }
  }

  assert_eq!(Porsche::GT3.get_base_price(), 160_000);
  assert_eq!(Porsche::GT3RS.get_base_price(), 200_000);
  assert_eq!(Porsche::GT4.get_base_price(), 120_000);
  assert_eq!(Porsche::GT4RS.get_base_price(), 160_000);
  assert_eq!(Porsche::CarreraGT(PriceKind::Special).get_base_price(), 1_000_000);
  assert_eq!(Porsche::CarreraGT(PriceKind::Regular).get_base_price(), 500_000);
}

#[test]
fn test_enum_and_pattern_match_with_option() {
  fn increment(value_holder: Option<i32>) -> Option<i32> {
    match value_holder {
      Some(value) => Some(value + 1),
      _ => None,
    }
  }

  let five_holder: Option<i32> = Some(5);
  let six_holder: Option<i32> = increment(five_holder);
  let none_holder: Option<i32> = increment(None);

  assert!(five_holder.is_some());
  assert!(six_holder.is_some());
  assert!(none_holder.is_none());

  assert_eq!(six_holder.unwrap(), 6);
  assert_eq!(five_holder.unwrap(), 5);
  assert_eq!(none_holder.unwrap_or_else(|| -1), -1);
}

#[test]
fn test_enum_flow_control_using_if_let() {
  let config_max_holder: Option<u8> = Some(3u8);
  assert_eq!(config_max_holder.unwrap(), 3);
  if let Some(value) = config_max_holder {
    assert_eq!(value, 3);
  }
}
