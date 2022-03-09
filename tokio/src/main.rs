// https://stackoverflow.com/questions/27454761/what-is-a-crate-attribute-and-where-do-i-add-it
#![allow(dead_code, unused_variables)]

// Connect to source files.
mod async_middleware;

// Imports.
use crate::async_middleware::SafeFnWrapper;

#[tokio::main]
async fn main() {
  let logger_mw = logger_mw();
  let adder_mw = adder_mw();

  logger_mw.spawn(Action::Add(1, 2)).await.unwrap();
  logger_mw.spawn(Action::Add(1, 2)).await.unwrap();

  println!("{:?}", adder_mw.spawn(Action::Add(1, 2)).await.unwrap());
  println!("{:?}", adder_mw.spawn(Action::Add(1, 2)).await.unwrap());
}

/// Does not capture context or return anything.
fn logger_mw() -> SafeFnWrapper<Action> {
  let logger_lambda = |action: Action| {
    println!("logging: {:?}", action);
    None
  };
  SafeFnWrapper::new(logger_lambda)
}

/// Captures context and returns a `Future<Action>`.
fn adder_mw() -> SafeFnWrapper<Action> {
  let mut stack: Vec<i32> = Vec::new();
  let adder_lambda = move |action: Action| match action {
    Action::Add(a, b) => {
      let sum = a + b;
      stack.push(a + b);
      Some(Action::Result(sum))
    }
    _ => None,
  };
  SafeFnWrapper::new(adder_lambda)
}

/// Action enum.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Action {
  Add(i32, i32),
  Result(i32),
  Clear,
}
