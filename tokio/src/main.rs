// https://stackoverflow.com/questions/27454761/what-is-a-crate-attribute-and-where-do-i-add-it
#![allow(dead_code, unused_variables)]

// Imports.
use std::sync::{Arc, RwLock};

use tokio::task::JoinHandle;

/// Action enum.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Action {
  Add(i32, i32),
  Multiply(i32, i32),
  Interpret(String),
  Clear,
}

/// https://stackoverflow.com/questions/59442080/rust-pass-a-function-reference-to-threads
/// https://stackoverflow.com/questions/68547268/cannot-borrow-data-in-an-arc-as-mutable
type ThreadSafeLambda = Arc<RwLock<dyn FnMut(Action) -> () + Sync + Send>>;
//                      ^^^^^^^^^^                           ^^^^^^^^^^^
//                      Safe to pass               `FnMut` has thread safety requirement
//                      around.                    declared to the rust compiler.

struct FnWrapper {
  fn_mut: ThreadSafeLambda,
}

impl FnWrapper {
  fn new(fn_mut: ThreadSafeLambda) -> Self {
    Self { fn_mut }
  }

  fn get_fn_mut(&self) -> ThreadSafeLambda {
    self.fn_mut.clone()
  }
}

#[tokio::main]
async fn main() {
  // `logger` fn wrapper.
  let logger_lambda = |action: Action| {
    println!("logging: {:?}", action);
  };
  let logger_ts_lambda: ThreadSafeLambda = Arc::new(RwLock::new(logger_lambda));
  let logger_wrapper = FnWrapper::new(logger_ts_lambda);

  // `adder` fn wrapper that captures context.
  let mut stack: Vec<i32> = Vec::new();
  let adder_lambda = move |action: Action| {
    if let Action::Add(a, b) = action {
      stack.push(a + b);
    }
  };
  let adder_ts_lambda: ThreadSafeLambda = Arc::new(RwLock::new(adder_lambda));
  let adder_wrapper = FnWrapper::new(adder_ts_lambda);

  // Run them both in using `tokio::spawn`.
  run_async(logger_wrapper.get_fn_mut(), Action::Add(1, 2))
    .await
    .unwrap();
}

type Future<T> = JoinHandle<T>;

fn run_async(
  lambda: ThreadSafeLambda,
  action: Action,
) -> Future<()> {
  tokio::spawn(async move {
    let mut fn_mut = lambda.write().unwrap();
    fn_mut(action)
  })
}
