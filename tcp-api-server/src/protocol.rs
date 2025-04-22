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
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use miette::IntoDiagnostic;
use r3bl_tui::ok;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::str::FromStr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::time::timeout;

/// Type alias for type to read from the stream to get the length prefix.
pub type LengthPrefixType = u64;
/// Type aliases for the payload buffer type.
pub type Buffer = Vec<BufferAtom>;
pub type BufferAtom = u8;

pub mod protocol_constants {
    use super::*;

    pub const MAGIC_NUMBER: u64 = 0xDEADBEEFCAFEBABE;
    pub const PROTOCOL_VERSION: u64 = 1;
    pub const TIMEOUT_DURATION: Duration = Duration::from_secs(1);
    pub const MAX_PAYLOAD_SIZE: u64 = 10_000_000;
}

pub mod compression {
    use super::*;

    /// Compress the payload using the [flate2] crate.
    pub fn compress(data: &[BufferAtom]) -> miette::Result<Buffer> {
        let uncompressed_size = data.len();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).into_diagnostic()?;
        let it = encoder.finish().into_diagnostic();
        let compressed_size = it.as_ref().map(|it| it.len()).unwrap_or(0);
        tracing::info!(
            "Compression: {:.2} kb -> {:.2} kb ({:.2}%)",
            uncompressed_size as f64 / 1000.0,
            compressed_size as f64 / 1000.0,
            (compressed_size as f64 / uncompressed_size as f64) * 100.0
        );
        it
    }

    /// Decompress the payload using the [flate2] crate.
    pub fn decompress(data: &[BufferAtom]) -> miette::Result<Buffer> {
        let compressed_size = data.len();
        let mut decoder = GzDecoder::new(data);
        let mut decompressed_data = Vec::new();
        decoder
            .read_to_end(&mut decompressed_data)
            .into_diagnostic()?;
        let uncompressed_size = decompressed_data.len();
        tracing::info!(
            "Decompression: {:.2} kb -> {:.2} kb ({:.2}%)",
            uncompressed_size as f64 / 1000.0,
            compressed_size as f64 / 1000.0,
            (compressed_size as f64 / uncompressed_size as f64) * 100.0
        );
        Ok(decompressed_data)
    }
}

/// Extend the protocol to validate that it is connecting to the correct type of server,
/// by implementing the following handshake mechanism:
///
/// # Client side - [handshake::try_connect_or_timeout]
/// 1. The client **writes** a "magic number" or protocol identifier, and version number
///    as the first message when establishing a connection.
/// 2. This number is then **read** back from the server to ensure that it is valid.
///
/// # Server side - [handshake::try_accept_or_timeout]
/// 1. The server **reads** the magic number and protocol version number, and checks to
///    make sure they are valid.
/// 2. It then **writes** the magic number back to the client (for it to validate).
pub mod handshake {
    use super::*;

    /// Client side handshake.
    pub async fn try_connect_or_timeout<W: AsyncWrite + Unpin, R: AsyncRead + Unpin>(
        read_half: &mut R,
        write_half: &mut W,
    ) -> miette::Result<()> {
        let result = timeout(
            protocol_constants::TIMEOUT_DURATION,
            try_connect(read_half, write_half),
        )
        .await;

        match result {
            Ok(Err(handshake_err)) => {
                miette::bail!("Handshake failed due to: {}", handshake_err.root_cause())
            }
            Err(_elapsed_err) => {
                miette::bail!("Handshake timed out")
            }
            _ => {
                ok!()
            }
        }
    }

    async fn try_connect<W: AsyncWrite + Unpin, R: AsyncRead + Unpin>(
        read_half: &mut R,
        write_half: &mut W,
    ) -> miette::Result<()> {
        // Send the magic number.
        write_half
            .write_u64(protocol_constants::MAGIC_NUMBER)
            .await
            .into_diagnostic()?;

        // Send the protocol version.
        write_half
            .write_u64(protocol_constants::PROTOCOL_VERSION)
            .await
            .into_diagnostic()?;

        // Flush the buffer.
        write_half.flush().await.into_diagnostic()?;

        // Read the magic number back from the server.
        let received_magic_number = read_half.read_u64().await.into_diagnostic()?;
        if received_magic_number != protocol_constants::MAGIC_NUMBER {
            miette::bail!("Invalid protocol magic number")
        }

        ok!()
    }

    /// Server side handshake.
    pub async fn try_accept_or_timeout<W: AsyncWrite + Unpin, R: AsyncRead + Unpin>(
        read_half: &mut R,
        write_half: &mut W,
    ) -> miette::Result<()> {
        let result = timeout(
            protocol_constants::TIMEOUT_DURATION,
            try_accept(read_half, write_half),
        )
        .await
        .into_diagnostic();

        match result {
            Ok(handshake_result) => match handshake_result {
                Ok(_) => ok!(),
                Err(handshake_err) => {
                    miette::bail!("Handshake failed due to: {}", handshake_err.root_cause())
                }
            },
            Err(_elapsed_err) => miette::bail!("Handshake timed out"),
        }
    }

