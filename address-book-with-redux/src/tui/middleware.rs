use crate::address_book::Action;

pub fn logger_mw(action: Action) -> Option<Action> {
  println!("logging: {:?}", action);
  None
}
