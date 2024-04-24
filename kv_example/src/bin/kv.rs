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

//! This sample binary uses the [kv] crate.
//!
//! Create a key value store that stores keys that are type [String], and values that are
//! type [MyValueType].
//!
//! 1. The values are serialized to [Bincode] (from Rust struct) before they are saved.
//! 2. The values are deserialized from [Bincode] (to Rust struct) after they are loaded.
//!
//! [Bincode] is like `CBOR`, except that it isn't standards based, but it is faster. It
//! also has full support of [serde] just like [kv] does.
//! - [More info comparing `CBOR` with `Bincode`](https://gemini.google.com/share/0684553f3d57)
//!
//! This crate works really well, even with multiple processes accessing the same database
//! on disk. You can run `cargo run --bin kv` a few times, and it works as expected. Even
//! with multiple processes writing to the kv store, the iterator can be used to read the
//! current state of the db, as expected. This is unlike the [rkv] crate.

use std::{thread::sleep, time::Duration};

use crossterm::style::Stylize;
use kv::*;
use kv_example::{random_number, MyKeyType, MyValueType};
use miette::{Context, IntoDiagnostic};

/// Convenience type alias for the [kv::Bucket] type.
/// 1. A [Bucket] is created from a [Store].
/// 2. A [Bucket] is given a name, and there may be many [Bucket]s in a [Store].
/// 3. A [Bucket] provides typed access to a section of the key/value store [kv].
///
/// The [Bucket] stores the following key value pairs.
/// - Key: The generic type `<KT>`.
/// - Value: This type makes it concrete that [Bincode] will be used to serialized the
///   data from the generic type `<VT>`.
type MyBucket<'a, KT, VT> = kv::Bucket<'a, KT, Bincode<VT>>;

/// Your [Store] folder path name. [kv] uses this folder to save your key/value store. It
/// is your database persistence folder.
const MY_DB_FOLDER: &str = "kv_db_folder";
/// Your [Bucket] name that is used to store the [String], [MyValueType] pairs.
/// - [Bincode] is used to serialize/deserialize the value stored in the key/value pair.
/// - A [Bucket] provides typed access to a section of the key/value store [kv].
const MY_PAYLOAD_BUCKET_NAME: &str = "my_payload_bucket";

/// Convenience type alias for [std::result::Result].
type MyResult<T> = miette::Result<T>;

fn main() -> MyResult<()> {
    // Setup tracing. More info: <https://tokio.rs/tokio/topics/tracing>
    tracing::subscriber::set_global_default(
        // More info: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#configuration>
        tracing_subscriber::fmt()
            // .pretty() /* multi line pretty output*/
            .compact() /* single line output */
            .without_time() /* don't display time in output */
            .with_thread_ids(true)
            .with_ansi(true)
            .with_target(false)
            .with_file(false)
            .with_line_number(false)
            .finish(),
    )
    .into_diagnostic()?;

    let mut max_count = 3;
    loop {
        sleep(Duration::from_secs(3));
        println!("---------------------------------");
        perform_db_operations()?;
        max_count -= 1;
        if max_count == 0 {
            break;
        }
    }
    Ok(())
}

fn perform_db_operations() -> MyResult<()> {
    let my_store = load_or_create_store(MY_DB_FOLDER.to_string())?;

    // A bucket provides typed access to a section of the key/value store.
    let my_payload_bucket: MyBucket<MyKeyType, MyValueType> =
        load_or_create_bucket_from_store(&my_store, MY_PAYLOAD_BUCKET_NAME.to_string())?;

    // Save to bucket.
    let key = format!("key_{}", random_number::<u8>());
    save_to_bucket(
        &my_payload_bucket,
        key.clone(),
        MyValueType {
            id: random_number::<f32>(),
            description: "first item".into(),
            data: vec![random_number::<u8>(), random_number::<u8>()],
        },
    )?;

    // Load from bucket.
    let _ = load_from_bucket(&my_payload_bucket, key)?;

    run_txn(&my_payload_bucket)?;

    // Enumerate contents of bucket.
    for (index, result_item) in my_payload_bucket.iter().enumerate() {
        let item = result_item
            .into_diagnostic()
            .wrap_err(KvError::CouldNotGetItemFromIteratorFromBucket)?;

        let key = item
            .key::<String>()
            .into_diagnostic()
            .wrap_err(KvError::CouldNotGetKeyFromItemFromIteratorFromBucket)?;

        // Deserialize the binary payload into a Rust struct.
        let Bincode(payload) = item
            .value::<Bincode<MyValueType>>()
            .into_diagnostic()
            .wrap_err(KvError::CouldNotGetValueFromItemFromIteratorFromBucket)?;

        println!(
            "[{}]: {} => {}",
            format!("{}", index).magenta(),
            format!("{}", key).yellow(),
            format!("{:?}", payload).cyan()
        )
    }

    Ok(())
}

