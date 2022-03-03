use super::Action;
use super::State;

pub fn address_book_reducer(
  state: &State,
  action: &Action,
) -> State {
  match action {
    Action::AddContact(name, email, phone) => {
      let mut new_state = state.clone();
      // new_state.address_book.push(contact.clone());
      new_state
    }
    Action::RemoveAllContacts => {
      let mut new_state = state.clone();
      // new_state.address_book.retain(|c| c.id != contact.id);
      new_state
    }
    Action::RemoveContact(id) => {
      let mut new_state = state.clone();
      // new_state.address_book.retain(|c| c.id != contact.id);
      // new_state.address_book.push(contact.clone());
      new_state
    }
    _ => state.clone(),
  }
}
