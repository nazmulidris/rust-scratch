// Imports.
use std::{
  marker::{Send, Sync},
  sync::{Arc, RwLock},
};
use tokio::task::JoinHandle;

/// https://stackoverflow.com/questions/59442080/rust-pass-a-function-reference-to-threads
/// https://stackoverflow.com/questions/68547268/cannot-borrow-data-in-an-arc-as-mutable
pub type ThreadSafeLambda<A> = Arc<RwLock<dyn FnMut(A) -> () + Sync + Send>>;
//                             ^^^^^^^^^^                      ^^^^^^^^^^^
//                             Safe to pass          `FnMut` has thread safety requirement
//                             around.               declared to the rust compiler.

pub struct FnWrapper<A> {
  fn_mut: ThreadSafeLambda<A>,
}

impl<A> FnWrapper<A> {
  pub fn new(fn_mut: ThreadSafeLambda<A>) -> Self {
    Self { fn_mut }
  }

  pub fn get_fn_mut(&self) -> ThreadSafeLambda<A> {
    self.fn_mut.clone()
  }
}

pub type Future<T> = JoinHandle<T>;
