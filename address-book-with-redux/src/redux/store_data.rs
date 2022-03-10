use super::{SafeMiddlewareFnWrapper, ListManager, ReducerFnWrapper, SafeSubscriberFnWrapper};

/// Redux store. Do not use this directly, please use [`Store`] instead.
pub struct StoreData<S, A> {
  pub state: S,
  pub history: Vec<S>,
  pub reducer_manager: ListManager<ReducerFnWrapper<S, A>>,
  pub subscriber_manager: ListManager<SafeSubscriberFnWrapper<S>>,
  pub middleware_manager: ListManager<SafeMiddlewareFnWrapper<A>>,
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
      reducer_manager: ListManager::new(),
      subscriber_manager: ListManager::new(),
      middleware_manager: ListManager::new(),
    }
  }
}
