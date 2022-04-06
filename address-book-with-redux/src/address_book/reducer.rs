/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

use super::{Action, Contact, State};
use r3bl_rs_utils::utils::with;

pub fn address_book_reducer(
  state: &State,
  action: &Action,
) -> State {
  match action {
    Action::AddContact(name, email, phone) => with(
      state.clone(),
      &|mut new_state: State| {
        new_state
          .address_book
          .push(Contact {
            id: new_state.address_book.len(),
            name: name.to_string(),
            email: email.to_string(),
            phone: phone.to_string(),
          });
        new_state
      },
    ),
    Action::RemoveAllContacts => with(
      state.clone(),
      &|mut new_state: State| {
        new_state.address_book.clear();
        new_state
      },
    ),
    Action::ResetState(new_state) => new_state.clone(),
    Action::RemoveContactById(id) => with(
      state.clone(),
      &|mut new_state: State| {
        new_state.address_book.remove(*id);
        new_state
      },
    ),
    Action::Search(search_term) => with(
      state.clone(),
      &|mut new_state: State| {
        match search_term.as_str() {
          "" => new_state.search_term = None,
          _ => new_state.search_term = Some(search_term.to_string()),
        };
        new_state
      },
    ),
  }
}
