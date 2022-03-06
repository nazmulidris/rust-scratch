use r3bl_rs_utils::utils::style_error;
use crate::address_book::Action;

pub fn logger_middleware_fn(action: &Action) -> Option<Action> {
  println!("{}: {:?}", style_error("logger_mw"), action);
  None
}
