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

//! This module contains all the test cases for the [miette] crate. This isn't used by any
//! other module in the project, and is only used for testing purposes.

use app_use_case_into_diagnostic_and_context::UnderlyingDatabaseError;

/// USE CASE 1: Lazy, don't want to make my own [miette::Diagnostic] trait implementation
/// along with custom error types (using [thiserror]).
///
/// This is a sample of how to use the [miette] to auto generate error reports for
/// existing [Result]s containing [std::error::Error] types **WHICH DO NOT** implement
/// [miette::Diagnostic] trait (unlike the examples below, which do implement it). You can
/// also add additional context information to the error report, so that it can bubble up
/// w/ more context provided after the errors are generated, and they are getting bubbled
/// up the call chain.
#[cfg(test)]
pub mod app_use_case_into_diagnostic_and_context {
    use crossterm::style::Stylize;
    use miette::{Context, IntoDiagnostic};

    #[derive(Debug, thiserror::Error)]
    pub enum UnderlyingDatabaseError {
        #[error("database corrupted")]
        DatabaseCorrupted,
    }

    fn return_error_result() -> Result<u32, std::num::ParseIntError> {
        let error_result: Result<u32, std::num::ParseIntError> = "1.2".parse::<u32>();
        error_result
    }

    /// The errors are "unwrapped" / displayed in the opposite order in which they are
    /// added / stacked.
    ///
    /// Create order / stack insertion order:
    /// 1. The parse error `std::num::ParseIntError`, which is displayed last.
    /// 2. The error context `🍍 foo bar baz`, which is displayed 2nd from last.
    /// 3. The custom string error `custom string error`, which is displayed 3rd from last.
    /// 4. The entity not found error `std::io::ErrorKind::NotFound`, which is displayed 4th from last.
    /// 5. The database corrupted error `rkv::StoreError::DatabaseCorrupted`, which is displayed 5th from last.
    /// 6. The additional context error `🎃 this is additional context about the failure`, which is displayed first.
    ///
    /// Here's the actual output showing the inverted display order:
    /// ```text
    /// Err(  × 🎃 this is additional context about the failure
    ///   ├─▶ database corrupted
    ///   ├─▶ entity not found
    ///   ├─▶ custom string error
    ///   ├─▶ 🍍 foo bar baz
    ///   ╰─▶ invalid digit found in string
    /// )
    /// ```
    #[test]
    fn test_into_diagnostic() -> miette::Result<()> {
        let error_result = return_error_result();
        assert!(error_result.is_err());

        let new_miette_result = error_result
            .into_diagnostic()
            .context("🍍 foo bar baz")
            .wrap_err(miette::miette!("custom string error"))
            .wrap_err(std::io::ErrorKind::NotFound)
            .wrap_err(UnderlyingDatabaseError::DatabaseCorrupted)
            .wrap_err("🎃 this is additional context about the failure");

        assert!(new_miette_result.is_err());

        println!(
            "{}:\n{:?}\n",
            "debug output".blue().bold(),
            new_miette_result
        );

        let Err(ref miette_report) = new_miette_result else {
            panic!("This should result in an error!");
        };

        let mut iter = miette_report.chain();
        // First.
        assert_eq!(
            iter.next().map(|it| it.to_string()).unwrap(),
            "🎃 this is additional context about the failure"
        );
        // Second.
        assert_eq!(
            iter.next().map(|it| it.to_string()).unwrap(),
            "database corrupted"
        );
        // Third.
        assert_eq!(
            iter.next().map(|it| it.to_string()).unwrap(),
            "entity not found"
        );
        // Fourth.
        assert_eq!(
            iter.next().map(|it| it.to_string()).unwrap(),
            "custom string error"
        );
        // Fifth.
        assert_eq!(
            iter.next().map(|it| it.to_string()).unwrap(),
            "🍍 foo bar baz"
        );
        // Sixth.
        assert_eq!(
            iter.next().map(|it| it.to_string()).unwrap(),
            "invalid digit found in string"
        );

        Ok(())
    }

    /// It is possible to convert from a [miette::Report] into an [Box]'d [std::error::Error].
    #[test]
    fn test_convert_report_into_error() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let miette_result: miette::Result<u32> =
            return_error_result()
                .into_diagnostic()
                .wrap_err(miette::Report::msg(
                    "wrapper for the source parse int error",
                ));

        // let converted_result: std::result::Result<u32, Box<dyn std::error::Error>> =
        //     miette_result.map_err(|report| report.into());

        let converted_result: std::result::Result<(), Box<dyn std::error::Error>> =
            match miette_result {
                Ok(_) => Ok(()),
                Err(miette_report) => {
                    let boxed_error: Box<dyn std::error::Error> = miette_report.into();
                    Err(boxed_error)
                }
            };

        println!(
            "{}:\n{:?}\n",
            "debug output".blue().bold(),
            converted_result
        );

        assert!(converted_result.is_err());

