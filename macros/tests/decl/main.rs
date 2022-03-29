/*
 *   Copyright (c) 2022 Nazmul Idris
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
*/

/// Add sub-folders in integration tests folder:
/// https://users.rust-lang.org/t/how-to-test-file-under-sub-dir-of-tests-folder/58584
///
/// You can run them using: `./cargo-watch-one-test.fish decl`
///                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ^^^^
///                          Script that runs            Test folder name
///                          given test                  containing many tests
///
/// Note that it is not possible to run just 1 test file in this way. So the following
/// work: `./cargo-watch-one-test.fish decl_gen_dsl`

mod decl_gen_dsl;
mod decl_gen_lambda;
mod decl_gen_my_vec;
mod decl_gen_struct_1;
mod decl_gen_struct_2;
mod decl_gen_unwrap;
mod manager_of_things_async_test;
mod manager_of_things_test;