/// Create the db folder if it doesn't exit. Otherwise load it from the folder on disk.
pub fn load_or_create_store(db_folder_path: String) -> MyResult<Store> {
    // Configure the database folder location.
    let cfg = Config::new(db_folder_path.clone());

    // Open the key/store store using the Config.
    let store = Store::new(cfg)
        .into_diagnostic()
        .wrap_err(KvError::CouldNotCreateDbFolder {
            db_folder_path: db_folder_path.clone(),
        })?;

    tracing::info!(
        "📑 {}",
        format!(
            "{}{}",
            "load or create a store: ".blue(),
            db_folder_path.bold().cyan()
        )
    );

    Ok(store)
}

/// A [Bucket] provides typed access to a section of the key/value store [kv].
pub fn load_or_create_bucket_from_store<'a>(
    store: &Store,
    bucket_name: String,
) -> MyResult<MyBucket<'a, String, MyValueType>> {
    let my_payload_bucket: MyBucket<MyKeyType, MyValueType> = store
        .bucket(Some(&bucket_name))
        .into_diagnostic()
        .wrap_err(KvError::CouldNotCreateBucketFromStore {
            bucket_name: bucket_name.clone(),
        })?;

    tracing::info!(
        "📦 {}",
        format!(
            "{}{}",
            "Load or create bucket from store, and instantiate: ".blue(),
            bucket_name.bold().cyan()
        )
    );

    Ok(my_payload_bucket)
}

/// The value is serialized using [Bincode] prior to saving it to the key/value store.
pub fn save_to_bucket(
    my_payload_bucket: &MyBucket<MyKeyType, MyValueType>,
    key: String,
    value: MyValueType,
) -> MyResult<()> {
    let value_str = format!("{:?}", value).bold().cyan();

    // Serialize the Rust struct into a binary payload.
    my_payload_bucket
        .set(&key, &Bincode(value))
        .into_diagnostic()
        .wrap_err(KvError::CouldNotSaveKeyValuePairToBucket)?;

    tracing::info!(
        "🔽 {}",
        format!(
            "{}: {}: {}",
            "Save key / value pair to bucket".red(),
            key.bold().cyan(),
            value_str
        )
    );

    Ok(())
}

/// The value in the key/value store is serialized using [Bincode]. Upon loading that
/// value it is deserialized and returned by this function.
pub fn load_from_bucket(
    my_payload_bucket: &MyBucket<MyKeyType, MyValueType>,
    key: String,
) -> MyResult<Option<MyValueType>> {
    let maybe_value: Option<Bincode<MyValueType>> =
        my_payload_bucket
            .get(&key)
            .into_diagnostic()
            .wrap_err(KvError::CouldNotLoadKeyValuePairFromBucket)?;

    let it = match maybe_value {
        // Deserialize the binary payload into a Rust struct.
        Some(Bincode(payload)) => Ok(Some(payload)),
        _ => Ok(None),
    };

    tracing::info!(
        "🔼 {}",
        format!(
            "{}: {}: {}",
            "Load key / value pair from bucket".green(),
            key.bold().cyan(),
            format!("{:?}", it).bold().cyan()
        )
    );

    it
}

pub fn run_txn(my_payload_bucket: &MyBucket<MyKeyType, MyValueType>) -> MyResult<()> {
    let rand_u8 = random_number::<u8>();
    let rand_f32_1 = (rand_u8 as f32) + 0.1f32;
    let rand_f32_2 = (rand_u8 as f32) + 0.2f32;
    let result_txn = my_payload_bucket.transaction(|txn| {
        let key_1 = format!("key_{}.1", rand_u8).to_string();
        txn.set(
            &key_1,
            &Bincode(MyValueType {
                id: rand_f32_1,
                description: "txn #1".into(),
                data: vec![31],
            }),
        )?;

        let key_2 = format!("key_{}.2", rand_u8).to_string();
        txn.set(
            &key_2,
            &Bincode(MyValueType {
                id: rand_f32_2,
                description: "txn #2".into(),
                data: vec![32],
            }),
        )?;

        println!(
            "⚡ {}, keys: [{}, {}]",
            "Saving 2 items in 1 transaction".red(),
            key_1.bold().yellow(),
            key_2.bold().yellow()
        );

        Ok(())
    });

    Ok(result_txn
        .into_diagnostic()
        .wrap_err(KvError::CouldNotExecuteTransaction)?)
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum KvError {
    #[error("📑 Could not create db folder: '{db_folder_path}' on disk")]
    CouldNotCreateDbFolder { db_folder_path: String },

    #[error("📦 Could not create bucket from store: '{bucket_name}'")]
    CouldNotCreateBucketFromStore { bucket_name: String },

    #[error("🔽 Could not save key/value pair to bucket")]
    CouldNotSaveKeyValuePairToBucket,

    #[error("🔼 Could not load key/value pair from bucket")]
    CouldNotLoadKeyValuePairFromBucket,

    #[error("🔍 Could not get item from iterator from bucket")]
    CouldNotGetItemFromIteratorFromBucket,

    #[error("🔍 Could not get key from item from iterator from bucket")]
    CouldNotGetKeyFromItemFromIteratorFromBucket,

    #[error("🔍 Could not get value from item from iterator from bucket")]
    CouldNotGetValueFromItemFromIteratorFromBucket,

    #[error("⚡ Could not execute transaction")]
    CouldNotExecuteTransaction,
}
