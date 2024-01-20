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

use std::{thread::sleep, time::Duration};

use kv_example::{random_number, MyKeyType, MyResult, MyValueType};

const MY_DB_FOLDER: &str = "rkv_db_folder";

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
    Ok(())
}
