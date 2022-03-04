use super::State;

// Action.
#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Action {
  ResetState(State),
  RemoveAllContacts,
  AddContact(String, String, String),
  RemoveContact(usize),
}
