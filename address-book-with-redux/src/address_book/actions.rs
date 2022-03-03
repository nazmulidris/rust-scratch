// Action.
#[derive(Clone, PartialEq, Debug, Hash)]
pub enum Action {
  RemoveAllContacts,
  AddContact(String, String, String),
  RemoveContact(usize),
}
