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

use std::str::FromStr;

use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

/// Size (number of bytes) to read from the stream to get the length prefix.
pub type LengthPrefixType = u64;
pub type Buffer = Vec<u8>;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};

pub mod byte_io {
    use super::*;

    /// Write the payload to the client. Use the length-prefix, binary payload, protocol.
    /// - The trait bounds on this function are so that this function can be tested w/ a
    ///   mock from [tokio_test::io::Builder].
    /// - More info: <https://tokio.rs/tokio/topics/testing>
    pub async fn write<Writer: AsyncWrite + Unpin>(
        buf_writer: &mut BufWriter<Writer>,
        payload_buffer: Buffer,
    ) -> miette::Result<()> {
        let payload_size = payload_buffer.len();

        // Write the length prefix number of bytes.
        buf_writer
            .write_u64(payload_size as LengthPrefixType)
            .await
            .into_diagnostic()?;

        // Write the payload.
        buf_writer
            .write_all(&payload_buffer)
            .await
            .into_diagnostic()?;

        // Flush the buffer.
        buf_writer.flush().await.into_diagnostic()?;

        Ok(())
    }

    /// Ready the payload from the client. Use the length-prefix [LengthPrefixType],
    /// binary payload, protocol.
    /// - The trait bounds on this function are so that this function can be tested w/ a
    ///   mock from [tokio_test::io::Builder].
    /// - More info: <https://tokio.rs/tokio/topics/testing>
    pub async fn read<Reader: AsyncRead + Unpin>(
        buf_reader: &mut BufReader<Reader>,
    ) -> miette::Result<Buffer> {
        // Read the length prefix number of bytes.
        let size_of_payload = buf_reader.read_u64().await.into_diagnostic()?;

        // Read the payload.
        let mut payload_buffer = vec![0; size_of_payload as usize];
        buf_reader
            .read_exact(&mut payload_buffer)
            .await
            .into_diagnostic()?;

        Ok(payload_buffer)
    }
}

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
    #[strum(ascii_case_insensitive)]
    Exit,

    #[default]
    #[strum(ascii_case_insensitive)]
    GetAll,

    #[strum(ascii_case_insensitive)]
    Insert(K, V),

    #[strum(ascii_case_insensitive)]
    Remove(K),

    #[strum(ascii_case_insensitive)]
    Get(K),

    #[strum(ascii_case_insensitive)]
    Clear,

    // CLEANUP: impl the messages below
    #[strum(ascii_case_insensitive)]
    Size,

    #[strum(ascii_case_insensitive)]
    BroadcastToOthers(V), /* Client A initiates this. It gets BroadcastToOthersAck(..). Other clients get HandleBroadcast(..) */
}

impl<K: Default, V: Default> ClientMessage<K, V> {
    pub fn parse_input(input: &str) -> Result<(Self, String), strum::ParseError> {
        // If input is empty, then return the default command.
        if input.is_empty() {
            return Ok((ClientMessage::default(), "".to_string()));
        }

        // If input has a space, then split it and use the first part as the command.
        let parts: Vec<&str> = input.split_whitespace().collect();
        // Same as: input.parse::<protocol::ClientMessage>();
        let client_message = ClientMessage::<K, V>::from_str(parts[0])?;
        let rest = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            "".to_string()
        };

        Ok((client_message, rest))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ServerMessage<K, V> {
    SetClientId(String),
    Exit,
    GetAll(Vec<(K, V)>),
    Insert(bool),
    Remove(bool),
    Get(Option<V>),
    Clear(bool),
    // CLEANUP: impl the messages below
    Size(usize),
    HandleBroadcast(V), /* Client A initiates BroadcastToOthers(..). Client B, C get this. */
    BroadcastToOthersAck(Vec<(String, bool)>), /* Client A initiates BroadcastToOthers(..). Client A gets this. */
}

impl<K, V> Default for ServerMessage<K, V> {
    fn default() -> Self {
        ServerMessage::GetAll(vec![])
    }
}

pub struct SerializeHelperData {
    pub size: usize,
    pub bytes: Buffer,
}

pub fn serialize_helper(value: &impl Serialize) -> miette::Result<SerializeHelperData> {
    let bytes: Buffer = bincode::serialize(value).into_diagnostic()?;
    Ok(SerializeHelperData {
        size: bytes.len(),
        bytes,
    })
}

#[cfg(test)]
mod tests_command_to_from_string {
    use super::*;
    use std::str::FromStr;
    use strum::IntoEnumIterator;

    #[test]
    fn parse_input() {
        let input = "remove foo";
        let result = ClientMessage::<String, String>::parse_input(input);
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

#[cfg(test)]
mod test_fixtures {
    use super::*;

    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    pub struct TestPayload {
        pub id: f32,
        pub description: String,
        pub data: Buffer,
    }
}

#[cfg(test)]
mod tests_serialize_helper {
    use super::test_fixtures::*;
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_serialize_helper() {
        let sample_data = TestPayload {
            id: 12.0,
            description: "foo bar".to_string(),
            data: vec![0, 1, 2],
        };
        let result = serialize_helper(&sample_data);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.size, 30);
        assert_eq!(data.bytes.len(), 30);

        let sample_data_deserialized: TestPayload = bincode::deserialize(&data.bytes).unwrap();
        pretty_assertions::assert_eq!(sample_data, sample_data_deserialized);
    }
}

/// More info:
/// - [what is bincode](https://docs.rs/bincode/latest/bincode/)
/// - [what is codec](https://g.co/bard/share/cbf732b548c7)
///
/// [bincode] is a crate for encoding and decoding using a tiny binary serialization
/// strategy. Using it, you can easily go from having an struct / object in memory,
/// quickly serialize it to bytes, and then deserialize it back just as fast!
#[cfg(test)]
mod tests_bincode_serde {
    use super::test_fixtures::*;
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bincode_serde() -> miette::Result<()> {
        let value = TestPayload {
            id: 12.0,
            description: "foo bar".to_string(),
            data: vec![0, 1, 2],
        };

        // Struct (MyValueType) -> Bytes (Buffer).
        let result_struct_to_bytes: Result<Buffer, Box<bincode::ErrorKind>> =
            bincode::serialize(&value);
        assert!(result_struct_to_bytes.is_ok());
        let struct_to_bytes: Buffer = result_struct_to_bytes.into_diagnostic()?;
        println!("{:?}", struct_to_bytes);

        // Bytes (Buffer) -> Struct (MyValueType).
        let result_struct_from_bytes: Result<TestPayload, Box<bincode::ErrorKind>> =
            bincode::deserialize(&struct_to_bytes);
        assert!(result_struct_from_bytes.is_ok());
        let struct_from_bytes: TestPayload = result_struct_from_bytes.into_diagnostic()?;
        println!("{:?}", struct_from_bytes);

        assert_eq!(value, struct_from_bytes);

        Ok(())
    }
}
