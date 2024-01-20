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

use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};

pub fn random_number<T>() -> T
where
    Standard: Distribution<T>,
{
    thread_rng().gen::<T>()
}

/// Convenience type alias for [std::result::Result].
pub type MyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Just a sample value or payload type. Replace this with whatever type you want to use.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MyValueType {
    pub id: f32,
    pub description: String,
    pub data: Vec<u8>,
}
/// Just a sample key type. Replace this with whatever type you want to use.
pub type MyKeyType = String;
