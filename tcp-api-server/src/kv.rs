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

//! Use module is standalone, you can use it in any project that needs to create an
//! embedded key/value store that stores keys that are of whatever type you choose, and
//! values that are whatever type you choose.
//! - It is a wrapper around the [kv] crate, to make it trivially simple to use. There are
//!   only 4 functions that allow you access to the capabilities of the key/value embedded
//!   store.
//!   - [load_or_create_store]
//!   - [load_or_create_bucket_from_store]
//!   - [save_to_bucket]
//!   - [load_from_bucket]
//!   - [remove_from_bucket]
//!   - [is_key_contained_in_bucket]
//! - And provide lots of really fine grained errors, using [miette] and [thiserror] (see
//!   [kv_error]).
//!
//! 1. The values are serialized to [Bincode] (from Rust struct) before they are saved.
//! 2. The values are deserialized from [Bincode] (to Rust struct) after they are loaded.
//!
//! See the tests in this module for an example of how to use this module.
//!
//! [Bincode] is like [`CBOR`](https://en.wikipedia.org/wiki/CBOR), except that it isn't
//! standards based, but it is faster. It also has full support of [serde] just like [kv]
//! does.
//! - [More info comparing [`CBOR`](https://en.wikipedia.org/wiki/CBOR) with
//!   [`Bincode`](https://gemini.google.com/share/0684553f3d57)
//!
//! The [kv] crate works really well, even with multiple processes accessing the same
//! database on disk. Even though [sled](https://github.com/spacejam/sled), which the [kv]
//! crate itself wraps, is not multi-process safe.
//!
//! In my testing, I've run multiple processes that write to the key/value store at the
//! same time, and it works as expected. Even with multiple processes writing to the
//! store, the iterator [Bucket::iter] can be used to read the current state of the db, as
//! expected.

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
/// The [Bucket] stores the following key/value pairs.
/// - `KeyT`: The generic type `<KeyT>`. This will not be serialized or deserialized. This
///   also has a trait bound on [Key]. See [save_to_bucket] for an example of this.
/// - `ValueT`: This type makes it concrete that [Bincode] will be used to serialize and
///   deserialize the data from the generic type `<ValueT>`, which has trait bounds on
///   [Serialize], [Deserialize]. See [save_to_bucket] for an example of this.
pub type KVBucket<'a, KeyT, ValueT> = kv::Bucket<'a, KeyT, Bincode<ValueT>>;

#[derive(Debug, strum_macros::EnumString, Hash, PartialEq, Eq, Clone, Copy)]
pub enum KVSettingsKeys {
    /// Your [Store] folder path name. [kv] uses this folder to save your key/value store.
    /// It is your database persistence folder.
    StoreFolderPath,
    /// Your [Bucket] name that is used to store the key/value pairs.
    /// - [Bincode] is used to serialize/deserialize the value stored in the key/value
    ///   pair.
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
/// Note there are no lifetime annotations on this function. All the other functions below
/// do have lifetime annotations, since they are all tied to the lifetime of the returned
/// [Store].
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

/// A [Bucket] provides typed access to a section of the key/value [Store]. It has a
/// lifetime, since the [Bucket] is created from a [Store].
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

pub fn remove_from_bucket<
    'a,
    KeyT: kv::Key<'a> + Display,
    ValueT: Debug + Serialize + for<'d> Deserialize<'d>,
>(
    my_payload_bucket: &KVBucket<'a, KeyT, ValueT>,
    key: KeyT,
) -> miette::Result<Option<ValueT>> {
    let maybe_value: Option<Bincode<ValueT>> = my_payload_bucket
        .remove(&key)
        .into_diagnostic()
        .wrap_err(KvErrorCouldNot::RemoveKeyValuePairFromBucket)?;

    let it = match maybe_value {
        // Deserialize the binary payload into a Rust struct.
        Some(Bincode(payload)) => Ok(Some(payload)),
        _ => Ok(None),
    };

    tracing::info!(
        "❌ {}",
        format!(
            "{}: {}: {}",
            "Delete key / value pair from bucket".green(),
            key.to_string().bold().cyan(),
            format!("{:?}", it).bold().cyan()
        )
    );

    it
}

pub fn is_key_contained_in_bucket<
    'a,
    KeyT: kv::Key<'a> + Display,
    ValueT: Debug + Serialize + for<'d> Deserialize<'d>,
>(
    my_payload_bucket: &KVBucket<'a, KeyT, ValueT>,
    key: KeyT,
) -> miette::Result<bool> {
    let it = my_payload_bucket
        .contains(&key)
        .into_diagnostic()
        .wrap_err(KvErrorCouldNot::LoadKeyValuePairFromBucket)?;

    tracing::info!(
        "🔼 {}",
        format!(
            "{}: {}: {}",
            "Check if key is contained in bucket".green(),
            key.to_string().bold().cyan(),
            match it {
                true => "true".to_string().green(),
                false => "false".to_string().red(),
            }
        )
    );

    Ok(it)
}

pub mod kv_error {
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

        #[error("❌ Could not remove key/value pair from bucket")]
        RemoveKeyValuePairFromBucket,

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

    fn setup_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();
    }

    fn perform_db_operations(path: &Path, bucket: String) -> miette::Result<()> {
        setup_tracing();

        let path_str = path.to_string_lossy().to_string();
        let store = load_or_create_store(Some(&path_str))?;

        // Check that the kv store folder exists.
        assert!(check_folder_exists(path));

        // A bucket provides typed access to a section of the key/value store.
        let bucket = load_or_create_bucket_from_store(&store, Some(&bucket))?;

        // Check if "foo" is contained in the bucket.
        assert!(!(is_key_contained_in_bucket(&bucket, "foo".to_string())?));

        // Save to bucket.
        save_to_bucket(&bucket, "foo".to_string(), "bar".to_string())?;

        // Check if "foo" is contained in the bucket.
        assert!(is_key_contained_in_bucket(&bucket, "foo".to_string())?);

        // Load from bucket.
        assert_eq!(
            load_from_bucket(&bucket, "foo".to_string())?,
            Some("bar".to_string())
        );

        // Enumerate contents of bucket.
        let mut map = HashMap::new();
        for result_item in bucket.iter() {
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

        // Remove from bucket.
        assert_eq!(
            remove_from_bucket(&bucket, "foo".to_string())?,
            Some("bar".to_string())
        );

        // Check if "foo" is contained in the bucket.
        assert!(!(is_key_contained_in_bucket(&bucket, "foo".to_string())?));

        // Remove from bucket.
        assert_eq!(remove_from_bucket(&bucket, "foo".to_string())?, None);

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
