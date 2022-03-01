//! The CLI app will take the following arguments:
//!
//! search <search-string> <path-to-file>
//!    ↑          ↑              ↑
//!  arg 0      arg 1           arg 1
//!
//! grapheme
//!    ↑    
//!  arg 0  

// Connect to Rust source files.
mod command_builder;
mod graphemes;

// Imports.
use graphemes::{print_cluster_breaks, print_graphemes};
use r3bl_rs_utils::utils::{style_primary, with};
use std::env::args;
use std::error::Error;

// More info on `Box<dyn Error>` or `&'static dyn Error`:
// - `'static` is the lifetime of `Box<dyn Error>`.
// - https://users.rust-lang.org/t/what-does-it-mean-to-return-dyn-error-static/37619/7
// - https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes
fn main() -> Result<(), Box<dyn Error>> {
  let args = args().collect::<Vec<String>>();
  with(format!("{:?}", args), |it| {
    println!("{}", style_primary(&it));
  });
  print_graphemes();
  print_cluster_breaks();
  Ok(())
}
