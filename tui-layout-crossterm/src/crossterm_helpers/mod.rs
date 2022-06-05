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
//!   - Raw mode: https://docs.rs/crossterm/0.23.2/crossterm/terminal/index.html#raw-mode
//! - Tutorial: https://medium.com/@otukof/build-your-text-editor-with-rust-part-2-74e03daef237
//! - Raw mode: https://en.wikipedia.org/wiki/POSIX_terminal_interface#Non-canonical_mode_processing
//! - Canonical mode: https://en.wikipedia.org/wiki/POSIX_terminal_interface#Canonical_mode_processing
//! - ANSI escape codes: https://en.wikipedia.org/wiki/ANSI_escape_code
//!   - Windows support: https://en.wikipedia.org/wiki/ANSI_escape_code#DOS,_OS/2,_and_Windows
//!   - Colors: https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
//! - ASCII control chars: https://www.asciitable.com/
//! - ANSI (8-bit) vs ASCII (7-bit): http://www.differencebetween.net/technology/web-applications/difference-between-ansi-and-ascii/
//! - Windows Terminal (bash): https://www.makeuseof.com/windows-terminal-vs-powershell/

// Attach source files.
pub mod raw_mode;
pub mod input_event;

// Re-export everything from attached source files.
pub use raw_mode::*;
pub use input_event::*;