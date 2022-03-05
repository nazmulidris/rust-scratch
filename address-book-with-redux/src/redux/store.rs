use std::{fmt::Debug};
use std::hash::Hash;
use r3bl_rs_utils::utils::{style_dimmed};

pub type ReducerFn<S, A> = dyn Fn(&S, &A) -> S;
pub type SubscriberFn<S> = dyn Fn(&S);
pub type MiddlewareFn<A> = dyn Fn(&A) -> Option<A>;
// Equivalent to:
// pub type ReducerFn<S, A> = fn(&S, &A) -> S;
// pub type SubscriberFn<S> = fn(&S);
// pub type MiddlewareFn<A> = fn(&A) -> A;

pub struct Store<'a, S, A>
where
  S: Default,
{
  pub state: S,
  pub reducer_fns: Vec<&'a ReducerFn<S, A>>,
  pub subscriber_fns: Vec<&'a SubscriberFn<S>>,
  pub middleware_fns: Vec<&'a MiddlewareFn<A>>,
}

impl<'a, S, A> Default for Store<'a, S, A>
where
  S: Default,
{
  fn default() -> Self {
    Self {
      state: Default::default(),
      reducer_fns: vec![],
      subscriber_fns: vec![],
      middleware_fns: vec![],
    }
  }
}

/// More info on method chaining approaches in Rust:
/// <https://randompoison.github.io/posts/returning-self/>
impl<'a, S, A> Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  // Manage subscribers.
  pub fn add_subscriber_fn(
    &mut self,
    new_subscriber_fn: &'a SubscriberFn<S>,
  ) -> &mut Self {
    match self.subscriber_exists(new_subscriber_fn) {
      (true, _) => println!("{}", style_dimmed("Subscriber already exists")),
      (false, _) => self.subscriber_fns.push(new_subscriber_fn),
    }
    self
  }

  pub fn remove_subscriber_fn(
    &mut self,
    subscriber_fn_to_remove: &'a SubscriberFn<S>,
  ) -> &mut Self {
    match self.subscriber_exists(subscriber_fn_to_remove) {
      (true, index) => {
        self.subscriber_fns.remove(index.unwrap());
      }
      _ => {}
    }
    self
  }

  pub fn remove_all_subscribers(&mut self) -> &mut Self {
    self.subscriber_fns.clear();
    self
  }

  /// https://doc.rust-lang.org/std/primitive.pointer.html
  /// https://users.rust-lang.org/t/compare-function-pointers-for-equality/52339
  /// https://www.reddit.com/r/rust/comments/98xlh3/how_can_i_compare_two_function_pointers_to_see_if/
  fn subscriber_exists(
    &self,
    new_subscriber: &'a SubscriberFn<S>,
  ) -> (bool, Option<usize>) {
    let this = new_subscriber as *const SubscriberFn<S>;
    let mut index_if_found = 0 as usize;
    if self
      .subscriber_fns
      .iter()
      .enumerate()
      .any(|(index, other)| {
        index_if_found = index;
        this == *other as *const SubscriberFn<S>
      })
    {
      return (true, Some(index_if_found));
    }
    return (false, None);
  }

  // Manage reducers.
  pub fn add_reducer_fn(
    &mut self,
    reducer_fn: &'a ReducerFn<S, A>,
  ) -> &mut Self {
    self.reducer_fns.push(reducer_fn);
    self
  }

  // Manage middleware.
  pub fn add_middleware_fn(
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
