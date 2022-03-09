// https://stackoverflow.com/questions/27454761/what-is-a-crate-attribute-and-where-do-i-add-it
#![allow(dead_code, unused_variables)]

// Connect to source files.
mod async_middleware;
use async_middleware::SafeFn;

// Imports.
use crate::async_middleware::{SafeFnWrapper, Future};
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
  let logger_mw = logger_mw();
  let adder_mw = adder_mw();

  spawn(logger_mw.unwrap(), Action::Add(1, 2))
    .await
    .unwrap();

  let result_action = spawn(adder_mw.unwrap(), Action::Add(1, 2))
    .await
    .unwrap();
  println!("result_action: {:?}", result_action);
}

/// Does not capture context or return anything.
fn logger_mw() -> SafeFnWrapper<Action> {
  let logger_lambda = |action: Action| {
    println!("logging: {:?}", action);
    None
  };
  let logger_ts_lambda: SafeFn<Action> = Arc::new(RwLock::new(logger_lambda));
  let logger_wrapper = SafeFnWrapper::wrap(logger_ts_lambda);
  logger_wrapper
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
  let adder_ts_lambda: SafeFn<Action> = Arc::new(RwLock::new(adder_lambda));
  let adder_wrapper = SafeFnWrapper::wrap(adder_ts_lambda);
  adder_wrapper
}

fn spawn(
  lambda: SafeFn<Action>,
  action: Action,
) -> Future<Option<Action>> {
  tokio::spawn(async move {
    let mut fn_mut = lambda.write().unwrap();
    fn_mut(action)
  })
}

/// Action enum.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Action {
  Add(i32, i32),
  Result(i32),
  Clear,
}
