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

use crate::{Action, Mw, State, PROMPT_STR};
use async_trait::async_trait;
use r3bl_rs_utils::{
  fire_and_forget, print_header,
  redux::{AsyncMiddleware, StoreStateMachine},
  utils::print_prompt,
};
use std::sync::Arc;
use tokio::{fs::File, io::AsyncWriteExt, sync::RwLock};

#[derive(Default)]
pub struct SaveCmdMw;

/// https://docs.serde.rs/serde_json/#creating-json-by-serializing-data-structures
/// https://docs.rs/tokio/latest/tokio/fs/struct.File.html
#[async_trait]
impl AsyncMiddleware<State, Action> for SaveCmdMw {
  async fn run(
    &self,
    action: Action,
    store_ref: Arc<RwLock<StoreStateMachine<State, Action>>>,
  ) {
    if let Action::Mw(Mw::SaveCmd) = action {
      fire_and_forget![{
        println!();
        print_header("╭──────────────────────────────────────────────────────╮");
        print_header("│ SaveCmdMw: save to `state.json`                      │");
        print_header("╰──────────────────────────────────────────────────────╯");
        let state = get_state_from(store_ref).await;
        save_state_to_file(state).await;
        print_prompt(PROMPT_STR).unwrap();
      }];
    }
  }
}

async fn save_state_to_file(state: State) {
  let mut file = File::create("state.json")
    .await
    .unwrap();
  let json = serde_json::to_string_pretty(&state).unwrap();
  file
    .write_all(json.as_bytes())
    .await
    .unwrap();
}

async fn get_state_from(
  store_ref: Arc<RwLock<StoreStateMachine<State, Action>>>
) -> State {
  store_ref
    .write()
    .await
    .get_state_clone()
}
