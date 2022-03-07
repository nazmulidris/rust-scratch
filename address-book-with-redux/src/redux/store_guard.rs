use std::sync::{Arc, RwLock};
use super::Store;

pub type ShareableStore<S, A> = Arc<RwLock<Store<S, A>>>;

pub struct StoreGuard<S, A> {
  store_arc: ShareableStore<S, A>,
}

impl<S, A> StoreGuard<S, A> {
  pub fn get_store_arc(&self) -> ShareableStore<S, A> {
    self.store_arc.clone()
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
