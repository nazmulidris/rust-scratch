// Imports.
use std::error::Error;
use address_book_with_redux_lib::redux::async_middleware::SafeMiddlewareFnWrapper;
use address_book_with_redux_lib::redux::sync_reducers::ReducerFnWrapper;
use rand::random;
use r3bl_rs_utils::utils::{
  print_header, style_error, style_primary, with, style_dimmed, readline_with_prompt,
  unwrap_arc_read_lock_and_call,
};
use address_book_with_redux_lib::redux::{Store};
use address_book_with_redux_lib::redux::async_subscribers::SafeSubscriberFnWrapper;
use crate::address_book::{address_book_reducer, Action, State};
use super::{render_fn, logger_mw};

#[tokio::main]
pub async fn run_tui_app(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  repl_loop(create_store()).await?;
  Ok(())
}

fn create_store() -> Store<State, Action> {
  let mut store = Store::<State, Action>::new();
  store
    .add_subscriber(SafeSubscriberFnWrapper::new(render_fn))
    .add_middleware(SafeMiddlewareFnWrapper::new(logger_mw))
    .add_reducer(ReducerFnWrapper::new(address_book_reducer));
  store
}

pub async fn repl_loop(store: Store<State, Action>) -> Result<(), Box<dyn Error>> {
  // Helper lambda.
  let get_history = || {
    with(store.get(), |store_arc| {
      unwrap_arc_read_lock_and_call(&store_arc, &mut |store| store.history.clone())
    })
  };

  print_header("Starting repl");

  // Repl.
  loop {
    let user_input = readline_with_prompt("r3bl> ")?;
    match user_input.as_str() {
      "help" => println!(
        "{}: {}",
        style_primary("Available commands"),
        style_dimmed("quit, exit, add, clear, remove, reset, search, history, help")
      ),
      "quit" => break,
      "exit" => break,
      "add" => {
        let id = random::<u8>();
        store
          .dispatch(&Action::AddContact(
            format!("John Doe #{}", id),
            format!("jd@gmail.com #{}", id),
            format!("123-456-7890 #{}", id),
          ))
          .await
      }
      "clear" => store.dispatch(&Action::RemoveAllContacts).await,
      "remove" => match readline_with_prompt("id> ") {
        Ok(id) => {
          store
            .dispatch(&Action::RemoveContactById(id.parse().unwrap()))
            .await
        }
        Err(_) => println!("{}", style_error("Invalid id")),
      },
      "search" => match readline_with_prompt("search_term> ") {
        Ok(search_term) => store.dispatch(&Action::Search(search_term)).await,
        Err(_) => println!("{}", style_error("Invalid id")),
      },
      "reset" => store.dispatch(&Action::ResetState(State::default())).await,
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
