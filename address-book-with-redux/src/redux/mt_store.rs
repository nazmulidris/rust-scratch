use std::sync::{Arc, RwLock};
use super::Store;

pub type ShareableStore<'a, S, A> = Arc<RwLock<Store<'a, S, A>>>;

pub struct MTStore<'a, S, A> {
  store_arc: ShareableStore<'a, S, A>,
}

impl<'a, S, A> MTStore<'a, S, A> {
  pub fn get_store_arc(&'a self) -> ShareableStore<'a, S, A> {
    self.store_arc.clone()
  }
}

impl<'a, S, A> Default for MTStore<'a, S, A>
where
  S: Default,
{
  fn default() -> MTStore<'a, S, A> {
    MTStore {
      store_arc: Arc::new(RwLock::new(Default::default())),
    }
  }
}
