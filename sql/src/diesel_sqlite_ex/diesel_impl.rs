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

use crossterm::style::Stylize as _;
use diesel::prelude::*;
use diesel::SqliteConnection;
use miette::{IntoDiagnostic, Result};

pub mod general_ops {
    use super::*;

    /// Specify your database URL, eg:
    /// - `path/to/your/file.db`
    /// - `file://file.db`
    /// - `:memory:`
    ///
    /// See [SqliteConnection] for more details.
    pub fn create_connection(database_url: &str) -> Result<SqliteConnection> {
        SqliteConnection::establish(database_url).into_diagnostic()
    }
}

pub mod data_table_ops {

    use super::*;
    use crate::diesel_sqlite_ex::{
        data_table,      /* from schema */
        DataTableRecord, /* from models */
    };

    pub fn print_all_records(connection: &mut SqliteConnection) -> Result<()> {
        let result_set = data_table::table
            .filter(data_table::id.is_not("1")) // Doesn't exclude anything, just an example.
            .limit(100) // Limit the number of records to fetch.
            .select(DataTableRecord::as_select())
            .load(connection)
            .into_diagnostic()?;

        println!("Number of records: {}", result_set.len());

        for (index, record) in result_set.iter().enumerate() {
            println!(
                "Row #{}: {}, {}, {}",
                (index + 1).to_string().blue(),
                record.id,
                record.name,
                record.data
            );
        }

        Ok(())
    }

    pub fn insert_a_few_records(connection: &mut SqliteConnection) -> Result<()> {
        let results = data_table::table
            .select((data_table::id, data_table::name, data_table::data))
            .load::<(String, String, String)>(connection)
            .into_diagnostic()?;

        for result in results {
            println!("{:?}", result);
        }

        Ok(())
    }
}

pub mod file_table_ops {
    use crate::diesel_sqlite_ex::schema::file_table::dsl::*;
}