        Ok(())
    }
}

/// USE CASE 2: Full custom error types, with [miette::Diagnostic] trait implementations.
///
/// More info on [miette] and [thiserror] crates:
/// - [tutorial](https://johns.codes/blog/build-a-db/part01)
/// - [thiserror](https://docs.rs/thiserror/latest/thiserror/#derives)
/// - [miette](https://docs.rs/miette/latest/miette/index.html)
///
/// The gist of creating custom error types is to use the [thiserror] crate to create the
/// error type, and then use the [miette] crate to create the error type's
/// [miette::Diagnostic] trait implementation (declaratively).
///
/// 1. The [thiserror] crate is used to create the error type. You can declaratively:
///    - Add the conversions from existing source errors to your custom type using the
///      `#[from]` attribute. This allows you to use the `?` operator to convert from the
///      source error to your custom error type.
///    - Provide really nice display output messages (using template literals to include
///      field values) for each error variant, using the `#[error]` attribute.
/// 2. Miette is used to create the [miette::Diagnostic] trait implementation for your
///    custom error type. This is done declaratively, and you can provide error codes and
///    help links for each error variant. The report is automatically generated by miette,
///    and you can customize if you like.
#[derive(thiserror::Error, Debug, miette::Diagnostic)]
enum KVStoreError {
    #[diagnostic(
        code(MyErrorCode::FileSystemError),
        help("https://docs.rs/rkv/latest/rkv/enum.StoreError.html"),
        // url(docsrs) /* Works if this code was on crates.io / docs.rs */
    )]
    #[error("📂 Could not create db folder: '{db_folder_path}' on disk")]
    CouldNotCreateDbFolder { db_folder_path: String },

    #[diagnostic(
        code(MyErrorCode::StoreCreationOrAccessError),
        help("https://docs.rs/rkv/latest/rkv/enum.StoreError.html"),
        // url(docsrs) /* Works if this code was on crates.io / docs.rs */
    )]
    #[error("💾 Could not get or create environment, or open store")]
    CouldNotGetOrCreateEnvironmentOrOpenStore {
        #[from]
        source: UnderlyingDatabaseError,
    },
}

/// USE CASE 2 (continued): Full custom error types, with [miette::Diagnostic] trait
/// implementations.
///
/// This is a sample of how to use the [miette] and [thiserror] crates to create custom
/// error types. All the tests in this module fail so that you can see how the error
/// messages are displayed.
#[cfg(test)]
mod library_use_case_fails_tests_miette_thiserror {
    use super::*;
    use pretty_assertions::assert_eq;

    // Flat error.
    fn return_flat_error_db() -> miette::Result<(), KVStoreError> {
        Result::Err(KVStoreError::CouldNotCreateDbFolder {
            db_folder_path: "some/path/to/db".into(),
        })
    }

    /// Expected output:
    /// ```text
    /// Error: MyErrorCode::FileSystemError
    ///
    /// × 📂 Could not create db folder: 'some/path/to/db' on disk
    /// help: https://docs.rs/rkv/latest/rkv/enum.StoreError.html
    /// ```
    #[test]
    fn fails_with_flat_error() -> miette::Result<()> {
        let result = return_flat_error_db();
        if let Err(error) = &result {
            assert_eq!(
                format!("{:?}", error),
                "CouldNotCreateDbFolder { db_folder_path: \"some/path/to/db\" }"
            );
        }

        // The following will induce a panic, since the error is not handled, and will display
        // a pretty message in the test output.
        result?;

        Ok(())
    }

    // Nested error.
    fn return_nested_error_store() -> miette::Result<(), KVStoreError> {
        // Variant 1 - Very verbose.
        let store_error = UnderlyingDatabaseError::DatabaseCorrupted;
        let rkv_error = KVStoreError::from(store_error);
        Result::Err(rkv_error)

        // Variant 2 - Same as above.
        // Err(UnderlyingDatabaseError::DatabaseCorrupted.into())

        // Variant 3 - Same as above.
        // Err(KVStoreError::CouldNotGetOrCreateEnvironmentOrOpenStore {
        //     source: UnderlyingDatabaseError::DatabaseCorrupted,
        // })
    }

    /// Expected output:
    /// ```text
    /// Error: MyErrorCode::StoreCreationOrAccessError
    ///   × 💾 Could not get or create environment, or open store
    ///   ╰─▶ database corrupted
    ///   help: https://docs.rs/rkv/latest/rkv/enum.StoreError.html
    /// ```
    #[test]
    fn fails_with_nested_error() -> miette::Result<()> {
        let result = return_nested_error_store();
        if let Err(rkv_error) = &result {
            assert_eq!(
                format!("{:?}", rkv_error),
                "CouldNotGetOrCreateEnvironmentOrOpenStore { source: DatabaseCorrupted }"
            );
        }

        // The following will induce a panic, since the error is not handled, and will display
        // a pretty message in the test output.
        result?;

        Ok(())
    }
}
