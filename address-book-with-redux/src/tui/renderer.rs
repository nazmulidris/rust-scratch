use r3bl_rs_utils::{utils::style_dimmed, tree_memory_arena::HasId};
use crate::address_book::{State, Contact};

pub fn render_fn(state: State) {
  // https://rust-lang.github.io/rfcs/2909-destructuring-assignment.html
  let State {
    search_term,
    address_book,
  } = state;

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
