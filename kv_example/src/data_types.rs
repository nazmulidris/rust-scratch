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

/// Just a sample value or payload type. Replace this with whatever type you want to use.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MyValueType {
    pub id: f32,
    pub description: String,
    pub data: Vec<u8>,
}
/// Just a sample key type. Replace this with whatever type you want to use.
pub type MyKeyType = String;

/// [bincode] is a crate for encoding and decoding using a tiny binary serialization
/// strategy. Using it, you can easily go from having an object in memory, quickly
/// serialize it to bytes, and then deserialize it back just as fast!
#[test]
fn bincode_serde() -> std::result::Result<(), Box<bincode::ErrorKind>> {
    let value = MyValueType {
        id: 12.0,
        description: "foo bar".to_string(),
        data: vec![0, 1, 2],
    };

    let result_struct_to_bytes: Result<Vec<u8>, Box<bincode::ErrorKind>> =
        bincode::serialize(&value);
    assert!(result_struct_to_bytes.is_ok());
    let struct_to_bytes: Vec<u8> = result_struct_to_bytes?;
    println!("{:?}", struct_to_bytes);

    let result_struct_from_bytes: Result<MyValueType, Box<bincode::ErrorKind>> =
        bincode::deserialize(&struct_to_bytes);
    assert!(result_struct_from_bytes.is_ok());
    let struct_from_bytes: MyValueType = result_struct_from_bytes?;
    println!("{:?}", struct_from_bytes);

    assert_eq!(value, struct_from_bytes);

    Ok(())
}
