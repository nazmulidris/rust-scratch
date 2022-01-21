use std::cmp::Ordering;

use ansi_term::Colour::Blue;
use rand::{Rng, thread_rng};

use crate::utils;
use crate::utils::{print_header, style_dimmed, style_error, style_primary, style_prompt};

/// Rust book - https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
pub fn run() {
  print_header("guessing_game");
  println!("Guess the number game :)");
  let answer: u32 = gen_rand_num();
  println!("The random number is: {}", answer);

  loop {
    let guess: String = make_a_guess();
    match guess.as_str().cmp("quit") {
      Ordering::Equal => { break; }
      _ => { match_guess(&answer, &guess); }
    }
  }
}

/// String not &str due to "struct lifetime" - https://stackoverflow.com/a/29026565/2085356
fn make_a_guess() -> String {
  println!("{}", Blue.paint("Please input your guess."));
  let (bytes_read, guess) = utils::readline();
  println!("{} {}, {} {}",
    style_dimmed("#bytes read:"),
    style_primary(&bytes_read.to_string()),
    style_dimmed("You guessed:"),
    style_primary(&guess)
  );
  guess
}

fn match_guess(answer: &u32, guess: &String) {
  // https://learning-rust.github.io/docs/e4.unwrap_and_expect.html
  match guess.parse::<u32>() { // https://techblog.tonsser.com/posts/what-is-rusts-turbofish
    Ok(value) => { perform_match(answer, &value) }
    Err(_) => { println!("{}", style_error("Invalid input, must be a number, try again.")) }
  }
}

fn perform_match(answer: &u32, value: &u32) {
  let resp: &str = match value.cmp(answer) {
    Ordering::Less => "too small",
    Ordering::Equal => "You win",
    Ordering::Greater => "Too big",
  };
  println!("Your guess is {}", style_prompt(resp))
}

fn gen_rand_num() -> u32 {
  thread_rng().gen_range(1..11)
}
