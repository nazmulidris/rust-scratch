use std::{
  error::Error,
  io::{stdin, BufRead},
};

use r3bl_rs_utils::utils::{style_primary, style_prompt};

use crate::piped_grep_command_builder::PipedGrepOptions;

pub fn piped_grep(options: PipedGrepOptions) -> Result<(), Box<dyn Error>> {
  println!(
    "{}: search for '{}' in `stdin` w/ {}",
    style_prompt("DEBUG"),
    options.search,
    match options.case_sensitive {
      true => "case sensitive",
      false => "case insensitive",
    }
  );
  stdin()
    .lock()
    .lines()
    .filter(|line| {
      let line = line.as_ref().unwrap();
      if options.case_sensitive {
        line.contains(&options.search)
      } else {
        line.to_lowercase().contains(&options.search.to_lowercase())
      }
    })
    .map(|line| line.unwrap())
    .for_each(|line| {
      let from = &options.search;
      let to = format!("{}", style_primary(&options.search));
      let line = line.replace(from, &to);
      println!("{}", line);
    });

  Ok(())
}
