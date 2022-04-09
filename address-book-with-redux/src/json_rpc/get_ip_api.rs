/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

use std::{collections::HashMap, error::Error, fmt::Display};

use reqwest::{header::HeaderMap, StatusCode};

const ENDPOINT: &str = "http://httpbin.org/ip";

pub struct IpResponse {
  pub payload: HashMap<String, String>,
  pub endpoint: String,
  pub status: StatusCode,
  pub headers: HeaderMap,
}

impl IpResponse {
  fn to_string(&self) -> String {
    format!(
      "payload: {:#?}\n  endpoint: {}\n  status: {}\n  headers: {}",
      self.payload,
      self.endpoint,
      self.status.to_string(),
      format!("{:#?}", self.headers)
    )
  }
}

impl Display for IpResponse {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

/// Example of the response from the [API](http://httpbin.org/ip):
/// ```no_run
/// {
///   "origin": "206.55.172.90"
/// }
/// ```
pub async fn get_ip() -> Result<IpResponse, Box<dyn Error>> {
  let res = reqwest::get(ENDPOINT).await?;

  let status = res.status();
  let headers = res.headers().clone();

  // Get response as text.
  let response_text = res.text().await?;

  // Convert the response text to a `serde_json::Value`.
  // https://docs.serde.rs/serde_json/#operating-on-untyped-json-values
  let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

  // Convert the `serde_json::Value` to a `HashMap`.
  let key = "origin";
  let value = response_json[key]
    .as_str()
    .unwrap();
  let mut payload_map = HashMap::new();
  payload_map.insert(key.to_string(), value.to_string());

  // Return the `HashMap`, etc. as a `IpResponse`.
  Ok(IpResponse {
    payload: payload_map,
    status,
    headers,
    endpoint: ENDPOINT.to_string(),
  })
}
