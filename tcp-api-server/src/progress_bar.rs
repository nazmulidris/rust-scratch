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

use indicatif::{ProgressBar, ProgressStyle};

/// More info:
/// - <https://docs.rs/indicatif/latest/indicatif/index.html#templates>
/// - <https://docs.rs/console/0.7.5/src/console/utils.rs.html#148-180>
pub fn create_progress_bar(message: &str, template: &str) -> ProgressBar {
    let message = message.to_string();
    if let Ok(template) = ProgressStyle::with_template(template) {
        ProgressBar::new_spinner()
            .with_style(template)
            .with_message(message)
    } else {
        ProgressBar::new_spinner().with_message(message)
    }
}
