pub type ReducerFn<S, A> = dyn Fn(&S, &A) -> S;
pub type SubscriberFn<S> = dyn Fn(&S);
pub type MiddlewareFn<A> = dyn Fn(&A) -> Option<A>;
// Equivalent to:
// pub type ReducerFn<S, A> = fn(&S, &A) -> S;
// pub type SubscriberFn<S> = fn(&S);
// pub type MiddlewareFn<A> = fn(&A) -> Option<A>;

pub struct Store<S, A> {
  pub state: S,
  pub history: Vec<S>,
  pub reducer_fns: Vec<Box<ReducerFn<S, A>>>,
  pub subscriber_fns: Vec<Box<SubscriberFn<S>>>,
  pub middleware_fns: Vec<Box<MiddlewareFn<A>>>,
}

// Default impl.
impl<S, A> Default for Store<S, A>
where
  S: Default,
{
  fn default() -> Store<S, A> {
    Store {
      state: Default::default(),
      history: vec![],
      reducer_fns: vec![],
      subscriber_fns: vec![],
      middleware_fns: vec![],
    }
  }
}
