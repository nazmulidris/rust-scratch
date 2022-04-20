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

// Imports.
use crate::{
  load_cmd_mw::LoadCmdMw, Action, AddAsyncCmdMw, AirCmdMw, IpCmdMw, LoggerMw, Mw,
  MyReducer, Renderer, SaveCmdMw, State, Std,
};
use r3bl_rs_utils::{
  print_header,
  redux::{AsyncMiddleware, AsyncReducer, AsyncSubscriber, Store},
  style_dimmed, style_error, style_primary,
  utils::readline_with_prompt,
};
use rand::random;
use std::error::Error;

pub async fn run_tui_app(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
  repl_loop(create_store().await).await?;
  Ok(())
}

async fn create_store() -> Store<State, Action> {
  let mut store = Store::<State, Action>::default();
  store
    .add_subscriber(Renderer::new())
    .await
    .add_middleware(LoggerMw::new())
    .await
    .add_middleware(AirCmdMw::new())
    .await
    .add_middleware(IpCmdMw::new())
    .await
    .add_middleware(AddAsyncCmdMw::new())
    .await
    .add_middleware(SaveCmdMw::new())
    .await
    .add_middleware(LoadCmdMw::new())
    .await
    .add_reducer(MyReducer::new())
    .await;
  store
}

const AVAIL_CMDS: &str =
  "quit, exit, add-async, add-sync, save, clear, remove, reset, search, history, ip, help";

pub async fn repl_loop(store: Store<State, Action>) -> Result<(), Box<dyn Error>> {
  print_header("Starting repl");

  // Repl.
  loop {
    let user_input = readline_with_prompt("r3bl> ")?;
    match user_input.as_str() {
      "help" => {
        println!(
          "{}: {}",
          style_primary("Available commands"),
          style_dimmed(AVAIL_CMDS)
        );
      }
      "quit" => break,
      "exit" => break,
      "add-sync" => {
        let id = random::<u8>();
        let action = Action::Std(Std::AddContact(
          format!("John Doe #{}", id),
          format!("jd@gmail.com #{}", id),
          format!("123-456-7890 #{}", id),
        ));
        store.dispatch_spawn(action);
      }
      "clear" => {
        let action = Action::Std(Std::RemoveAllContacts);
        store.dispatch_spawn(action);
      }
      "remove" => {
        match readline_with_prompt("id> ") {
          Ok(id) => {
            let action = Action::Std(Std::RemoveContactById(
              id.parse().unwrap(),
            ));
            store.dispatch_spawn(action)
          }
          Err(_) => println!("{}", style_error("Invalid id")),
        };
      }
      "search" => {
        match readline_with_prompt("search_term> ") {
          Ok(search_term) => {
            let action = Action::Std(Std::Search(search_term));
            store.dispatch_spawn(action)
          }
          Err(_) => println!("{}", style_error("Invalid id")),
        };
      }
      "reset" => {
        let action = Action::Std(Std::ResetState(State::default()));
        store.dispatch_spawn(action);
      }
      "history" => {
        println!("{:#?}", store.get_history().await);
      }
      "add-async" => {
        let action = Action::Mw(Mw::AsyncAddCmd);
        store.dispatch_spawn(action);
        println!(
          "{}",
          "🧵 Spawning add_async_cmd_mw.rs ..."
        );
      }
      "ip" => {
        let action = Action::Mw(Mw::IpCmd);
        store.dispatch_spawn(action);
        println!(
          "{}",
          "🧵 Spawning ip_cmd_mw.rs ..."
        );
      }
      "air" => {
        let action = Action::Mw(Mw::AirCmd);
        store.dispatch_spawn(action);
        println!(
          "{}",
          "🧵 Spawning air_cmd_mw.rs ..."
        );
      }
      "save" => {
        let action = Action::Mw(Mw::SaveCmd);
        store.dispatch_spawn(action);
        println!(
          "{}",
          "🧵 Spawning save_cmd_mw.rs ..."
        );
      }
      "load" => {
        let action = Action::Mw(Mw::LoadCmd);
        store.dispatch_spawn(action);
        println!(
          "{}",
          "🧵 Spawning load_cmd_mw.rs ..."
        );
      }
      // Catchall.
      _ => {
        println!(
          "{}",
          style_error("Unknown command")
        );
      }
    }; // end match user_input.

    // Print confirmation at the end of 1 repl loop.
    println!(
      "{} {}",
      style_primary(&user_input),
      style_dimmed("was executed.")
    );
  }

  Ok(())
}
