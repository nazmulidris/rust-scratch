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

pub mod clap_args;
pub mod client_task;
pub mod protocol;
pub mod server_task;
pub mod utils;

pub use clap_args::*;
pub use client_task::*;
pub use protocol::*;
pub use server_task::*;
pub use utils::*;

pub const CHANNEL_SIZE: usize = 10;
