// https://stackoverflow.com/questions/27454761/what-is-a-crate-attribute-and-where-do-i-add-it
#![allow(dead_code, unused_variables)]

// Connect to source files.
mod async_middleware;
use async_middleware::ThreadSafeLambda;

// Imports.
use crate::async_middleware::{FnWrapper, Future};
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
  let logger_mw = logger_mw();
  let adder_mw = adder_mw();

  spawn(logger_mw.get_fn_mut(), Action::Add(1, 2))
    .await
    .unwrap();

  spawn(adder_mw.get_fn_mut(), Action::Add(1, 2))
    .await
    .unwrap();
}

/// Does not capture context or return anything.
fn logger_mw() -> FnWrapper<Action> {
  let logger_lambda = |action: Action| {
    println!("logging: {:?}", action);
  };
  let logger_ts_lambda: ThreadSafeLambda<Action> = Arc::new(RwLock::new(logger_lambda));
  let logger_wrapper = FnWrapper::new(logger_ts_lambda);
  logger_wrapper
}

/// Captures context and returns a `Future<Action>`.
fn adder_mw() -> FnWrapper<Action> {
  let mut stack: Vec<i32> = Vec::new();
  let adder_lambda = move |action: Action| {
    if let Action::Add(a, b) = action {
      stack.push(a + b);
    }
  };
  let adder_ts_lambda: ThreadSafeLambda<Action> = Arc::new(RwLock::new(adder_lambda));
  let adder_wrapper = FnWrapper::new(adder_ts_lambda);
  adder_wrapper
}

fn spawn(
  lambda: ThreadSafeLambda<Action>,
  action: Action,
) -> Future<()> {
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
