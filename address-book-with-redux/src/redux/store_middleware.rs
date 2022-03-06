use std::fmt::Debug;
use std::hash::Hash;
use super::{Store, MiddlewareFn, MiddlewareManager};

/// More info on method chaining approaches in Rust:
/// <https://randompoison.github.io/posts/returning-self/>
impl<'a, S, A> MiddlewareManager<'a, A> for Store<'a, S, A>
where
  S: Clone
    + Default
    + PartialEq
    + Debug
    + Hash
{
  // Manage middleware.
  fn add_middleware_fn(
    &mut self,
    middleware_fn: &'a MiddlewareFn<A>,
  ) -> &mut Self {
    self.middleware_fns.push(middleware_fn);
    self
  }

  fn middleware_runner(
    &mut self,
    action: &A,
  ) -> Vec<Option<A>> {
    let resulting_actions = self
      .middleware_fns
      .iter()
      .map(|middleware_fn| middleware_fn(action))
      .collect::<Vec<Option<A>>>();
    resulting_actions
  }
}
