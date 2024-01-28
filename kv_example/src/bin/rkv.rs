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
//! - More about `rkv` crate: <https://docs.rs/rkv/latest/rkv/>
//! - More about `bincode` crate: <https://docs.rs/bincode/latest/bincode/>
//!
//! Create a key value store that stores keys that are type [String], and values that are
//! type [MyValueType].
//!
//! This uses a [SafeMode] store and not [LMDB](http://www.lmdb.tech/doc/starting.html),
//! since the current support for LMDB is limited. The [SafeMode] backend performs well,
//! with two caveats: the entire database is stored in memory, and write transactions are
//! synchronously written to disk (only on commit).
//! - LMDB supports concurrent access by [many
//!   processes](https://g.co/bard/share/a9a5d101182f) and employs [POSIX file
//!   locking](https://g.co/bard/share/84df29aae2de) to do this. Note that SQLite also
//!   supports access from [multiple processes](https://g.co/bard/share/7b812f2aa616).
//! - The [rkv::backend::BackendEnvironment] is a wrapper around a variety of different
//!   stores (like [SafeMode] and [LMDB]), and it is used to create a [rkv::Rkv] instance.
//!
//! The [Manager] enforces that each process opens the same environment at most once by
//! caching a handle to each environment that it opens. Use it to retrieve the handle to
//! an opened environment—or create one if it hasn't already been opened.
//!
//! In order to perform CRUD operations on the database backend, both the following are
//! needed:
//! 1) environment: [rkv::Rkv<<SafeModeEnvironment>>].
//! 2) store: [SingleStore<SafeModeDatabase>]. A [SingleStore] allows key value pairs
//!    where there may only be 1 value for a key to be stored.
//!
//! The architecture of this crate is very different from [kv]. Lambda functions are used
//! to perform operations on the database. This is due to the fact that the [Manager] is
//! used to cache the handle to the environment, and the [Manager] is a singleton. This
//! means that the [Manager] is a shared resource, and it is not possible to have multiple
//! [Manager]s in the same process. This is why the [Manager] is wrapped in a
//! [std::sync::RwLock] to allow for concurrent access to the [Manager] from multiple
//! threads.
//!
//! Also [rkv::Reader]s and [rkv::Writer]s work closely with each other.
//! - No readers or writers are allowed, once one writer is active.
//! - Use a write transaction to mutate the store via a [rkv::Writer]. There can be only
//!   one writer for a given environment, so opening a second one will block until the
//!   first completes.
//! - Use a read transaction to query the store via a [rkv::Reader]. There can be multiple
//!   concurrent readers for a store, and readers never block on a writer nor other
//!   readers.
//!
//! [bincode] is a crate for encoding and decoding using a tiny binary serialization
//! strategy. Using it, you can easily go from having an object in memory, quickly
//! serialize it to bytes, and then deserialize it back just as fast!

use crossterm::style::Stylize;
use miette::{Context, IntoDiagnostic};
use rkv::backend::{SafeMode, SafeModeDatabase, SafeModeEnvironment};
use rkv::{Manager, Rkv, SingleStore, StoreOptions, Value};
use std::{fs, path::Path, sync::RwLockReadGuard};
use std::{thread::sleep, time::Duration};

use kv_example::{random_number, MyKeyType, MyValueType};

const MY_DB_FOLDER: &str = "rkv_db_folder";
const MY_PAYLOAD_STORE_NAME: &str = "my_payload_bucket";

fn main() -> miette::Result<()> {
    let mut max_count = 5;
    loop {
        sleep(Duration::from_secs(1));
        println!("---------------------------------");
        perform_db_operations()?;
        max_count -= 1;
        if max_count == 0 {
            break;
        }
    }
    Ok(())
}

