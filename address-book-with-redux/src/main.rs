// Connect to source files.
mod address_book;

// Imports.
use std::{env::args, error::Error, process::exit};
use rand::random;
use r3bl_rs_utils::{
  utils::{
    call_if_err, print_header, style_error, style_primary, with, style_dimmed, with_mut,
    readline_with_prompt,
  },
  tree_memory_arena::HasId,
};
use address_book::{address_book_reducer, Action, State, Contact};
use address_book_with_redux_lib::redux::{
  Store, StoreInterface, ReducerManager, SubscriberManager, MiddlewareManager,
  DispatchManager,
};

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
  // https://rust-lang.github.io/rfcs/2909-destructuring-assignment.html
  let State {
    search_term,
    address_book,
  } = state;

  for contact in address_book.iter() {
    if search_term.is_none() || contact_matches_search_term(contact, search_term) {
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

fn logger_middleware_fn(action: &Action) -> Option<Action> {
  println!("{}: {:?}", style_error("logger_mw"), action);
  None
}

fn run_repl(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
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
