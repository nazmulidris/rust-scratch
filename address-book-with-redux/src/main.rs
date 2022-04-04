/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

// Connect to source files.
mod address_book;
mod tui;

// Imports.
use std::{env::args, process::exit};
use r3bl_rs_utils::utils::{call_if_err, style_error, style_primary, with, ArgsToStrings};
use tui::run_tui_app;

fn main() {
  with(
    run_tui_app(args().filter_and_convert_to_strings()),
    |result| {
      call_if_err(&result, &|err| {
        eprintln!("{}: {}", style_error("Problem encountered"), err);
        exit(1);
      });
      println!("{}", style_primary("Goodbye."));
      exit(0);
    },
  );
}
