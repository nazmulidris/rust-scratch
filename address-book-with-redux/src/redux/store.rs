pub type ReducerFn<S, A> = dyn Fn(&S, &A) -> S;
pub type SubscriberFn<S> = dyn Fn(&S);
pub type MiddlewareFn<A> = dyn Fn(&A) -> Option<A>;
// Equivalent to:
// pub type ReducerFn<S, A> = fn(&S, &A) -> S;
// pub type SubscriberFn<S> = fn(&S);
// pub type MiddlewareFn<A> = fn(&A) -> A;

pub struct Store<'a, S, A> {
  pub state: S,
  pub history: Vec<S>,
  pub reducer_fns: Vec<&'a ReducerFn<S, A>>,
  pub subscriber_fns: Vec<&'a SubscriberFn<S>>,
  pub middleware_fns: Vec<&'a MiddlewareFn<A>>,
}

// Default impl.
impl<'a, S, A> Default for Store<'a, S, A>
where
  S: Default,
{
  fn default() -> Self {
    Self {
      state: Default::default(),
      history: vec![],
      reducer_fns: vec![],
      subscriber_fns: vec![],
      middleware_fns: vec![],
    }
  }
}
