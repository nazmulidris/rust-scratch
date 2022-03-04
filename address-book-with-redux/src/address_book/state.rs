use super::Contact;

// State.
#[derive(Clone, Default, PartialEq, Debug, Hash)]
pub struct State {
  pub address_book: Vec<Contact>,
}
