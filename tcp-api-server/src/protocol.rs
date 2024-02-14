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

use std::str::FromStr;

use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, net::tcp::OwnedReadHalf};
use tokio::{
    io::{AsyncReadExt, BufReader, BufWriter},
    net::tcp::OwnedWriteHalf,
};

pub type LengthPrefixType = u64;

/// Write the payload to the client. Use the length-prefix, binary payload, protocol.
pub async fn write_bytes(
    buf_writer: &mut BufWriter<OwnedWriteHalf>,
    payload_buffer: Vec<u8>,
) -> miette::Result<()> {
    let size_of_payload = payload_buffer.len();

    // Write the length prefix number of bytes.
    buf_writer
        .write_u64(size_of_payload as LengthPrefixType)
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

/// Ready the payload from the client. Use the length-prefix, binary payload, protocol.
pub async fn read_bytes(buf_reader: &mut BufReader<OwnedReadHalf>) -> miette::Result<Vec<u8>> {
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum ClientMessage {
    Insert(MyKey, MyPayload),
    Remove(MyKey),
    Get(MyKey),
    #[default]
    GetAll,
    Clear,
    Size,
    ContainsKey(MyKey),
    ContainsValue(MyPayload),
    Keys,
    Values,
    IsEmpty,
    Exit,
    BroadcastToOthers(MyPayload), /* Client A initiates this. It gets BroadcastToOthersAck(..). Other clients get HandleBroadcast(..) */
}

impl FromStr for ClientMessage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "insert" => Ok(ClientMessage::Insert(
                MyKey::default(),
                MyPayload::default(),
            )),
            "remove" => Ok(ClientMessage::Remove(MyKey::default())),
            "get" => Ok(ClientMessage::Get(MyKey::default())),
            "get_all" => Ok(ClientMessage::GetAll),
            "clear" => Ok(ClientMessage::Clear),
            "size" => Ok(ClientMessage::Size),
            "contains_key" => Ok(ClientMessage::ContainsKey(MyKey::default())),
            "contains_value" => Ok(ClientMessage::ContainsValue(MyPayload::default())),
            "keys" => Ok(ClientMessage::Keys),
            "values" => Ok(ClientMessage::Values),
            "is_empty" => Ok(ClientMessage::IsEmpty),
            "exit" => Ok(ClientMessage::Exit),
            "broadcast_to_others" => Ok(ClientMessage::BroadcastToOthers(MyPayload::default())),
            _ => Err(format!("Unknown command: {}", s)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ServerMessage {
    SetClientId(String),
    Insert(bool),
    Remove(bool),
    Get(Option<MyPayload>),
    GetAll(Vec<(MyKey, MyPayload)>),
    Clear,
    Size(usize),
    ContainsKey(bool),
    ContainsValue(bool),
    Keys(Vec<MyKey>),
    Values(Vec<MyPayload>),
    IsEmpty(bool),
    Exit,
    HandleBroadcast(MyPayload), /* Client A initiates BroadcastToOthers(..). Client B, C get this. */
    BroadcastToOthersAck(Vec<(String, bool)>), /* Client A initiates BroadcastToOthers(..). Client A gets this. */
}

impl Default for ServerMessage {
    fn default() -> Self {
        ServerMessage::GetAll(vec![])
    }
}

/// Just a sample value or payload type. Replace this with whatever type you want to use.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MyPayload {
    pub id: f32,
    pub description: String,
    pub data: Vec<u8>,
}

/// Just a sample key type. Replace this with whatever type you want to use.
pub type MyKey = String;

pub struct SerializeHelperData {
    pub size: usize,
    pub bytes: Vec<u8>,
}

pub fn serialize_helper(value: &impl Serialize) -> miette::Result<SerializeHelperData> {
    let bytes: Vec<u8> = bincode::serialize(value).into_diagnostic()?;
    Ok(SerializeHelperData {
        size: bytes.len(),
        bytes,
    })
}

#[cfg(test)]
mod serialize_helper_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_serialize_helper() {
        let sample_data = MyPayload {
            id: 12.0,
            description: "foo bar".to_string(),
            data: vec![0, 1, 2],
        };
        let result = serialize_helper(&sample_data);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.size, 30);
        assert_eq!(data.bytes.len(), 30);

        let sample_data_deserialized: MyPayload = bincode::deserialize(&data.bytes).unwrap();
        assert_eq!(sample_data, sample_data_deserialized);
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
mod bincode_serde_tests {
    use super::*;

    #[test]
    fn test_bincode_serde() -> miette::Result<()> {
        let value = MyPayload {
            id: 12.0,
            description: "foo bar".to_string(),
            data: vec![0, 1, 2],
        };

        // Struct (MyValueType) -> Bytes (Vec<u8>).
        let result_struct_to_bytes: Result<Vec<u8>, Box<bincode::ErrorKind>> =
            bincode::serialize(&value);
        assert!(result_struct_to_bytes.is_ok());
        let struct_to_bytes: Vec<u8> = result_struct_to_bytes.into_diagnostic()?;
        println!("{:?}", struct_to_bytes);

        // Bytes (Vec<u8>) -> Struct (MyValueType).
        let result_struct_from_bytes: Result<MyPayload, Box<bincode::ErrorKind>> =
            bincode::deserialize(&struct_to_bytes);
        assert!(result_struct_from_bytes.is_ok());
        let struct_from_bytes: MyPayload = result_struct_from_bytes.into_diagnostic()?;
        println!("{:?}", struct_from_bytes);

        assert_eq!(value, struct_from_bytes);

        Ok(())
    }
}
