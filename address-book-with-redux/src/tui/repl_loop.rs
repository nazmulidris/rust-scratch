// Imports.
use std::error::Error;
use rand::random;
use r3bl_rs_utils::utils::{
  print_header, style_error, style_primary, with, style_dimmed, with_mut,
  readline_with_prompt, unwrap_arc_write_lock_and_call, unwrap_arc_read_lock_and_call,
};
use address_book_with_redux_lib::redux::StoreGuard;
use crate::address_book::{address_book_reducer, Action, State};
use super::{render_fn, logger_middleware_fn};

pub fn run_tui_app(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  let store_guard: StoreGuard<State, Action> =
    with(StoreGuard::default(), |store_guard| {
      setup_store_guard(&store_guard);
      store_guard
    });
  repl_loop(store_guard)?;
  Ok(())
}

fn setup_store_guard(store_guard: &StoreGuard<State, Action>) {
  with(store_guard.get_store_arc(), |store_arc| {
    unwrap_arc_write_lock_and_call(&store_arc, &mut |store| {
      store
        .add_reducer_fn(Box::new(address_book_reducer))
        .add_subscriber_fn(Box::new(render_fn))
        .add_subscriber_fn(Box::new(render_fn))
        .add_middleware_fn(Box::new(logger_middleware_fn));
    });
  });
}

pub fn repl_loop(store_guard: StoreGuard<State, Action>) -> Result<(), Box<dyn Error>> {
  // Helper lambda.
  let dispatch = |action: Action| {
    with(store_guard.get_store_arc(), |store_arc| {
      unwrap_arc_write_lock_and_call(&store_arc, &mut |store| {
        store.dispatch_action(&action);
      });
    });
  };

  // Helper lambda.
  let get_history = || {
    with(store_guard.get_store_arc(), |store_arc| {
      unwrap_arc_read_lock_and_call(&store_arc, &mut |store| store.history.clone())
    })
  };

  print_header("Starting repl");

  // Repl loop.
  loop {
    let user_input = readline_with_prompt("r3bl> ")?;
    match user_input.as_str() {
      "help" => println!(
        "{}: {}",
        style_primary("Available commands"),
        style_dimmed("quit, exit, add, clear, remove, reset, search, help")
      ),
      "quit" => break,
      "exit" => break,
      "add" => with_mut(&mut random::<u8>(), &mut |id| {
        dispatch(Action::AddContact(
          format!("John Doe #{}", id),
          format!("jd@gmail.com #{}", id),
          format!("123-456-7890 #{}", id),
        ));
      }),
      "clear" => dispatch(Action::RemoveAllContacts),
      "remove" => match readline_with_prompt("id> ") {
        Ok(id) => dispatch(Action::RemoveContactById(id.parse().unwrap())),
        Err(_) => println!("{}", style_error("Invalid id")),
      },
      "search" => match readline_with_prompt("search_term> ") {
        Ok(search_term) => dispatch(Action::Search(search_term)),
        Err(_) => println!("{}", style_error("Invalid id")),
      },
      "reset" => dispatch(Action::ResetState(State::default())),
      "history" => println!("{:#?}", get_history()),
      // Catchall.
      _ => println!("{}", style_error("Unknown command")),
    } // end match user_input.

    // Print confirmation at the end of 1 repl loop.
    println!(
      "{} {}",
      style_primary(&user_input),
      style_dimmed("was executed.")
    );
  }

  Ok(())
}
