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

//! This sample binary uses the [rkv] crate.
//!
//! Create a key value store that stores keys that are type [String], and values that are
//! type [MyValueType].
//!
//! This uses a [SafeMode] store and not [LMDB](http://www.lmdb.tech/doc/starting.html),
//! since the current support for LMDB is limited. The [SafeMode] backend performs well,
//! with two caveats: the entire database is stored in memory, and write transactions are
//! synchronously written to disk (only on commit). LMDB supports concurrent access by
//! [many processes](https://g.co/bard/share/a9a5d101182f) and employs [POSIX file
//! locking](https://g.co/bard/share/84df29aae2de) to do this. Note that SQLite also
//! supports access from [multiple processes](https://g.co/bard/share/7b812f2aa616).
//!
//! The [Manager] enforces that each process opens the same environment at most once by
//! caching a handle to each environment that it opens. Use it to retrieve the handle to
//! an opened environment—or create one if it hasn't already been opened.

use rkv::backend::{BackendEnvironment, SafeMode, SafeModeDatabase, SafeModeEnvironment};
use rkv::{Manager, Rkv, SingleStore, StoreOptions, Value};
use std::{
    fs,
    io::{Error, ErrorKind},
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use std::{thread::sleep, time::Duration};

use kv_example::{random_number, MyKeyType, MyResult, MyValueType};

const MY_DB_FOLDER: &str = "rkv_db_folder";
const MY_PAYLOAD_STORE_NAME: &str = "my_payload_bucket";

fn main() -> MyResult<()> {
    let mut max_count = 5;
    loop {
        sleep(Duration::from_secs(5));
        println!("---------------------------------");
        perform_db_operations()?;
        max_count -= 1;
        if max_count == 0 {
            break;
        }
    }
    Ok(())
}

// fn perform_db_operations_2() -> MyResult<()> {
//     let db_folder_path = Path::new(MY_DB_FOLDER);

//     let mut manager = Manager::<SafeModeEnvironment>::singleton().write()?;

//     let arc_created = manager.get_or_create(db_folder_path, Rkv::new::<SafeMode>)?;

//     let result_env = arc_created.read();
//     match result_env {
//         Ok(env) => {
//             let store = env.open_single(MY_PAYLOAD_STORE_NAME, StoreOptions::create())?;
//             Ok(())
//         }
//         Err(_) => Err(Box::new(Error::new(ErrorKind::NotFound, "!"))),
//     }
// }

fn perform_db_operations() -> MyResult<()> {
    run_functor_on_db_store(MY_DB_FOLDER, MY_PAYLOAD_STORE_NAME, my_db_functor)?;
    Ok(())
}

pub fn run_functor_on_db_store<'a>(
    db_folder_path: &'a str,
    store_name: &'a str,
    db_functor: DbFunctor,
) -> MyResult<()> {
    // First determine the path to the environment, which is represented on disk as a
    // directory containing two files:
    //   * a data file containing the key/value stores
    //   * a lock file containing metadata about current transactions
    let db_folder_path = Path::new(db_folder_path);
    fs::create_dir_all(db_folder_path)?;

    let mut manager = Manager::<SafeModeEnvironment>::singleton().write()?;

    let arc_created = manager.get_or_create(db_folder_path, Rkv::new::<SafeMode>)?;

    let result_env = arc_created.read();
    match result_env {
        Ok(env) => {
            let store = env.open_single(store_name, StoreOptions::create())?;
            db_functor(store, env)
        }
        Err(_) => Err(Box::new(Error::new(
            ErrorKind::NotFound,
            format!(
                "Could not get or create store in path {}!",
                db_folder_path.display()
            ),
        ))),
    }
}

pub type DbFunctor = fn(
    store: SingleStore<SafeModeDatabase>,
    env: RwLockReadGuard<'_, Rkv<SafeModeEnvironment>>,
) -> MyResult<()>;

fn my_db_functor(
    store: SingleStore<SafeModeDatabase>,
    env: RwLockReadGuard<'_, Rkv<SafeModeEnvironment>>,
) -> MyResult<()> {
    Ok(())
}
