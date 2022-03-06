// Connect to source files.
mod address_book;

// Imports.
use std::{env::args, error::Error, process::exit};
use r3bl_rs_utils::utils::{
  call_if_err, print_header, style_error, style_primary, with, style_dimmed, with_mut,
  readline_with_prompt,
};
use address_book_with_redux_lib::redux::{Store};
use address_book::{address_book_reducer, Action, State};
use rand::random;

fn main() {
  let args = args().collect::<Vec<String>>();
  with(run_repl(args), |result| {
    call_if_err(&result, &|err| {
      eprintln!("{}: {}", style_error("Problem encountered"), err);
      exit(1);
    });
    println!("{}", style_primary("Goodbye."));
    exit(0);
  });
}

fn render_fn(state: &State) {
  match state.search_term {
    Some(ref search_term) => println!("TODO! Searching for: {}", search_term),
    None => println!(
      "{}: {}",
      style_primary("render\n"),
      style_dimmed(&format!("{:#?}", state))
    ),
  }
}

fn logger_middleware_fn(action: &Action) -> Option<Action> {
  println!(
    "{}: {}",
    style_error("logger_mw"),
    style_dimmed(&format!("{:#?}", action))
  );
  None
}

fn run_repl(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  let mut store = with(Store::default(), |mut store| {
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
