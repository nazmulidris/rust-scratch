use std::{
  fmt::Debug,
  hash::Hash,
  sync::{Arc, RwLock},
};
use super::{StoreData, async_subscribers::SafeSubscriberFnWrapper, async_middleware::SafeMiddlewareFnWrapper, sync_reducers::ReducerFnWrapper};

/// Thread safe and async Redux store (using [`tokio`]). This is built atop [`Store`] (which should
/// not be used directly).
pub struct Store<S, A> {
  store_arc: ShareableStoreData<S, A>,
}

pub type ShareableStoreData<S, A> = Arc<RwLock<StoreData<S, A>>>;

impl<S, A> Store<S, A>
where
  S: Default + Clone + PartialEq + Debug + Hash + Sync + Send + 'static,
  A: Clone + Sync + Send + 'static,
{
  pub fn new() -> Store<S, A> {
    Store {
      store_arc: Arc::new(RwLock::new(Default::default())),
    }
  }

  pub fn get(&self) -> ShareableStoreData<S, A> {
    self.store_arc.clone()
  }

  pub async fn dispatch(
    &self,
    action: &A,
  ) {
    self.get().write().unwrap().dispatch_action(&action).await;
  }

  pub fn add_subscriber(
    &self,
    subscriber_fn: SafeSubscriberFnWrapper<S>,
  ) -> &Store<S, A> {
    self
      .get()
      .write()
      .unwrap()
      .subscriber_fns
      .push(subscriber_fn);
    self
  }

  pub fn add_middleware(
    &self,
    middleware_fn: SafeMiddlewareFnWrapper<A>,
  ) -> &Store<S, A> {
    self
      .get()
      .write()
      .unwrap()
      .middleware_fns
      .push(middleware_fn);
    self
  }

  pub fn add_reducer(
    &self,
    reducer_fn: ReducerFnWrapper<S, A>,
  ) -> &Store<S, A> {
    self
      .get()
      .write()
      .unwrap()
      .reducer_fns
      .push(reducer_fn);
    self
  }
}
