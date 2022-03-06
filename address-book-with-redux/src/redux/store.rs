use std::{clone::Clone, default::Default, cmp::PartialEq, fmt::Debug, hash::Hash};

pub type ReducerFn<S, A> = dyn Fn(&S, &A) -> S;
pub type SubscriberFn<S> = dyn Fn(&S);
pub type MiddlewareFn<A> = dyn Fn(&A) -> Option<A>;
// Equivalent to:
// pub type ReducerFn<S, A> = fn(&S, &A) -> S;
// pub type SubscriberFn<S> = fn(&S);
// pub type MiddlewareFn<A> = fn(&A) -> A;

pub struct Store<'a, S, A> {
  pub state: S,
  pub reducer_fns: Vec<&'a ReducerFn<S, A>>,
  pub subscriber_fns: Vec<&'a SubscriberFn<S>>,
  pub middleware_fns: Vec<&'a MiddlewareFn<A>>,
}

impl<'a, S, A> StoreInterface<'a, S, A> for Store<'a, S, A>
where
  S: Clone + Default + PartialEq + Debug + Hash,
{
  fn new() -> Self {
    Self {
      state: Default::default(),
      reducer_fns: vec![],
      subscriber_fns: vec![],
      middleware_fns: vec![],
    }
  }
}

pub trait StoreInterface<'a, S, A>:
  MiddlewareManager<'a, A>
  + DispatchManager<A>
  + ReducerManager<'a, S, A>
  + SubscriberManager<'a, S, A>
{
  fn new() -> Self;
}

pub trait MiddlewareManager<'a, A> {
  fn add_middleware_fn(
    &mut self,
    middleware_fn: &'a MiddlewareFn<A>,
  ) -> &mut Self;

  fn middleware_runner(
    &mut self,
    action: &A,
  ) -> Vec<Option<A>>;
}

pub trait DispatchManager<A> {
  fn dispatch_action(
    &mut self,
    action: &A,
  ) where
    A: Clone;

  fn actually_dispatch_action(
    &mut self,
    action: &A,
  );
}

pub trait ReducerManager<'a, S, A> {
  fn add_reducer_fn(
    &mut self,
    reducer_fn: &'a ReducerFn<S, A>,
  ) -> &mut Self;
}

pub trait SubscriberManager<'a, S, A> {
  fn add_subscriber_fn(
    &mut self,
    new_subscriber_fn: &'a SubscriberFn<S>,
  ) -> &mut Self;

  fn remove_subscriber_fn(
    &mut self,
    subscriber_fn_to_remove: &'a SubscriberFn<S>,
  ) -> &mut Self;

  fn remove_all_subscribers(&mut self) -> &mut Self;

  fn subscriber_exists(
    &self,
    new_subscriber: &'a SubscriberFn<S>,
  ) -> (bool, Option<usize>);
}
