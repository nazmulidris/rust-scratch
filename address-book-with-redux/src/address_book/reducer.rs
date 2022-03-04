use r3bl_rs_utils::utils::with;

use super::Action;
use super::Contact;
use super::State;

pub fn address_book_reducer(
  state: &State,
  action: &Action,
) -> State {
  match action {
    Action::AddContact(name, email, phone) => {
      with(state.clone(), &|mut new_state: State| {
        new_state.address_book.push(Contact {
          id: new_state.address_book.len(),
          name: name.to_string(),
          email: email.to_string(),
          phone: phone.to_string(),
        });
        new_state
      })
    }
    Action::RemoveAllContacts => with(state.clone(), &|mut new_state: State| {
      new_state.address_book.clear();
      new_state
    }),
    Action::ResetState(new_state) => new_state.clone(),
    Action::RemoveContactById(id) => with(state.clone(), &|mut new_state: State| {
      new_state.address_book.remove(*id);
      new_state
    }),
  }
}
