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

//! This module is standalone, you can use it any project that needs to communicate
//! between a client and a server using a length-prefix, binary payload, protocol. The
//! generics `K` and `V` are used to specify the exact type of the key and value used in
//! the messages by whatever module is using this protocol.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// These are messages that the client can send to the server.
///
/// More info:
/// - <https://docs.rs/strum_macros/latest/strum_macros/derive.EnumString.html>
/// - <https://docs.rs/strum_macros/latest/strum_macros/derive.Display.html>
/// - <https://docs.rs/strum_macros/latest/strum_macros/derive.EnumIter.html>
#[derive(
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    strum_macros::EnumString,
    strum_macros::EnumIter,
    strum_macros::Display,
)]
pub enum ClientMessage<K: Default, V: Default> {
    #[default]
    #[strum(ascii_case_insensitive)]
    GetAll,
    #[strum(ascii_case_insensitive)]
    Exit,
    #[strum(ascii_case_insensitive)]
    Insert(K, V),
    #[strum(ascii_case_insensitive)]
    Remove(K),
    #[strum(ascii_case_insensitive)]
    Get(K),
    #[strum(ascii_case_insensitive)]
    Clear,
    #[strum(ascii_case_insensitive)]
    Size,
    /// Client A initiates this. It gets BroadcastToOthersAck(..). Other clients get HandleBroadcast(..).
    #[strum(ascii_case_insensitive)]
    BroadcastToOthers(V),
}

impl<K: Default, V: Default> ClientMessage<K, V> {
    pub fn try_parse_input(input: &str) -> Result<(Self, String), strum::ParseError> {
        // If input is empty, then return the default command.
        if input.is_empty() {
            return Ok((ClientMessage::default(), "".to_string()));
        }

        // If input has a space, then split it and use the first part as the command.
        let parts: Vec<&str> = input.split_whitespace().collect();
        let client_message = ClientMessage::<K, V>::from_str(parts[0])?; // Same as: input.parse::<ClientMessage<K,V>>()?;
        let rest = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            "".to_string()
        };

        Ok((client_message, rest))
    }
}

/// These are messages that the server can send to the client.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ServerMessage<K, V> {
    SetClientId(String),
    Exit,
    GetAll(Vec<(K, V)>),
    Insert(bool),
    Remove(bool),
    Get(Option<V>),
    Clear(bool),
    Size(usize),
    /// Client A initiates BroadcastToOthers(..). Client A gets this.
    /// The usize is the number of clients, total # clients that received the broadcast.
    /// This does NOT include Client A.
    BroadcastToOthersAck(usize),
    /// Client A initiates BroadcastToOthers(..). Client B, C get this.
    HandleBroadcast(V),
}

impl<K, V> Default for ServerMessage<K, V> {
    fn default() -> Self {
        ServerMessage::GetAll(vec![])
    }
}

#[cfg(test)]
mod tests_command_to_from_string {
    use super::*;
    use std::str::FromStr;
    use strum::IntoEnumIterator;

    #[test]
    fn parse_input() {
        let input = "remove foo";
        let result = ClientMessage::<String, String>::try_parse_input(input);
        let (command, rest) = result.unwrap();
        assert!(matches!(command, ClientMessage::Remove(_)));
        assert_eq!(rest, "foo");
    }

    #[test]
    fn to_string() {
        let commands = ClientMessage::<String, String>::iter()
            .map(|it| it.to_string())
            .collect::<Vec<String>>();
        println!("{:?}", commands);
    }

    #[test]
    fn from_string() {
        let commands = ClientMessage::<String, String>::iter()
            .map(|it| it.to_string().to_lowercase())
            .collect::<Vec<String>>();
        println!("{:?}", commands);

        for command in commands {
            let result = ClientMessage::<String, String>::from_str(&command);
            println!("{:?}", result);
        }
    }
}
