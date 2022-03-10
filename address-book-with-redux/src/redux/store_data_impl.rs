use crate::redux::{async_subscribers::SafeSubscriberFnWrapper};
use core::{hash::Hash, fmt::Debug};
use std::sync::Arc;
use super::{
  async_middleware::SafeMiddlewareFnWrapper, StoreData, sync_reducers::ReducerFnWrapper,
};

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
    self.reducer_fns.iter().for_each(|reducer_fn| {
      let reducer_fn = reducer_fn.get();
      let new_state = reducer_fn(&self.state, &action);
      update_history(&mut self.history, &new_state);
      self.state = new_state;
    });

    // Run subscribers.
    for subscriber_fn in self.subscriber_fns.iter() {
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

  // Manage middleware.
  pub fn add_middleware_fn(
    &mut self,
    middleware_fn: SafeMiddlewareFnWrapper<A>,
  ) -> &mut StoreData<S, A> {
    self.middleware_fns.push(middleware_fn);
    self
  }

  /// Run middleware and return a list of resulting actions. If a middleware produces `None` that
  /// isn't added to the list that's returned.
  async fn middleware_runner(
    &mut self,
    action: &A,
  ) -> Vec<A> {
    let mut results: Vec<A> = vec![];
    for middleware_fn in self.middleware_fns.iter() {
      let result = middleware_fn.spawn(action.clone()).await;
      if let Ok(option) = result {
        if let Some(action) = option {
          results.push(action);
        }
      }
    }
    results
  }

  // Manage reducers.
  pub fn add_reducer_fn(
    &mut self,
    new_reducer: ReducerFnWrapper<S, A>,
  ) -> &mut StoreData<S, A> {
    self.reducer_fns.push(new_reducer);
    self
  }

  pub fn remove_all_reducers(&mut self) -> &mut StoreData<S, A> {
    self.reducer_fns.clear();
    self
  }

  // Manage subscribers.
  pub fn add_subscriber_fn(
    &mut self,
    new_subscriber: SafeSubscriberFnWrapper<S>,
  ) -> &mut StoreData<S, A> {
    for subscriber_fn in self.subscriber_fns.iter() {
      let is_same = Arc::ptr_eq(&subscriber_fn.get(), &new_subscriber.get());
      if is_same {
        return self;
      }
    }
    self.subscriber_fns.push(new_subscriber);
    self
  }

  pub fn remove_all_subscribers(&mut self) -> &mut StoreData<S, A> {
    self.subscriber_fns.clear();
    self
  }
}
