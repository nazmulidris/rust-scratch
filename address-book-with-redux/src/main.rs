// Connect to source files.
mod address_book;

// Imports.
use std::{env::args, error::Error, process::exit};
use r3bl_rs_utils::utils::{
  call_if_err, print_header, style_error, style_primary, with,
  style_dimmed, with_mut, readline_with_prompt,
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

fn render_state(state: &State) {
  println!("{:#?}", state);
}

fn run_repl(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  let mut store = with(Store::new(&address_book_reducer), |mut store| {
    store
      .add_subscriber_fn(&render_state)
      .add_subscriber_fn(&render_state);
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
      "reset" => store.dispatch_action(&Action::ResetState(State::default())),
      "help" => println!(
        "{}: {}",
        style_primary("Available commands"),
        style_dimmed("quit, exit, add, clear, remove, reset, help")
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
