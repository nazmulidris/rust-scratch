use core::{hash::Hash, fmt::Debug};
use super::StoreData;

// Handle dispatch & history.
impl<S, A> StoreData<S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash + Sync + Send + 'static,
  A: Clone + Sync + Send + 'static,
{
  pub async fn dispatch_action(
    &mut self,
    action: &A,
  ) where
    A: Clone + Send + Sync + 'static,
  {
    // Run middleware & collect resulting actions.
    let mut resulting_actions = self.middleware_runner(action).await;

    // Add the original action to the resulting actions.
    resulting_actions.push(action.clone());

    // Dispatch the resulting actions.
    for action in resulting_actions.iter() {
      self.actually_dispatch_action(action).await;
    }
  }

  async fn actually_dispatch_action(
    &mut self,
    action: &A,
  ) {
    // Run reducers.
    self.reducer_manager.iter().for_each(|reducer_fn| {
      let reducer_fn = reducer_fn.get();
      let new_state = reducer_fn(&self.state, &action);
      update_history(&mut self.history, &new_state);
      self.state = new_state;
    });

    // Run subscribers.
    for subscriber_fn in self.subscriber_manager.iter() {
      subscriber_fn.spawn(self.state.clone()).await.unwrap();
    }

    // Update history.
    fn update_history<S>(
      history: &mut Vec<S>,
      new_state: &S,
    ) where
      S: PartialEq + Clone,
    {
      // Update history.
      let mut update_history = false;
      if history.is_empty() {
        update_history = true;
      } else if let Some(last_known_state) = history.last() {
        if *last_known_state != *new_state {
          update_history = true;
        }
      }
      if update_history {
        history.push(new_state.clone())
      };
    }
  }

  /// Run middleware and return a list of resulting actions. If a middleware produces `None` that
  /// isn't added to the list that's returned.
  async fn middleware_runner(
    &mut self,
    action: &A,
  ) -> Vec<A> {
    let mut results: Vec<A> = vec![];
    for middleware_fn in self.middleware_manager.iter() {
      let result = middleware_fn.spawn(action.clone()).await;
      if let Ok(option) = result {
        if let Some(action) = option {
          results.push(action);
        }
      }
    }
    results
  }
}
