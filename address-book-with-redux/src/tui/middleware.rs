use rand::Rng;

use crate::{
  address_book::Action,
  tui::{MIN_DELAY, MAX_DELAY},
};

pub fn logger_mw(action: Action) -> Option<Action> {
  // Artificial delay before calling the function.
  let delay_ms = rand::thread_rng().gen_range(MIN_DELAY..MAX_DELAY) as u64;
  std::thread::sleep(tokio::time::Duration::from_millis(delay_ms));

  // Log the action.
  println!("logging: {:?}", action);
  None
}
