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

#[cfg(test)]
pub mod simple_miette_usage {
    use crossterm::style::Stylize;
    use miette::{Context, IntoDiagnostic};

    #[derive(Debug, thiserror::Error)]
    pub enum UnderlyingDatabaseError {
        #[error("database corrupted")]
        DatabaseCorrupted,
    }

    fn return_error_result() -> Result<u32, std::num::ParseIntError> {
        "1.2".parse::<u32>()
    }

    #[test]
    fn test_into_diagnostic() -> miette::Result<()> {
        let error_result: Result<u32, std::num::ParseIntError> = return_error_result();
        assert!(error_result.is_err());

        // let it: u32 = error_result.into_diagnostic()?;

        let new_miette_result: miette::Result<u32> = error_result
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

        if let Err(ref miette_report) = new_miette_result {
            println!(
                "{}:\n{:?}\n",
                "miette report".red().bold(),
                miette_report.to_string()
            );

            let mut iter = miette_report.chain();

            // First.
            pretty_assertions::assert_eq!(
                iter.next().unwrap().to_string(),
                "🎃 this is additional context about the failure".to_string()
            );

            // Second.
            pretty_assertions::assert_eq!(
                iter.next().unwrap().to_string(),
                "database corrupted".to_string()
            );

            // Third.
            pretty_assertions::assert_eq!(
                iter.next().unwrap().to_string(),
                "entity not found".to_string()
            );

            // Fourth.
            pretty_assertions::assert_eq!(
                iter.next().unwrap().to_string(),
                "custom string error".to_string()
            );

            // Fifth.
            pretty_assertions::assert_eq!(
                iter.next().unwrap().to_string(),
                "🍍 foo bar baz".to_string()
            );

            // Final.
            pretty_assertions::assert_eq!(
                iter.next().unwrap().to_string(),
                "invalid digit found in string".to_string()
            );
        }

        Ok(())
    }

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

#[cfg(test)]
pub mod complex_miette_usage {
    use std::error::Error;

    use crate::simple_miette_usage::UnderlyingDatabaseError;
    use pretty_assertions::assert_eq;

    #[derive(thiserror::Error, Debug, miette::Diagnostic)]
    pub enum KvStoreError {
        #[diagnostic(
            code(MyErrorCode::FileSystemError),
            help("https://docs.rs/rkv/latest/rkv/enum.StoreError.html"),
            // url(docsrs) /* Works if this code was on crates.io / docs.rs */
        )]
        #[error("📂 Could not create db folder: '{db_folder_path}' on disk")]
        CouldNotCreateDbFolder { db_folder_path: String },

        #[diagnostic(
            code(MyErrorCode::StoreCreateOrAccessError),
            help("https://docs.rs/rkv/latest/rkv/enum.StoreError.html"),
            // url(docsrs) /* Works if this code was on crates.io / docs.rs */
        )]
        #[error("💾 Could not get or create environment, or open store")]
        CouldNotGetOrCreateEnvOrOpenStore {
            #[from]
            source: UnderlyingDatabaseError,
        },
    }

    fn return_flat_err() -> miette::Result<(), KvStoreError> {
        Result::Err(KvStoreError::CouldNotCreateDbFolder {
            db_folder_path: "some/path/to/db".to_string(),
        })
    }

    /// This test will not run! It will fail and demonstrate the default
    /// [report handler](miette::ReportHandler) of the `miette` crate.
    #[test]
    fn fails_with_flat_err() -> miette::Result<()> {
        let result = return_flat_err();

        if let Err(error) = &result {
            assert_eq!(
                format!("{:?}", error),
                "CouldNotCreateDbFolder { db_folder_path: \"some/path/to/db\" }"
            );
        }

        result?;

        Ok(())
    }

    fn return_nested_err() -> miette::Result<(), KvStoreError> {
        // Variant 1 - Very verbose.
        let store_error = UnderlyingDatabaseError::DatabaseCorrupted;
        let rkv_error = KvStoreError::from(store_error);
        Result::Err(rkv_error)

        // Variant 2.
        // Result::Err(KvStoreError::CouldNotGetOrCreateEnvOrOpenStore {
        //     source: UnderlyingDatabaseError::DatabaseCorrupted,
        // })
    }

    /// This test will not run! It will fail and demonstrate the default
    /// [report handler](miette::ReportHandler) of the `miette` crate.
    #[test]
    fn fails_with_nested_err() -> miette::Result<()> {
        let result = return_nested_err();

        if let Err(error) = &result {
            assert_eq!(
                format!("{:?}", error),
                "CouldNotGetOrCreateEnvOrOpenStore { source: DatabaseCorrupted }"
            );
        }

        result?;

        Ok(())
    }
}
