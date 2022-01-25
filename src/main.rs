mod guessing_game;
mod hello_world;
mod strings;
mod utils;
mod variables;
mod control_flow;
mod ownership;
mod structs;
mod enum_variants;

fn main() {
  hello_world::run();
  strings::run();
  guessing_game::run();
  variables::run();
  control_flow::run();
  ownership::run();
  structs::run();
  enum_variants::run();
}