fn perform_db_operations() -> miette::Result<()> {
    {
        let key = format!("key_{}", random_number::<u8>());

        run_lambda_on_db_store(
            MY_DB_FOLDER,
            MY_PAYLOAD_STORE_NAME,
            |single_store, environment| {
                let value = MyValueType {
                    id: random_number::<f32>(),
                    description: "first item".into(),
                    data: vec![random_number::<u8>(), random_number::<u8>()],
                };
                write_op(single_store, environment, key.clone(), value.clone())?;
                println!(
                    "{}: key: {}, value: {}",
                    "write_op".red(),
                    key.clone().dark_magenta(),
                    format!("{:?}", value).magenta()
                );
                Ok(())
            },
        )?;

        run_lambda_on_db_store(
            MY_DB_FOLDER,
            MY_PAYLOAD_STORE_NAME,
            |single_store, environment| {
                let maybe_value = read_op(single_store, environment, key.clone())?;
                if let Some(value) = maybe_value {
                    println!(
                        "{}: key: {}, value: {}",
                        "read_op".green(),
                        key.clone().dark_blue(),
                        format!("{:?}", value).blue()
                    )
                }
                Ok(())
            },
        )?;

        run_lambda_on_db_store(
            MY_DB_FOLDER,
            MY_PAYLOAD_STORE_NAME,
            |single_store, environment| {
                delete_op(single_store, environment, key.clone())?;
                println!("{}: key: {}", "delete_op".red(), key.clone().dark_blue());
                Ok(())
            },
        )?;

        sleep(Duration::from_secs(1));

        run_lambda_on_db_store(
            MY_DB_FOLDER,
            MY_PAYLOAD_STORE_NAME,
            |single_store, environment| {
                iter_op(single_store, environment, |key, value| {
                    println!(
                        "{}: key: {}, value: {}",
                        "iter_op".cyan(),
                        key.clone().dark_blue(),
                        format!("{:?}", value).blue()
                    );
                    Ok(())
                })
            },
        )?;

        run_lambda_on_db_store(
            MY_DB_FOLDER,
            MY_PAYLOAD_STORE_NAME,
            |single_store, environment| {
                clear_op(single_store, environment)?;
                println!("{}", "clear_op".red());
                Ok(())
            },
        )?;
    }

    Ok(())
}

pub fn iter_op<F>(
    store: SingleStore<SafeModeDatabase>,
    environment: RwLockReadGuard<'_, Rkv<SafeModeEnvironment>>,
    mut functor: F,
) -> miette::Result<()>
where
    F: FnMut(/* key */ MyKeyType, /* value */ MyValueType) -> miette::Result<()>,
{
    let reader = environment
        .read()
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotGetReaderFromEnvironment)?;

    let iter = store
        .iter_start(&reader)
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotGetIteratorFromStore)?;

    for it in iter {
        if let Ok((key, value)) = it {
            match (String::from_utf8(key.to_vec()), value) {
                (Ok(key), Value::Blob(bytes)) => {
                    let value = bincode::deserialize::<MyValueType>(&bytes)
                        .into_diagnostic()
                        .wrap_err(CustomError::CouldNotDeserializeValue)?;
                    functor(key, value)?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub fn clear_op(
    store: SingleStore<SafeModeDatabase>,
    environment: RwLockReadGuard<'_, Rkv<SafeModeEnvironment>>,
) -> miette::Result<()> {
    // Use a write transaction to mutate the store via a `Writer`. There can be only
    // one writer for a given environment, so opening a second one will block until
    // the first completes.
    let mut writer = environment
        .write()
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotGetWriterFromEnvironment)?;
    store
        .clear(&mut writer)
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotRunClearOperationWithWriter)?;
    writer
        .commit()
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotCommitTransaction)?;

    Ok(())
}

pub fn delete_op(
    store: SingleStore<SafeModeDatabase>,
    environment: RwLockReadGuard<'_, Rkv<SafeModeEnvironment>>,
    key: MyKeyType,
) -> miette::Result<()> {
    // Use a write transaction to mutate the store via a `Writer`. There can be only
    // one writer for a given environment, so opening a second one will block until
    // the first completes.
    let mut writer = environment
        .write()
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotGetWriterFromEnvironment)?;
    store
        .delete(&mut writer, key)
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotRunDeleteOperationWithWriter)?;
    writer
        .commit()
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotCommitTransaction)?;

    Ok(())
}

pub fn write_op(
    store: SingleStore<SafeModeDatabase>,
    environment: RwLockReadGuard<'_, Rkv<SafeModeEnvironment>>,
    key: MyKeyType,
    value: MyValueType,
) -> miette::Result<()> {
    // Use a write transaction to mutate the store via a `Writer`. There can be only
    // one writer for a given environment, so opening a second one will block until
    // the first completes.
    let mut writer = environment
        .write()
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotGetWriterFromEnvironment)?;

    let bytes: Vec<u8> = bincode::serialize(&value)
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotSerializeValue)?;

    store
        .put(&mut writer, key, &Value::Blob(&bytes))
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotRunPutOperationWithWriter)?;

    // You must commit a write transaction before the writer goes out of scope, or the
    // transaction will abort and the data won't persist.
    writer
        .commit()
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotCommitTransaction)?;

    Ok(())
}

pub fn read_op(
    store: SingleStore<SafeModeDatabase>,
    environment: RwLockReadGuard<'_, Rkv<SafeModeEnvironment>>,
    key: MyKeyType,
) -> miette::Result<Option<MyValueType>> {
    // Use a read transaction to query the store via a `Reader`. There can be multiple
    // concurrent readers for a store, and readers never block on a writer nor other
    // readers.
    let reader = environment
        .read()
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotGetReaderFromEnvironment)?;

    let maybe_blob = store
        .get(&reader, key)
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotRunGetOperationWithReader)?;

    if let Some(Value::Blob(bytes)) = maybe_blob {
        let my_value: MyValueType = bincode::deserialize(bytes)
            .into_diagnostic()
            .wrap_err(CustomError::CouldNotDeserializeValue)?;
        return Ok(Some(my_value));
    }

    // A read transaction will automatically close once the reader goes out of scope,
    // so isn't necessary to close it explicitly, although you can do so by calling
    // `Reader.abort()`.
    Ok(None)
}

