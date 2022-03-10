use core::{hash::Hash, fmt::Debug};
use r3bl_rs_utils::utils::style_dimmed;
use super::{Store, ReducerFn, SubscriberFn, async_middleware::SafeFnWrapper};

// Handle dispatch & history.
impl<S, A> Store<S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
   pub async  fn dispatch_action(
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
      let new_state = reducer_fn(&self.state, &action);
      update_history(&mut self.history, &new_state);
      self.state = new_state;
    });

    // Run subscribers.
    self.subscriber_fns.iter_mut().for_each(|subscriber_fn| {
      (subscriber_fn)(&self.state);
    });

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
}

// Manage middleware.
impl<S, A> Store<S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
  A: Sync + Send + Clone + 'static,
{
  pub fn add_middleware_fn(
    &mut self,
    middleware_fn: SafeFnWrapper<A>,
  ) -> &mut Store<S, A> {
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
}

// Manage reducers.
impl<S, A> Store<S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  pub fn add_reducer_fn(
    &mut self,
    reducer_fn: Box<ReducerFn<S, A>>,
  ) -> &mut Store<S, A> {
    self.reducer_fns.push(reducer_fn);
    self
  }
}

// Manage subscribers.
impl<S, A> Store<S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  pub fn add_subscriber_fn(
    &mut self,
    new_subscriber_fn: Box<SubscriberFn<S>>,
  ) -> &mut Store<S, A> {
    match self.subscriber_exists(new_subscriber_fn.as_ref()) {
      (true, _) => println!("{}", style_dimmed("Subscriber already exists")),
      (false, _) => self.subscriber_fns.push(new_subscriber_fn),
    }
    self
  }

  pub fn remove_subscriber_fn(
    &mut self,
    subscriber_fn_to_remove: Box<SubscriberFn<S>>,
  ) -> &mut Store<S, A> {
    if let (true, index) = self.subscriber_exists(subscriber_fn_to_remove.as_ref()) {
      let _ = self.subscriber_fns.remove(index.unwrap());
    }
    self
  }

  pub fn remove_all_subscribers(&mut self) -> &mut Store<S, A> {
    self.subscriber_fns.clear();
    self
  }

  /// https://doc.rust-lang.org/std/primitive.pointer.html
  /// https://users.rust-lang.org/t/compare-function-pointers-for-equality/52339
  /// https://www.reddit.com/r/rust/comments/98xlh3/how_can_i_compare_two_function_pointers_to_see_if/
  fn subscriber_exists(
    &self,
    new_subscriber: &SubscriberFn<S>,
  ) -> (bool, Option<usize>) {
    let mut index_if_found = 0 as usize;
    if self
      .subscriber_fns
      .iter()
      .enumerate()
      .any(|(index, other)| {
        index_if_found = index;
        new_subscriber as *const SubscriberFn<S>
          == other.as_ref() as *const SubscriberFn<S>
      })
    {
      return (true, Some(index_if_found));
    }
    return (false, None);
  }
}
