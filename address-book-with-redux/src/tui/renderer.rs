use r3bl_rs_utils::{utils::style_dimmed, tree_memory_arena::HasId};
use rand::Rng;
use crate::{
  address_book::{State, Contact},
  tui::{MIN_DELAY, MAX_DELAY},
};

pub fn render_fn(state: State) {
  // https://rust-lang.github.io/rfcs/2909-destructuring-assignment.html
  let State {
    search_term,
    address_book,
  } = state;

  // Artificial delay before rendering.
  let delay_ms = rand::thread_rng().gen_range(MIN_DELAY..MAX_DELAY) as u64;
  std::thread::sleep(tokio::time::Duration::from_millis(delay_ms));

  // Actually perform render.
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
