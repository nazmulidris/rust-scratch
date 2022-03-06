use core::hash::Hash;
use core::fmt::Debug;
use super::Store;

/// More info on method chaining approaches in Rust:
/// <https://randompoison.github.io/posts/returning-self/>
impl<'a, S, A> Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  // Dispatch.
  pub fn dispatch_action(
    &mut self,
    action: &A,
  ) where
    A: Clone,
  {
    // Run middleware & collect resulting actions.
    let mut resulting_actions = self
      .middleware_runner(action)
      .iter()
      .filter(|action| action.is_some())
      .map(|action| action.clone().unwrap())
      .collect::<Vec<A>>();

    // Add the original action to the resulting actions.
    resulting_actions.push(action.clone());

    // Dispatch the resulting actions.
    resulting_actions
      .iter()
      .for_each(|action| self.actually_dispatch_action(action));
  }

  fn actually_dispatch_action(
    &mut self,
    action: &A,
  ) {
    // Run reducers.
    self.reducer_fns.iter().for_each(|reducer_fn| {
      self.state = reducer_fn(&self.state, &action);
    });

    // Run subscribers.
    self.subscriber_fns.iter_mut().for_each(|subscriber_fn| {
      (subscriber_fn)(&self.state);
    });
  }
}
