/*
 *   Copyright (c) 2022 Nazmul Idris
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

//! Markdown Frontmatter: <https://markdoc.dev/docs/frontmatter#parse-the-frontmatter>
//! Raw literal strings:
//! - https://stackoverflow.com/a/35703815/2085356
//! - https://doc.rust-lang.org/reference/tokens.html#raw-string-literals
use std::borrow::Cow;

use super::FRONTMATTER_DELIMITER_PATTERN;

pub fn get_md_file_invalid_frontmatter<'caller>() -> Cow<'caller, str> {
  let markdown_input = r#"
---
# My Heading
"#;
  Cow::Borrowed(markdown_input)
}

pub fn get_md_file_no_frontmatter<'caller>() -> Cow<'caller, str> {
  let markdown_input = include_str!("assets/valid-content.md");
  Cow::Borrowed(markdown_input)
}

pub fn get_md_file_with_yaml_frontmatter<'caller>() -> Cow<'caller, str> {
  let frontmatter_yaml = include_str!("assets/valid-frontmatter.yaml");

  let markdown = get_md_file_no_frontmatter();

  let final_str = format!(
    "{}\n{}\n{}\n{}",
    FRONTMATTER_DELIMITER_PATTERN, frontmatter_yaml, FRONTMATTER_DELIMITER_PATTERN, markdown
  );
  Cow::Owned(final_str)
}

pub fn get_md_file_with_json_frontmatter<'caller>() -> Cow<'caller, str> {
  let frontmatter_json = include_str!("assets/valid-frontmatter.json");

  let markdown = get_md_file_no_frontmatter();

  let final_str = format!(
    "{}\n{}\n{}\n{}",
    FRONTMATTER_DELIMITER_PATTERN, frontmatter_json, FRONTMATTER_DELIMITER_PATTERN, markdown
  );
  Cow::Owned(final_str)
}
