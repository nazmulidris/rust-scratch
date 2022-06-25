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
  do_load, do_save, load_cmd_mw::LoadCmdMw, Action, AddAsyncCmdMw, AirCmdMw, IpCmdMw, LoggerMw, Mw,
  MyReducer, Renderer, SaveCmdMw, State, Std,
};
use r3bl_rs_utils::{
  print_header,
  redux::{AsyncMiddleware, AsyncMiddlewareSpawns, AsyncReducer, AsyncSubscriber, Store},
  spawn_dispatch_action, style_dimmed, style_error, style_primary,
  utils::readline_with_prompt,
  SharedStore,
};
use rand::random;
use std::{error::Error, sync::Arc};
use tokio::sync::RwLock;

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
    .add_middleware_spawns(AddAsyncCmdMw::new())
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
  "quit, exit, add-async, add-sync, save, load, clear, remove, reset, search, history, \
  ip, air, help";

pub async fn repl_loop(mut _store: Store<State, Action>) -> Result<(), Box<dyn Error>> {
  let shared_store: SharedStore<State, Action> = Arc::new(RwLock::new(_store));
  print_header("Starting repl");
  on_start(&shared_store.clone()).await;

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
        spawn_dispatch_action!(shared_store, action);
      }
      "clear" => {
        let action = Action::Std(Std::RemoveAllContacts);
        spawn_dispatch_action!(shared_store, action);
      }
      "remove" => {
        match readline_with_prompt("id> ") {
          Ok(id) => {
            let action = Action::Std(Std::RemoveContactById(id.parse().unwrap()));
            spawn_dispatch_action!(shared_store, action);
          }
          Err(_) => println!("{}", style_error("Invalid id")),
        };
      }
      "search" => {
        match readline_with_prompt("search_term> ") {
          Ok(search_term) => {
            let action = Action::Std(Std::Search(search_term));
            spawn_dispatch_action!(shared_store, action);
          }
          Err(_) => println!("{}", style_error("Invalid id")),
        };
      }
      "reset" => {
        let action = Action::Std(Std::ResetState(State::default()));
        spawn_dispatch_action!(shared_store, action);
      }
      "history" => {
        println!("{:#?}", shared_store.read().await.get_history());
      }
      "add-async" => {
        let action = Action::Mw(Mw::AsyncAddCmd);
        spawn_dispatch_action!(shared_store, action);

        println!("{}", "🧵 Spawning add_async_cmd_mw.rs ...");
      }
      "ip" => {
        let action = Action::Mw(Mw::IpCmd);
        spawn_dispatch_action!(shared_store, action);

        println!("{}", "🧵 Spawning ip_cmd_mw.rs ...");
      }
      "air" => {
        let action = Action::Mw(Mw::AirCmd);
        spawn_dispatch_action!(shared_store, action);

        println!("{}", "🧵 Spawning air_cmd_mw.rs ...");
      }
      "save" => {
        let action = Action::Mw(Mw::SaveCmd);
        spawn_dispatch_action!(shared_store, action);

        println!("{}", "🧵 Spawning save_cmd_mw.rs ...");
      }
      "load" => {
        let action = Action::Mw(Mw::LoadCmd);
        spawn_dispatch_action!(shared_store, action);

        println!("{}", "🧵 Spawning load_cmd_mw.rs ...");
      }
      // Catchall.
      _ => {
        println!("{}", style_error("Unknown command"));
      }
    }; // end match user_input.

    // Print confirmation at the end of 1 repl loop.
    println!(
      "{} {}",
      style_primary(&user_input),
      style_dimmed("was executed.")
    );
  }

  on_end(&mut shared_store.clone()).await;

  Ok(())
}

async fn on_start(shared_store: &SharedStore<State, Action>) {
  print_header("on_start");
  let opt_action = do_load().await;
  if let Some(action) = opt_action {
    shared_store.write().await.dispatch_action(action).await;
  }
}

async fn on_end(shared_store: &SharedStore<State, Action>) {
  print_header("on_end");
  do_save(&shared_store.read().await.get_state()).await;
}
