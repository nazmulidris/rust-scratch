use std::fmt::Debug;
use std::hash::Hash;
use super::{Store, MiddlewareFn};

/// More info on method chaining approaches in Rust:
/// <https://randompoison.github.io/posts/returning-self/>
impl<'a, S, A> Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  // Manage middleware.
  pub fn add_middleware_fn(
    &mut self,
    middleware_fn: &'a MiddlewareFn<A>,
  ) -> &mut Self {
    self.middleware_fns.push(middleware_fn);
    self
  }

  pub fn middleware_runner(
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
