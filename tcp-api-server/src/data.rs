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
use serde::{Deserialize, Serialize};

use crate::Buffer;

/// This is the data structure that is used to send messages between the client and the
/// server.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub id: f32,
    pub description: String,
    pub data: Buffer,
}

/// These type aliases are used throughout the codebase to make it easier to specify the
/// exact type of the key used in [crate::ServerMessage] and [crate::ClientMessage].
pub type MessageKey = String;

/// These type aliases are used throughout the codebase to make it easier to specify the
/// exact type of the value used in [crate::ServerMessage] and [crate::ClientMessage].
pub type MessageValue = Data;
