use crate::utils::print_header;
use rand::{thread_rng, Rng};
use std::cmp::Ordering;
use std::io::stdin;

pub fn run() {
  print_header("guessing_game");

  println!("Guess the number game :)");

  let rand_num: u32 = thread_rng().gen_range(1..11);
  println!("The random number is: {}", rand_num);

  println!("Please input your guess.");
  let mut guess: String = String::new();
  let bytes_read: usize = stdin().read_line(&mut guess).expect("Failed to read line!");

  // Remove any whitespace (including \n).
  let guess: String = guess.trim().to_string();

  println!("#bytes read: {}, You guessed: {}", bytes_read, guess);

  // Turn guess from String -> u32.
  let guess: u32 = guess.parse().expect("Please type a number!");

  let resp: &str = match guess.cmp(&rand_num) {
    Ordering::Less => "too small",
    Ordering::Equal => "You win",
    Ordering::Greater => "Too big",
  };

  println!("Your guess is {}", resp)
}
