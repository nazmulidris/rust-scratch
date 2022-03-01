//! The CLI app will take the following arguments:
//!
//! search <search-string> <path-to-file>
//!    ↑          ↑              ↑
//!  arg 0      arg 1           arg 1
//!
//! grapheme
//!    ↑    
//!  arg 0  

use std::env::args;

use r3bl_rs_utils::utils::{style_primary, with};
use seshat::unicode::Segmentation;
use seshat::unicode::Ucd;

fn main() {
  let args = args().collect::<Vec<String>>();
  with(format!("{:?}", args), |it| {
    println!("{}", style_primary(&it));
  });
  print_graphemes();
  print_cluster_breaks();
}

fn print_graphemes() {
  println!("🦀 is {}!", '🦀'.na());
  println!("📦 is {}!", '📦'.na());
  println!("🦜 is {}!", '🦜'.na());
  println!("Multiple code points: 🙏🏽");
  println!("Multiple code points: 💇🏽‍♂️");
}

fn print_cluster_breaks() {
  let s = "Hi + 📦 + 🙏🏽 + 👨🏾‍🤝‍👨🏿";
  let breaks = s.break_graphemes();
  for (size, str) in breaks.enumerate() {
    println!("{}: '{}'", size, str);
  }
}
