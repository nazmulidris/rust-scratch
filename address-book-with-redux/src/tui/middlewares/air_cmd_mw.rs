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

use crate::{
  json_rpc::awair_local_api::make_request as awair_local_api, Action, Mw, State,
};
use async_trait::async_trait;
use r3bl_rs_utils::{
  fire_and_forget,
  redux::{AsyncMiddleware, StoreStateMachine},
  style_error,
  utils::print_prompt,
};

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct AirCmdMw;

#[async_trait]
impl AsyncMiddleware<State, Action> for AirCmdMw {
  async fn run(
    &self,
    action: Action,
    _store_ref: Arc<RwLock<StoreStateMachine<State, Action>>>,
  ) {
    fire_and_forget![{
      if let Action::Mw(Mw::AirCmd) = action {
        match awair_local_api().await {
          Ok(resp_data) => {
            println!("{:#?}", resp_data);
            print_prompt("r3bl> ").unwrap();
          }
          Err(e) => println!("{}", style_error(&e.to_string())),
        };
      }
    }];
  }
}