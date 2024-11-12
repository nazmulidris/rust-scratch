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

use super::{
    constants::DATABASE_URL,
    data_table_ops,
    db_support::{create_connection, try_run_migrations},
    file_table_ops,
};
use crossterm::style::Stylize as _;

pub fn run() -> miette::Result<()> {
    println!(
        "{}",
        "Running diesel_sqlite_ex".magenta().bold().underlined()
    );

    let connection = &mut create_connection(DATABASE_URL)?;

    if try_run_migrations(connection).is_err() {
        println!("Error running migrations");
        miette::bail!("Error running migrations");
    }

    println!("{}", "Running data_table_ops".magenta().bold().underlined());
    data_table_ops::insert_a_few_records(connection)?;
    data_table_ops::update_first_record(connection)?;
    data_table_ops::delete_last_record(connection)?;
    data_table_ops::print_all_records(connection)?;

    println!("{}", "Running file_table_ops".magenta().bold().underlined());
    file_table_ops::insert_a_few_records(connection)?;
    file_table_ops::update_first_record(connection)?;
    file_table_ops::delete_last_record(connection)?;
    file_table_ops::print_all_records(connection)?;

    Ok(())
}
