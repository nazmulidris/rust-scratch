use std::{env::args, error::Error, process::exit};

use r3bl_rs_utils::utils::{style_error, with};

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
  Ok(())
}
