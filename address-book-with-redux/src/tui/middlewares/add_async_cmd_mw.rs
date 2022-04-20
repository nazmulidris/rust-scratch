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
  json_rpc::{
    fake_contact_data_api::make_request as fake_contact_data_api, FakeContactData,
  },
  Action, Mw, State, Std, PROMPT_STR,
};
use async_trait::async_trait;
use r3bl_rs_utils::{
  fire_and_forget,
  redux::{AsyncMiddleware, StoreStateMachine}, utils::print_prompt,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct AddAsyncCmdMw;

#[async_trait]
impl AsyncMiddleware<State, Action> for AddAsyncCmdMw {
  async fn run(
    &self,
    action: Action,
    store_ref: Arc<RwLock<StoreStateMachine<State, Action>>>,
  ) {
    fire_and_forget![{
      if let Action::Mw(Mw::AsyncAddCmd) = action {
        let fake_data = fake_contact_data_api()
          .await
          .unwrap_or_else(|_| FakeContactData {
            name: "Foo Bar".to_string(),
            phone_h: "123-456-7890".to_string(),
            email_u: "foo".to_string(),
            email_d: "bar.com".to_string(),
            ..FakeContactData::default()
          });

        let action = Action::Std(Std::AddContact(
          format!("{}", fake_data.name),
          format!(
            "{}@{}",
            fake_data.email_u, fake_data.email_d
          ),
          format!("{}", fake_data.phone_h),
        ));

        store_ref
          .write()
          .await
          .dispatch_action(action, store_ref.clone())
          .await;

        print_prompt(PROMPT_STR).unwrap();
      }
    }];
  }
}
