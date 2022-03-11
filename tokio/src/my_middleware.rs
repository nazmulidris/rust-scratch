use crate::middleware::SafeFnWrapper;

/// Does not capture context or return anything.
pub fn logger_mw() -> SafeFnWrapper<Action> {
  let logger_lambda = |action: Action| {
    println!("logging: {:?}", action);
    None
  };
  SafeFnWrapper::new(logger_lambda)
}

/// Captures context and returns a `Future<Action>`.
pub fn adder_mw() -> SafeFnWrapper<Action> {
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
pub enum Action {
  Add(i32, i32),
  Result(i32),
}
