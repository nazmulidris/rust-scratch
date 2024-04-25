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

use crossterm::style::Stylize;
use kv::*;
use miette::{Context, IntoDiagnostic};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::{collections::HashMap, fmt::Display};

/// Convenience type alias for the [kv::Bucket] type.
/// 1. A [Bucket] is created from a [Store].
/// 2. A [Bucket] is given a name, and there may be many [Bucket]s in a [Store].
/// 3. A [Bucket] provides typed access to a section of the key/value store [kv].
///
/// The [Bucket] stores the following key value pairs.
/// - Key: The generic type `<KT>`.
/// - Value: This type makes it concrete that [Bincode] will be used to serialized the
///   data from the generic type `<VT>`.
pub type KVBucket<'a, KeyT, ValueT> = kv::Bucket<'a, KeyT, Bincode<ValueT>>;

#[derive(Debug, strum_macros::EnumString, Hash, PartialEq, Eq, Clone, Copy)]
pub enum KVSettingsKeys {
    /// Your [Store] folder path name. [kv] uses this folder to save your key/value store. It
    /// is your database persistence folder.
    StoreFolderPath,
    /// Your [Bucket] name that is used to store the [String], [MyValueType] pairs.
    /// - [Bincode] is used to serialize/deserialize the value stored in the key/value pair.
    /// - A [Bucket] provides typed access to a section of the key/value store [kv].
    BucketName,
}
pub static KV_SETTINGS: Lazy<HashMap<KVSettingsKeys, String>> = Lazy::new(|| {
    let mut it = HashMap::new();
    it.insert(KVSettingsKeys::StoreFolderPath, "kv_db_folder".to_string());
    it.insert(KVSettingsKeys::BucketName, "my_data_bucket".to_string());
    it
});