    async fn try_accept<W: AsyncWrite + Unpin, R: AsyncRead + Unpin>(
        read_half: &mut R,
        write_half: &mut W,
    ) -> miette::Result<()> {
        // Read and validate the magic number.
        let received_magic_number = read_half.read_u64().await.into_diagnostic()?;
        if received_magic_number != protocol_constants::MAGIC_NUMBER {
            miette::bail!("Invalid protocol magic number")
        }

        // Read and validate the protocol version.
        let received_protocol_version = read_half.read_u64().await.into_diagnostic()?;
        if received_protocol_version != protocol_constants::PROTOCOL_VERSION {
            miette::bail!("Invalid protocol version")
        }

        // Write the magic number back to the client.
        write_half
            .write_u64(protocol_constants::MAGIC_NUMBER)
            .await
            .into_diagnostic()?;

        ok!()
    }
}

#[cfg(test)]
mod tests_handshake {
    use super::*;
    use r3bl_tui::{get_mock_socket_halves, MockSocket};

    #[tokio::test]
    async fn test_handshake() {
        let MockSocket {
            mut client_read,
            mut client_write,
            mut server_read,
            mut server_write,
        } = get_mock_socket_halves();

        let client_handshake =
            handshake::try_connect_or_timeout(&mut client_read, &mut client_write);

        let server_handshake =
            handshake::try_accept_or_timeout(&mut server_read, &mut server_write);

        let (client_handshake_result, server_handshake_result) =
            tokio::join!(client_handshake, server_handshake);

        assert!(client_handshake_result.is_ok());
        assert!(server_handshake_result.is_ok());
    }
}

/// <https://github.com/bincode-org/bincode/blob/trunk/docs/migration_guide.md>
pub mod byte_io {
    use super::*;

    /// Write the payload to the client. Use the length-prefix, binary payload, protocol.
    /// - The trait bounds on this function are so that this function can be tested w/ a
    ///   mock from [tokio_test::io::Builder].
    /// - More info: <https://tokio.rs/tokio/topics/testing>
    pub async fn try_write<W: AsyncWrite + Unpin, T: Serialize>(
        buf_writer: &mut BufWriter<W>,
        data: &T,
    ) -> miette::Result<()> {
        // Try to serialize the data.
        let config = bincode::config::standard();
        let payload_buffer = bincode::serde::encode_to_vec(data, config).into_diagnostic()?;

        // Compress the payload.
        let payload_buffer = compression::compress(&payload_buffer)?;

        // Write the length prefix number of bytes.
        let payload_size = payload_buffer.len();
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
    pub async fn try_read<R: AsyncRead + Unpin, T: for<'d> Deserialize<'d>>(
        buf_reader: &mut BufReader<R>,
    ) -> miette::Result<T> {
        // Read the length prefix number of bytes.
        let size_of_payload = buf_reader.read_u64().await.into_diagnostic()?;

        // Ensure that the payload size is within the expected range.
        if size_of_payload > protocol_constants::MAX_PAYLOAD_SIZE {
            // Adjust this threshold as needed
            miette::bail!("Payload size is too large")
        }

        // Read the payload.
        let mut payload_buffer = vec![0; size_of_payload as usize];
        buf_reader
            .read_exact(&mut payload_buffer)
            .await
            .into_diagnostic()?;

        // Decompress the payload.
        let payload_buffer = compression::decompress(&payload_buffer)?;

        // Deserialize the payload.
        let config = bincode::config::standard();
        let (payload_buffer, _bytes_read) =
            bincode::serde::decode_from_slice(&payload_buffer, config).into_diagnostic()?;

        Ok(payload_buffer)
    }
}

#[cfg(test)]
mod tests_byte_io {
    use super::*;
    use r3bl_tui::{get_mock_socket_halves, MockSocket};

    #[tokio::test]
    async fn test_byte_io() {
        let MockSocket {
            client_read: _,
            mut client_write,
            mut server_read,
            server_write: _,
        } = get_mock_socket_halves();

        for sent_payload in test_fixtures::get_all_client_messages() {
            byte_io::try_write(&mut BufWriter::new(&mut client_write), &sent_payload)
                .await
                .unwrap();

            let received_payload: ClientMessage<String, String> =
                byte_io::try_read(&mut BufReader::new(&mut server_read))
                    .await
                    .unwrap();

            assert_eq!(received_payload, sent_payload);
        }
    }
}

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

#[cfg(test)]
mod test_fixtures {
    use super::*;

    pub fn get_all_client_messages() -> Vec<ClientMessage<String, String>> {
        use strum::IntoEnumIterator as _;
        ClientMessage::iter().collect()
    }

    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    pub struct TestPayload {
        pub id: f32,
        pub description: String,
        pub data: Buffer,
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
        let config = bincode::config::standard();
        let result_struct_to_bytes = bincode::serde::encode_to_vec(&value, config);

        assert!(result_struct_to_bytes.is_ok());
        let struct_to_bytes: Buffer = result_struct_to_bytes.into_diagnostic()?;
        println!("{:?}", struct_to_bytes);

        // Bytes (Buffer) -> Struct (MyValueType).
        let res = bincode::serde::decode_from_slice::<TestPayload, _>(&struct_to_bytes, config);
        assert!(res.is_ok());
        let (result_struct_from_bytes, _bytes_read) = res.into_diagnostic()?;
        let struct_from_bytes: TestPayload = result_struct_from_bytes;
        println!("{:?}", struct_from_bytes);

        assert_eq!(value, struct_from_bytes);

        Ok(())
    }
}
