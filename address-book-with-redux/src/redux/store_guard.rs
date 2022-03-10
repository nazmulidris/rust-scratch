use std::{
  fmt::Debug,
  hash::Hash,
  sync::{Arc, RwLock},
};
use super::Store;

pub type ShareableStore<S, A> = Arc<RwLock<Store<S, A>>>;

pub struct StoreGuard<S, A> {
  store_arc: ShareableStore<S, A>,
}

impl<S, A> StoreGuard<S, A>
where
  S: Default + Clone + PartialEq + Debug + Hash,
  A: Clone + Sync + Send + 'static,
{
  pub fn get_store_arc(&self) -> ShareableStore<S, A> {
    self.store_arc.clone()
  }

  pub async fn dispatch(
    &self,
    action: &A,
  ) {
    self
      .get_store_arc()
      .write()
      .unwrap()
      .dispatch_action(&action)
      .await;
  }
}

impl<S, A> Default for StoreGuard<S, A>
where
  S: Default,
{
  fn default() -> StoreGuard<S, A> {
    StoreGuard {
      store_arc: Arc::new(RwLock::new(Default::default())),
    }
  }
}
