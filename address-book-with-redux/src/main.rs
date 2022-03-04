// Connect to source files.
mod address_book;

// Imports.
use std::{env::args, error::Error, process::exit};
use r3bl_rs_utils::utils::{
  call_if_err, print_header, print_prompt, readline, style_error, style_primary, with,
  style_dimmed, with_mut,
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
  let mut store = Store::new(&address_book_reducer);
  store.add_subscriber_fn(&render_state);

  print_header("Starting repl");

  loop {
    print_prompt("r3bl> ")?;
    let (_, user_input) = readline();

    match user_input.as_ref() {
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
      _ => {
        println!("{}", style_error("Unknown command"));
      }
    }
    // TODO: RemoveContactById
    // TODO: ResetState

    println!(
      "{} {}",
      style_primary(&user_input),
      style_dimmed("was executed.")
    );
  }

  Ok(())
}
