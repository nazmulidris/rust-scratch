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

use crate::{
  address_book::{Contact, State},
  tui::{MAX_DELAY, MIN_DELAY, DELAY_ENABLED},
};
use r3bl_rs_utils::{
  print_header, style_dimmed, tree_memory_arena::HasId, utils::print_prompt,
};
use rand::Rng;

pub fn render_fn(state: State) {
  // https://rust-lang.github.io/rfcs/2909-destructuring-assignment.html
  let State {
    search_term,
    address_book,
  } = state;

  if DELAY_ENABLED {
  // Artificial delay before rendering.
  let delay_ms = rand::thread_rng().gen_range(MIN_DELAY..MAX_DELAY) as u64;
    std::thread::sleep(tokio::time::Duration::from_millis(
      delay_ms,
    ));
  }

  // Actually perform render.
  println!("");
  print_header("render");
  for contact in address_book.iter() {
    if search_term.is_none() || contact_matches_search_term(contact, &search_term) {
      println!(
        "{} {} {} {}",
        style_dimmed(&contact.get_id().to_string()),
        contact.name,
        contact.email,
        contact.phone
      );
    }
  }
  print_prompt("r3bl> ").unwrap();

  // Helper functions.
  fn contact_matches_search_term(
    contact: &Contact,
    search_term: &Option<String>,
  ) -> bool {
    match search_term {
      Some(search_term) => {
        contact
          .name
          .to_lowercase()
          .contains(&search_term.to_lowercase())
          || contact
            .email
            .to_lowercase()
            .contains(&search_term.to_lowercase())
          || contact
            .phone
            .to_lowercase()
            .contains(&search_term.to_lowercase())
      }
      None => true,
    }
  }
}
