/*
 * Copyright (c) 2022 Nazmul Idris. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// Disable warnings for dead code & unused imports since they're needed by tests.
#![allow(dead_code)]
#![allow(unused_imports)]

// Attach the following files to the binary module.
mod basics; // This is a module that contains many other files.
mod data_structures; // This is a module that contains many other files.
mod intermediate; // This is a module that contains many other files.
mod runnable; // This is a module that contains many other files.

fn main() {
  // The following have runnable code.
  runnable::hello_world::run();
  runnable::simple_strings::run();
  runnable::guessing_game::run();

  // The following only have tests.
  basics::variables::run();
  basics::control_flow::run();
  basics::ownership::run();
  basics::structs::run();
  basics::enum_variants::run();

  data_structures::vector::run();
  data_structures::strings::run();
  data_structures::hashmap::run();

  intermediate::error_handling::run();
  intermediate::generic_types::run();
  intermediate::traits::run();
  intermediate::lifetimes::run();
  intermediate::closures::run();
  intermediate::iterators::run();
  intermediate::smart_pointers::run();
}
