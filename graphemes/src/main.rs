// Connect to Rust source files.
mod graphemes;

// Imports.
use graphemes::{print_cluster_breaks, print_graphemes};

fn main() {
  print_graphemes();
  print_cluster_breaks();
}
