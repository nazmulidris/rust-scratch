use std::fmt::Debug;
use std::hash::Hash;
use super::{Store, ReducerFn};

/// More info on method chaining approaches in Rust:
/// <https://randompoison.github.io/posts/returning-self/>
impl<'a, S, A> Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  // Manage reducers.
  pub fn add_reducer_fn(
    &mut self,
    reducer_fn: &'a ReducerFn<S, A>,
  ) -> &mut Self {
    self.reducer_fns.push(reducer_fn);
    self
  }
}
