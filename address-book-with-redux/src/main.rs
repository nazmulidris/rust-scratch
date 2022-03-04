// Connect to source files.
mod address_book;

// Imports.
use std::{env::args, error::Error, process::exit};
use r3bl_rs_utils::utils::{
  call_if_err, print_header, print_prompt, readline, style_error, style_primary, with,
};
use address_book_with_redux_lib::redux::{Store, SubscriberFn};
use address_book::{address_book_reducer, Action, State};

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

fn run_repl(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  let mut count = 0 as usize;
  let mut store = Store::new(&address_book_reducer);

  let subscriber_fn: &SubscriberFn<State> = &|state| {
    println!("{:?}", state);
  };
  store.add_subscriber_fn(&subscriber_fn);
  store.add_subscriber_fn(&subscriber_fn);

  print_header("Starting repl");

  loop {
    print_prompt("r3bl> ")?;
    let (_, user_input) = readline();
    match user_input.as_ref() {
      "quit" => break,
      "exit" => break,
      "add" => {
        count = count + 1;
        store.dispatch_action(&Action::AddContact(
          format!("John Doe #{}", count),
          format!("jd@gmail.com #{}", count),
          format!("123-456-7890 #{}", count),
        ));
      }
      // TODO: add more strings for actions here
      _ => {
        println!("{}", style_error("Unknown command"));
      }
    }

    if (user_input == "exit") || (user_input == "quit") {
      break;
    }
    println!("{}", style_primary(&user_input));
  }

  Ok(())
}
