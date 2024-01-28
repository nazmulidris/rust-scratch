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
//! - [More info comparing `CBOR` with `Bincode`](https://g.co/bard/share/0684553f3d57)

use std::{thread::sleep, time::Duration};

use crossterm::style::Stylize;
use kv::*;
use kv_example::{random_number, MyKeyType, MyValueType};

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
type MyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

fn perform_db_operations() -> MyResult<()> {
    let store = get_store_by_creating_or_loading_db(MY_DB_FOLDER.into())?;

    // A bucket provides typed access to a section of the key/value store.
    let my_payload_bucket: MyBucket<MyKeyType, MyValueType> =
        create_or_load_bucket_from_store(&store, MY_PAYLOAD_BUCKET_NAME.into())?;
    println!(
        "{}: {}",
        "Load or create db, and instantiate".blue(),
        MY_PAYLOAD_BUCKET_NAME.bold().cyan()
    );

    // Save to bucket.
    save_to_bucket(
        &my_payload_bucket,
        format!("key_{}", random_number::<u8>()),
        MyValueType {
            id: random_number::<f32>(),
            description: "first item".into(),
            data: vec![random_number::<u8>(), random_number::<u8>()],
        },
    )?;
    println!("{}", "Save to bucket: key_1".red());

    save_to_bucket(
        &my_payload_bucket,
        format!("key_{}", random_number::<u8>()),
        MyValueType {
            id: random_number::<f32>(),
            description: "second item".into(),
            data: vec![random_number::<u8>(), random_number::<u8>()],
        },
    )?;
    println!("{}", "Save to bucket: key_2".red());

    // Load from bucket.
    if let Some(payload) = load_from_bucket(&my_payload_bucket, "key_1".into())? {
        println!(
            "{} : {}",
            "load from bucket => value of key_1".green(),
            format!("{:?}", payload).cyan()
        );
    };

    if let Some(payload) = load_from_bucket(&my_payload_bucket, "key_2".into())? {
        println!(
            "{} : {}",
            "load from bucket => value of key_2".green(),
            format!("{:?}", payload).cyan()
        );
    };

    run_txn(&my_payload_bucket)?;

    // Enumerate contents of bucket.
    for (index, result_item) in my_payload_bucket.iter().enumerate() {
        let item = result_item?;
        let key = item.key::<String>()?;
        // Deserialize the binary payload into a Rust struct.
        let Bincode(payload) = item.value::<Bincode<MyValueType>>()?;
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
pub fn get_store_by_creating_or_loading_db(path: String) -> MyResult<Store> {
    // Configure the database folder location.
    let cfg = Config::new(path);

    // Open the key/store store using the Config.
    let store = Store::new(cfg)?;

    Ok(store)
}

/// A [Bucket] provides typed access to a section of the key/value store [kv].
pub fn create_or_load_bucket_from_store<'a>(
    store: &Store,
    bucket_name: &str,
) -> MyResult<MyBucket<'a, String, MyValueType>> {
    let my_payload_bucket: MyBucket<MyKeyType, MyValueType> = store.bucket(Some(bucket_name))?;
    Ok(my_payload_bucket)
}

/// The value in the key/value store is serialized using [Bincode]. Upon loading that
/// value it is deserialized and returned by this function.
pub fn load_from_bucket(
    my_payload_bucket: &MyBucket<MyKeyType, MyValueType>,
    key: String,
) -> MyResult<Option<MyValueType>> {
    let maybe_value: Option<Bincode<MyValueType>> = my_payload_bucket.get(&key)?;
    match maybe_value {
        // Deserialize the binary payload into a Rust struct.
        Some(Bincode(payload)) => Ok(Some(payload)),
        _ => Ok(None),
    }
}

/// The value is serialized using [Bincode] prior to saving it to the key/value store.
pub fn save_to_bucket(
    my_payload_bucket: &MyBucket<MyKeyType, MyValueType>,
    key: String,
    value: MyValueType,
) -> MyResult<()> {
    // Serialize the Rust struct into a binary payload.
    my_payload_bucket.set(&key, &Bincode(value))?;
    Ok(())
}

pub fn run_txn(my_payload_bucket: &MyBucket<MyKeyType, MyValueType>) -> MyResult<()> {
    let rand_u8 = random_number::<u8>();
    let rand_f32_1 = (rand_u8 as f32) + 0.1f32;
    let rand_f32_2 = (rand_u8 as f32) + 0.2f32;
    let result_txn = my_payload_bucket.transaction(|txn| {
        txn.set(
            &format!("key_{}.1", rand_u8).into(),
            &Bincode(MyValueType {
                id: rand_f32_1,
                description: "txn #1".into(),
                data: vec![31],
            }),
        )?;
        txn.set(
            &format!("key_{}.2", rand_u8).into(),
            &Bincode(MyValueType {
                id: rand_f32_2,
                description: "txn #2".into(),
                data: vec![32],
            }),
        )?;
        println!("{}", "Saving 2 items in 1 transaction".red());
        Ok(())
    });
    Ok(result_txn?)
}
