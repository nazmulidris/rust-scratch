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

use crate::{Action, Mw, State, Std, STATE_JSON_FNAME};
use async_trait::async_trait;
use r3bl_rs_utils::{
  fire_and_forget, print_header,
  redux::{AsyncMiddleware, StoreStateMachine},
};
use std::sync::Arc;
use tokio::{fs::File, io::AsyncReadExt, sync::RwLock};

#[derive(Default)]
pub struct LoadCmdMw;

/// https://docs.serde.rs/serde_json/#parsing-json-as-strongly-typed-data-structures
/// https://docs.rs/tokio/latest/tokio/fs/struct.File.html
#[async_trait]
impl AsyncMiddleware<State, Action> for LoadCmdMw {
  async fn run(
    &self,
    action: Action,
    store_ref: Arc<RwLock<StoreStateMachine<State, Action>>>,
  ) {
    if let Action::Mw(Mw::LoadCmd) = action {
      fire_and_forget![{
        println!();
        print_header("╭──────────────────────────────────────────────────────╮");
        print_header("│ LoadCmdMw: load from `state.json` coming soon!       │");
        print_header("╰──────────────────────────────────────────────────────╯");
        let json_str = load_str_from_file(STATE_JSON_FNAME).await;
        let state = get_state_from(json_str).await;
        let action = Action::Std(Std::ResetState(state));
        store_ref
          .write()
          .await
          .dispatch_action(action, store_ref.clone())
          .await;
      }];
    }
  }
}

async fn load_str_from_file(fname: &str) -> String {
  let mut file = File::open(fname).await.unwrap();
  let mut file_content_str = String::new();
  file
    .read_to_string(&mut file_content_str)
    .await
    .unwrap();
  file_content_str
}

async fn get_state_from(json_str: String) -> State {
  let state: State = serde_json::from_str(&json_str).unwrap();
  state
}
