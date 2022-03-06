use super::State;

// Action.
#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Action {
  AddContact(String, String, String),
  RemoveAllContacts,
  RemoveContactById(usize),
  ResetState(State),
  Search(String),
}