/*
 *   Copyright (c) 2023 Nazmul Idris
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

//! The main entry point (function) for this Markdown parsing module is [parser#parse_markdown]. It
//! takes a string slice and returns a vector of [Block]s.

pub mod parser;
pub mod translator;
pub mod types;

pub use parser::*;
pub use translator::*;
pub use types::*;

pub(crate) mod parser_impl_block;
pub(crate) mod parser_impl_element;

pub(crate) use parser_impl_block::*;
pub(crate) use parser_impl_element::*;
