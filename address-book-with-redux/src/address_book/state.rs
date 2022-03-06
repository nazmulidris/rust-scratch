use r3bl_rs_utils::tree_memory_arena::HasId;

// State.
#[derive(Clone, Default, PartialEq, Debug, Hash)]
pub struct State {
  pub address_book: Vec<Contact>,
  pub search_term: Option<String>,
}

// Contact.
#[derive(Clone, Default, PartialEq, Debug, Hash)]
pub struct Contact {
  pub id: usize,
  pub name: String,
  pub email: String,
  pub phone: String,
}

impl HasId for Contact {
  type IdType = usize;

  /// Delegate this to `self.id`, which is type `usize`.
  fn get_id(&self) -> usize {
    self.id.get_id()
  }

  /// Delegate this to `self.id`, which is type `usize`.
  fn into_some(&self) -> Option<usize> {
    self.id.into_some()
  }
}
