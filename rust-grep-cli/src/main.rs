// Connect to Rust source files.
mod grep;
mod grep_command_builder;
mod piped_grep;
mod piped_grep_command_builder;

// Imports.
use grep::grep;
use grep_command_builder::GrepOptionsBuilder;
use piped_grep::piped_grep;
use piped_grep_command_builder::PipedGrepOptionsBuilder;
use r3bl_rs_utils::utils::{is_stdin_piped, style_error, with};
use std::env::args;
use std::error::Error;
use std::process::exit;

/// This program has 2 modes of operation.
///
/// # Mode 1 - Not using `stdin` (input is not piped into this program).
///
/// <search-string> <path-to-file> <case-sensitive>
///       ↑               ↑              ↑          
///     arg 0           arg 0          arg 0   
///
/// # Mode 2 - Using `stdin` (input is piped into this program).
///
/// Content is piped into the program via `stdin` and it only needs to take the search string and
/// the case sensitive matching as arguments.
///
/// More info on `Box<dyn Error>` or `&'static dyn Error`:
/// - `'static` is the lifetime of `Box<dyn Error>`.
/// - <https://users.rust-lang.org/t/what-does-it-mean-to-return-dyn-error-static/37619/7>
/// - <https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes>
fn main() {
  let args = args().collect::<Vec<String>>();
  with(run(args), |it| match it {
    Ok(()) => exit(0),
    Err(err) => {
      eprintln!("{}: {}", style_error("Problem encountered"), err);
      exit(1);
    }
  });
}

fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
  match is_stdin_piped() {
    true => piped_grep(PipedGrepOptionsBuilder::parse(args)?)?,
    false => grep(GrepOptionsBuilder::parse(args)?)?,
  }
  Ok(())
}
