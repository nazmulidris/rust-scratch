/*
 *   Copyright (c) 2024 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

#[cfg(test)]
pub mod fixtures {
    pub use crossterm::style::Stylize as _;

    /// Returns a string representation of the memory address. That is formatted using
    /// bold and white style.
    pub fn format_address<T>(val: *const T) -> String {
        format!("{:?}", val).white().bold().to_string()
    }

    pub fn format_size(size: usize) -> String {
        format!("{} bytes", size).green().bold().to_string()
    }
}
