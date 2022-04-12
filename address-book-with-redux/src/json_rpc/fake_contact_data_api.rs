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

use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

const ENDPOINT: &str = "https://api.namefake.com/english-united-states/female/";

pub async fn make_request() -> Result<FakeContactData, Box<dyn Error>> {
  let res = reqwest::get(ENDPOINT).await?;
  let res_text = res.text().await?;
  let res_json: FakeContactData = serde_json::from_str(&res_text)?;
  Ok(res_json)
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FakeContactData {
  pub name: String,
  pub phone_h: String,
  pub email_u: String,
  pub email_d: String,
  pub address: String,
  pub latitude: f64,
  pub longitude: f64,
  pub maiden_name: String,
  pub birth_data: String,
  pub phone_w: String,
  pub username: String,
  pub password: String,
  pub domain: String,
  pub useragent: String,
  pub ipv4: String,
  pub macaddress: String,
  pub plasticcard: String,
  pub cardexpir: String,
  pub bonus: i64,
  pub company: String,
  pub color: String,
  pub uuid: String,
  pub height: i64,
  pub weight: f64,
  pub blood: String,
  pub eye: String,
  pub hair: String,
  pub pict: String,
  pub url: String,
  pub sport: String,
  pub ipv4_url: String,
  pub email_url: String,
  pub domain_url: String,
}

impl FakeContactData {
  pub fn to_string(&self) -> String {
    serde_json::to_string(&self).unwrap()
  }
}

impl Display for FakeContactData {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}
