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

//! Background information on terminals, ANSI, ASCII, etc.
//!
//! - crossterm docs: https://docs.rs/crossterm/latest/crossterm/index.html
//! - Tutorial: https://medium.com/@otukof/build-your-text-editor-with-rust-part-2-74e03daef237
//! - Raw mode: https://en.wikipedia.org/wiki/POSIX_terminal_interface#Non-canonical_mode_processing
//! - Canonical mode: https://en.wikipedia.org/wiki/POSIX_terminal_interface#Canonical_mode_processing
//! - ANSI escape codes: https://en.wikipedia.org/wiki/ANSI_escape_code
//!   - Windows support: https://en.wikipedia.org/wiki/ANSI_escape_code#DOS,_OS/2,_and_Windows
//!   - Colors: https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
//! - ASCII control chars: https://www.asciitable.com/
//! - ANSI (8-bit) vs ASCII (7-bit): http://www.differencebetween.net/technology/web-applications/difference-between-ansi-and-ascii/
//! - Windows Terminal (bash): https://www.makeuseof.com/windows-terminal-vs-powershell/

use crossterm::terminal;
use r3bl_rs_utils::CommonResult;
use tokio::io::{stdin, AsyncReadExt, Stdin};

pub async fn emit_crossterm_commands() -> CommonResult<()> {
  println!("TODO: crossterm: Hello, world!");
  // repl_canonical_mode().await?;
  repl_raw_mode().await?;
  Ok(())
}

macro_rules! println_raw {
  ($arg:tt) => {
    println!("{}\r", $arg)
  };
}

/// To use this, you need to make sure to create an instance using `default()` (which
/// enables raw mode) and then when this instance is dropped (when it falls out of scope)
/// raw mode will be disabled.
struct RawMode;
impl RawMode {
  fn start() -> Self {
    println_raw!("start raw mode");
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    RawMode
  }
}
impl Drop for RawMode {
  fn drop(&mut self) {
    terminal::disable_raw_mode().expect("Failed to disable raw mode");
    println_raw!("end raw mode");
  }
}

pub async fn repl_raw_mode() -> CommonResult<()> {
  let _raw_mode = RawMode::start();
  repl().await?;
  return Ok(());

  enum StdinState {
    NoInput,
    InputControlChar(u8),
    InputNormalChar(char),
  }

  async fn read_stdin_to_state() -> CommonResult<StdinState> {
    let mut read_buffer = [0; 1];
    let bytes_read_into_read_buffer = stdin().read(&mut read_buffer).await?;

    if bytes_read_into_read_buffer == 0 {
      return Ok(StdinState::NoInput);
    }

    let character = read_buffer[0] as char;
    let is_control = character.is_control();

    if is_control {
      let number = character as u8;
      Ok(StdinState::InputControlChar(number))
    } else {
      Ok(StdinState::InputNormalChar(character))
    }
  }

  async fn repl() -> CommonResult<()> {
    println_raw!("Type x to exit repl.");
    loop {
      let state = read_stdin_to_state().await?;
      match state {
        StdinState::NoInput => break,
        StdinState::InputNormalChar('x') => break,
        StdinState::InputControlChar(number) => {
          let msg = format!("CONTROL {}", number);
          println_raw!(msg);
        }
        StdinState::InputNormalChar(character) => {
          println_raw!(character);
        }
      }
    }
    Ok(())
  }
}

pub async fn repl_canonical_mode() -> CommonResult<()> {
  println!("REPL: canonical mode");
  println!("- To quit: [Ctrl+D (EOL)] or [x + 👇]");
  println!("- To print: Type, then press 👇");

  let mut stdin = stdin();
  loop {
    match read_stdin_to_state(&mut stdin).await? {
      StdinState::EOL => {
        println!("REPL: EOL");
        break;
      }
      StdinState::QUIT => {
        println!("REPL: QUIT");
        break;
      }
      StdinState::INPUT(char_read) => {
        println!("REPL: INPUT: {}", char_read);
      }
    }
  }

  return Ok(());

  enum StdinState {
    EOL,
    QUIT,
    INPUT(char),
  }

  async fn read_stdin_to_state(stdin: &mut Stdin) -> CommonResult<StdinState> {
    let mut read_buffer = [0u8; 1];
    let bytes_read_into_read_buffer = stdin.read(&mut read_buffer).await?;
    if bytes_read_into_read_buffer == 0 {
      return Ok(StdinState::EOL);
    }
    let char_read = read_buffer[0] as char;
    return match char_read {
      'x' => Ok(StdinState::QUIT),
      _ => Ok(StdinState::INPUT(char_read)),
    };
  }
}
