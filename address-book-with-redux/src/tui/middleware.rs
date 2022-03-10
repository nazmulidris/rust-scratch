use address_book_with_redux_lib::redux::async_middleware::SafeFnWrapper;
use crate::address_book::Action;

pub fn logger_mw() -> SafeFnWrapper<Action> {
  let logger_lambda = |action: Action| {
    println!("logging: {:?}", action);
    None
  };
  SafeFnWrapper::new(logger_lambda)
}
