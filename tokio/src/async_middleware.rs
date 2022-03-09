// Imports.
use std::{
  marker::{Send, Sync},
  sync::{Arc, RwLock},
};
use tokio::task::JoinHandle;

/// https://stackoverflow.com/questions/59442080/rust-pass-a-function-reference-to-threads
/// https://stackoverflow.com/questions/68547268/cannot-borrow-data-in-an-arc-as-mutable
pub type SafeFn<A> = Arc<RwLock<dyn FnMut(A) -> Option<A> + Sync + Send>>;
//                   ^^^^^^^^^^                             ^^^^^^^^^^^
//                   Safe to pass      Declare`FnMut` has thread safety
//                   around.           requirement to rust compiler.

pub struct SafeFnWrapper<A> {
  fn_mut: SafeFn<A>,
}

impl<A> SafeFnWrapper<A> {
  pub fn wrap(fn_mut: SafeFn<A>) -> Self {
    Self { fn_mut }
  }

  pub fn unwrap(&self) -> SafeFn<A> {
    self.fn_mut.clone()
  }
}

pub type Future<T> = JoinHandle<T>;
