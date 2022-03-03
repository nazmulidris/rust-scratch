use super::Contact;

// State.
#[derive(Clone, Default, PartialEq, Debug, Hash)]
pub struct State {
  address_book: Vec<Contact>,
}
