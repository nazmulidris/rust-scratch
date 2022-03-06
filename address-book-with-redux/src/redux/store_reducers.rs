use std::fmt::Debug;
use std::hash::Hash;
use super::{Store, ReducerFn, ReducerManager};

/// More info on method chaining approaches in Rust:
/// <https://randompoison.github.io/posts/returning-self/>
impl<'a, S, A> ReducerManager<'a, S, A> for Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  // Manage reducers.
  fn add_reducer_fn(
    &mut self,
    reducer_fn: &'a ReducerFn<S, A>,
  ) -> &mut Self {
    self.reducer_fns.push(reducer_fn);
    self
  }
}
