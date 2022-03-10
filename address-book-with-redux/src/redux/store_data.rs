use super::{
  async_middleware::SafeMiddlewareFnWrapper, async_subscribers::SafeSubscriberFnWrapper,
  sync_reducers::ReducerFnWrapper,
};

/// Redux store. Do not use this directly, please use [`StoreWrapper`] instead.
pub struct StoreData<S, A> {
  pub state: S,
  pub history: Vec<S>,
  pub reducer_fns: Vec<ReducerFnWrapper<S, A>>,
  pub subscriber_fns: Vec<SafeSubscriberFnWrapper<S>>,
  pub middleware_fns: Vec<SafeMiddlewareFnWrapper<A>>,
}

/// Default impl of Redux store.
impl<S, A> Default for StoreData<S, A>
where
  S: Default,
{
  fn default() -> StoreData<S, A> {
    StoreData {
      state: Default::default(),
      history: vec![],
      reducer_fns: vec![],
      subscriber_fns: vec![],
      middleware_fns: vec![],
    }
  }
}