/// [FnMut] info: <https://g.co/bard/share/2f53e5a2e611>
pub fn run_lambda_on_db_store<'a, F>(
    db_folder_path_str: &'a str,
    store_name: &'a str,
    mut lambda: F,
) -> miette::Result<()>
where
    F: FnMut(
        /* store */ SingleStore<SafeModeDatabase>,
        /* environment */ RwLockReadGuard<'_, Rkv<SafeModeEnvironment>>,
    ) -> miette::Result<()>,
{
    // First determine the path to the environment, which is represented on disk as a
    // directory containing two files:
    //   * a data file containing the key/value stores.
    //   * a lock file containing metadata about current transactions.
    let db_folder_path = Path::new(db_folder_path_str);
    fs::create_dir_all(db_folder_path)
        .into_diagnostic()
        .wrap_err(CustomError::CouldNotCreateDbFolder {
            db_folder_path: db_folder_path_str.to_string(),
        })?;

    // The `Manager` enforces that each process opens the same environment at most once by
    // caching a handle to each environment that it opens. Use it to retrieve the handle
    // to an opened environment—or create one if it hasn't already been opened.
    match Manager::<SafeModeEnvironment>::singleton().write() {
        Ok(mut manager) => {
            match manager
                .get_or_create(db_folder_path, Rkv::new::<SafeMode>)
                .into_diagnostic()
                .wrap_err(CustomError::CouldNotGetManager)?
                .read()
            {
                Ok(environment) => {
                    // Then you can use the environment handle to get a handle to a datastore.
                    let store = environment
                        .open_single(store_name, StoreOptions::create())
                        .into_diagnostic()?;
                    let _ = lambda(store, environment);
                }
                _ => {}
            }
        }
        Err(_) => {
            return Err(CustomError::CouldNotGetManager).into_diagnostic();
        }
    }

    Ok(())
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum CustomError {
    #[diagnostic(code(rkv::DatabaseError::FileSystemError))]
    #[error("📂 Could not create db folder: '{db_folder_path}' on disk")]
    CouldNotCreateDbFolder { db_folder_path: String },

    #[diagnostic(code(rkv::DatabaseError::StoreCreationOrAccessError))]
    #[error("👨‍💼 Could not get a manager")]
    CouldNotGetManager,

    #[diagnostic(code(rkv::DatabaseError::StoreCreationOrAccessError))]
    #[error("💾 Could not get or create environment, or open store")]
    CouldNotGetOrCreateEnvironmentOrOpenStore { store_name: String },

    #[diagnostic(code(rkv::DatabaseError::CouldNotGetReaderFromEnvironment))]
    #[error("💾 Could not create a read transaction from the environment")]
    CouldNotGetReaderFromEnvironment,

    #[diagnostic(code(rkv::DatabaseError::CouldNotRunGetOperationWithReader))]
    #[error("💾 Could perform get operation with reader transaction")]
    CouldNotRunGetOperationWithReader,

    #[diagnostic(code(rkv::DatabaseError::CouldNotDeserializeValue))]
    #[error("💾 Could not deserialize value")]
    CouldNotDeserializeValue,

    #[diagnostic(code(rkv::DatabaseError::CouldNotGetWriterFromEnvironment))]
    #[error("💾 Could not create a write transaction from the environment")]
    CouldNotGetWriterFromEnvironment,

    #[diagnostic(code(rkv::DatabaseError::CouldNotSerializeValue))]
    #[error("💾 Could not serialize value")]
    CouldNotSerializeValue,

    #[diagnostic(code(rkv::DatabaseError::CouldNotRunPutOperationWithWriter))]
    #[error("💾 Could perform put operation with writer transaction")]
    CouldNotRunPutOperationWithWriter,

    #[diagnostic(code(rkv::DatabaseError::CouldNotCommitTransaction))]
    #[error("💾 Could not commit transaction")]
    CouldNotCommitTransaction,

    #[diagnostic(code(rkv::DatabaseError::CouldNotRunDeleteOperationWithWriter))]
    #[error("💾 Could perform delete operation with writer transaction")]
    CouldNotRunDeleteOperationWithWriter,

    #[diagnostic(code(rkv::DatabaseError::CouldNotRunClearOperationWithWriter))]
    #[error("💾 Could perform clear operation with writer transaction")]
    CouldNotRunClearOperationWithWriter,

    #[diagnostic(code(rkv::DatabaseError::CouldNotGetIteratorFromStore))]
    #[error("💾 Could not get iterator from store")]
    CouldNotGetIteratorFromStore,
}
