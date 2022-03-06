// Connect to source files.
mod address_book;
mod tui;

// Imports.
use std::{env::args, process::exit};
use r3bl_rs_utils::{
  utils::{call_if_err, style_error, style_primary, with},
};
use tui::start_repl_loop;

fn main() {
  let args = args().collect::<Vec<String>>();
  with(start_repl_loop(args), |result| {
    call_if_err(&result, &|err| {
      eprintln!("{}: {}", style_error("Problem encountered"), err);
      exit(1);
    });
    println!("{}", style_primary("Goodbye."));
    exit(0);
  });
}
