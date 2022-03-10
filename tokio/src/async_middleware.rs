// Imports.
use std::{
  marker::{Send, Sync},
  sync::{Arc, RwLock},
};
use tokio::task::JoinHandle;

/// Excellent resources on lifetimes, closures, and returning references:
/// 1. https://stackoverflow.com/questions/59442080/rust-pass-a-function-reference-to-threads
/// 2. https://stackoverflow.com/questions/68547268/cannot-borrow-data-in-an-arc-as-mutable
/// 3. https://willmurphyscode.net/2018/04/25/fixing-a-simple-lifetime-error-in-rust/
/// 4. https://medium.com/@alistairisrael/demystifying-closures-futures-and-async-await-in-rust-part-3-async-await-9ed20eede7a4
pub type SafeFn<A> = Arc<RwLock<dyn FnMut(A) -> Option<A> + Sync + Send>>;
//                   ^^^^^^^^^^                             ^^^^^^^^^^^
//                   Safe to pass      Declare`FnMut` has thread safety
//                   around.           requirement to rust compiler.

pub struct SafeFnWrapper<A> {
  fn_mut: SafeFn<A>,
}

impl<A: Sync + Send + 'static> SafeFnWrapper<A> {
  pub fn new(
    fn_mut: impl FnMut(A) -> Option<A> + Send + Sync + 'static
  ) -> SafeFnWrapper<A> {
    SafeFnWrapper::set(Arc::new(RwLock::new(fn_mut)))
  }

  pub fn set(fn_mut: SafeFn<A>) -> Self {
    Self { fn_mut }
  }

  /// Get a clone of the `fn_mut` field (which holds a thread safe `FnMut`).
  pub fn get(&self) -> SafeFn<A> {
    self.fn_mut.clone()
  }

  /// This is an async function. Make sure to use `await` on the return value.
  pub fn spawn(
    &self,
    action: A,
  ) -> Future<Option<A>> {
    let arc_lock_fn_mut = self.get();
    tokio::spawn(async move {
      let mut fn_mut = arc_lock_fn_mut.write().unwrap();
      fn_mut(action)
    })
  }
}

pub type Future<T> = JoinHandle<T>;