/// Create the db folder if it doesn't exit. Otherwise load it from the folder on disk.
pub fn load_or_create_store(maybe_db_folder_path: Option<&String>) -> miette::Result<Store> {
    // Configure the database folder location.
    let db_folder_path = maybe_db_folder_path
        .cloned()
        .unwrap_or_else(|| "db_folder".to_string());

    let cfg = Config::new(db_folder_path.clone());

    // Open the key/store store using the Config.
    let store = Store::new(cfg)
        .into_diagnostic()
        .wrap_err(KvErrorCouldNot::CreateDbFolder {
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
pub fn load_or_create_bucket_from_store<
    'a,
    KT: kv::Key<'a>,
    VT: Serialize + for<'d> Deserialize<'d>,
>(
    store: &Store,
    maybe_bucket_name: Option<&String>,
) -> miette::Result<KVBucket<'a, KT, VT>> {
    let bucket_name = maybe_bucket_name
        .cloned()
        .unwrap_or_else(|| "data_bucket".to_string());

    let my_payload_bucket: KVBucket<KT, VT> = store
        .bucket(Some(&bucket_name))
        .into_diagnostic()
        .wrap_err(KvErrorCouldNot::CreateBucketFromStore {
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
pub fn save_to_bucket<
    'a,
    KT: kv::Key<'a> + Display,
    VT: Debug + Serialize + for<'d> Deserialize<'d>,
>(
    my_payload_bucket: &'a KVBucket<'a, KT, VT>,
    key: KT,
    value: VT,
) -> miette::Result<()> {
    let value_str = format!("{:?}", value).bold().cyan();

    // Serialize the Rust struct into a binary payload.
    my_payload_bucket
        .set(&key, &Bincode(value))
        .into_diagnostic()
        .wrap_err(KvErrorCouldNot::SaveKeyValuePairToBucket)?;

    tracing::info!(
        "🔽 {}",
        format!(
            "{}: {}: {}",
            "Save key / value pair to bucket".red(),
            key.to_string().bold().cyan(),
            value_str
        )
    );

    Ok(())
}

/// The value in the key/value store is serialized using [Bincode]. Upon loading that
/// value it is deserialized and returned by this function.
pub fn load_from_bucket<
    'a,
    KeyT: kv::Key<'a> + Display,
    ValueT: Debug + Serialize + for<'d> Deserialize<'d>,
>(
    my_payload_bucket: &KVBucket<'a, KeyT, ValueT>,
    key: KeyT,
) -> miette::Result<Option<ValueT>> {
    let maybe_value: Option<Bincode<ValueT>> = my_payload_bucket
        .get(&key)
        .into_diagnostic()
        .wrap_err(KvErrorCouldNot::LoadKeyValuePairFromBucket)?;

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
            key.to_string().bold().cyan(),
            format!("{:?}", it).bold().cyan()
        )
    );

    it
}

mod kv_error {
    #[allow(dead_code)]
    #[derive(thiserror::Error, Debug, miette::Diagnostic)]
    pub enum KvErrorCouldNot {
        #[error("📑 Could not create db folder: '{db_folder_path}' on disk")]
        CreateDbFolder { db_folder_path: String },

        #[error("📦 Could not create bucket from store: '{bucket_name}'")]
        CreateBucketFromStore { bucket_name: String },

        #[error("🔽 Could not save key/value pair to bucket")]
        SaveKeyValuePairToBucket,

        #[error("🔼 Could not load key/value pair from bucket")]
        LoadKeyValuePairFromBucket,

        #[error("🔍 Could not get item from iterator from bucket")]
        GetItemFromIteratorFromBucket,

        #[error("🔍 Could not get key from item from iterator from bucket")]
        GetKeyFromItemFromIteratorFromBucket,

        #[error("🔍 Could not get value from item from iterator from bucket")]
        GetValueFromItemFromIteratorFromBucket,

        #[error("⚡ Could not execute transaction")]
        ExecuteTransaction,
    }
}
use kv_error::*;

#[cfg(test)]
mod kv_tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn check_folder_exists(path: &Path) -> bool {
        path.exists() && path.is_dir()
    }

    fn join_path_with_str(path: &Path, str: &str) -> PathBuf {
        path.join(str)
    }

    fn perform_db_operations(path: &Path, bucket: String) -> miette::Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let my_store = load_or_create_store(Some(&path_str))?;

        // Check that the kv store folder exists.
        assert!(check_folder_exists(path));

        // A bucket provides typed access to a section of the key/value store.
        let my_payload_bucket: KVBucket<String, String> =
            load_or_create_bucket_from_store(&my_store, Some(&bucket))?;

        // Save to bucket.
        save_to_bucket(&my_payload_bucket, "foo".to_string(), "bar".to_string())?;

        // Load from bucket.
        assert_eq!(
            load_from_bucket(&my_payload_bucket, "foo".to_string())?,
            Some("bar".to_string())
        );

        // Enumerate contents of bucket.
        let mut map = HashMap::new();
        for result_item in my_payload_bucket.iter() {
            let item = result_item
                .into_diagnostic()
                .wrap_err(KvErrorCouldNot::GetItemFromIteratorFromBucket)?;

            let key = item
                .key::<String>()
                .into_diagnostic()
                .wrap_err(KvErrorCouldNot::GetKeyFromItemFromIteratorFromBucket)?;

            // Deserialize the binary payload into a Rust struct.
            let Bincode(payload) = item
                .value::<Bincode<String>>()
                .into_diagnostic()
                .wrap_err(KvErrorCouldNot::GetValueFromItemFromIteratorFromBucket)?;

            map.insert(key.to_string(), payload);
        }

        assert_eq!(map.get("foo"), Some(&"bar".to_string()));

        Ok(())
    }

    #[test]
    fn test_kv_operations() {
        // The `dir` will be automatically deleted when it goes out of scope.
        let dir = tempdir().expect("Failed to create temp dir");
        // You can use `path` here for your operations.
        let path = dir.path();
        // Run the tests.
        perform_db_operations(
            join_path_with_str(path, "db_folder").as_path(),
            "bucket".to_string(),
        )
        .unwrap();
    }
}
