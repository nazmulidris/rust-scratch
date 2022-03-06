// Imports.
use std::error::Error;
use rand::random;
use r3bl_rs_utils::{
  utils::{
    print_header, style_error, style_primary, with, style_dimmed, with_mut,
    readline_with_prompt,
  },
};
use address_book_with_redux_lib::redux::{
  Store, StoreInterface, ReducerManager, SubscriberManager, MiddlewareManager,
  DispatchManager,
};
use crate::address_book::{address_book_reducer, Action, State};
use super::{render_fn, logger_middleware_fn};

pub fn start_repl_loop(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  let mut store = with(Store::new(), |mut store| {
    store
      .add_reducer_fn(&address_book_reducer)
      .add_subscriber_fn(&render_fn)
      .add_subscriber_fn(&render_fn)
      .add_middleware_fn(&logger_middleware_fn);
    store
  });

  print_header("Starting repl");

  loop {
    let user_input = readline_with_prompt("r3bl> ")?;

    match user_input.as_str() {
      "quit" => break,
      "exit" => break,
      "add" => with_mut(&mut random::<u8>(), &mut |id| {
        store.dispatch_action(&Action::AddContact(
          format!("John Doe #{}", id),
          format!("jd@gmail.com #{}", id),
          format!("123-456-7890 #{}", id),
        ));
      }),
      "clear" => store.dispatch_action(&Action::RemoveAllContacts),
      "remove" => match readline_with_prompt("id> ") {
        Ok(id) => store.dispatch_action(&Action::RemoveContactById(id.parse().unwrap())),
        Err(_) => println!("{}", style_error("Invalid id")),
      },
      "search" => match readline_with_prompt("search_term> ") {
        Ok(search_term) => store.dispatch_action(&Action::Search(search_term)),
        Err(_) => println!("{}", style_error("Invalid id")),
      },
      "reset" => store.dispatch_action(&Action::ResetState(State::default())),
      "help" => println!(
        "{}: {}",
        style_primary("Available commands"),
        style_dimmed("quit, exit, add, clear, remove, reset, search, help")
      ),
      _ => {
        println!("{}", style_error("Unknown command"));
      }
    } // end match user_input.

    // Print confirmation at the end of 1 repl loop.
    println!(
      "{} {}",
      style_primary(&user_input),
      style_dimmed("was executed.")
    );
  } // end loop.

  Ok(())
}
