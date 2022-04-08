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

use std::{collections::HashMap, error::Error};

use reqwest::{header::HeaderMap, StatusCode};

const ENDPOINT: &str = "http://httpbin.org/ip";

pub struct IpResponse {
  pub payload: HashMap<String, String>,
  pub endpoint: String,
  pub status: StatusCode,
  pub headers: HeaderMap,
}

pub async fn get_ip() -> Result<IpResponse, Box<dyn Error>> {
  let res = reqwest::get(ENDPOINT).await?;

  let status = res.status();
  let headers = res.headers().clone();
  let payload = res
    .json::<HashMap<String, String>>()
    .await?;

  Ok(IpResponse {
    payload,
    status,
    headers,
    endpoint: ENDPOINT.to_string(),
  })
}
