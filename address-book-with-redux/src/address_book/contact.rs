use r3bl_rs_utils::tree_memory_arena::HasId;

// Contact.
#[derive(Clone, Default, PartialEq, Debug, Hash)]
pub struct Contact {
  id: usize,
  name: String,
  email: String,
  phone: String,
}

impl HasId for Contact {
  type IdType = usize;

  /// Delegate this to `self.id`, which is type `usize`.
  fn get_id(&self) -> usize {
    self.id.get_id()
  }

  /// Delegate this to `self.id`, which is type `usize`.
  fn into_some(&self) -> Option<&(dyn HasId<IdType = usize>)> {
    self.id.into_some()
  }
}
