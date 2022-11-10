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

mod common;

#[cfg(test)]
mod syntect {
  use crate::common::*;

  /// Use a [std::io::Cursor] as a fake [std::fs::File]:
  /// <https://stackoverflow.com/a/41069910/2085356>
  #[test]
  fn load_theme() -> std::io::Result<()> {
    let theme = try_load_r3bl_theme()?;
    dbg!(&theme);
    Ok(())
  }

  #[test]
  fn simple_md_highlight() {
    use syntect::{easy::*, highlighting::*, parsing::*, util::*};

    // Generate MD content.
    let md_content = get_md_file_no_frontmatter();

    // Load these once at the start of your program.
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme = try_load_r3bl_theme().unwrap();

    // Prepare Markdown syntax highlighting.
    let md_syntax = syntax_set.find_syntax_by_extension("md").unwrap();
    let mut highlight_lines = HighlightLines::new(md_syntax, &theme);

    for line in /* LinesWithEndings enables use of newlines mode. */ LinesWithEndings::from(&md_content) {
      let ranges: Vec<(Style, &str)> = highlight_lines.highlight_line(line, &syntax_set).unwrap();

      // for (style, text) in &ranges {
      //   dbg!(text);
      //   dbg!(style);
      //   println!("{}", as_24_bit_terminal_escaped(&ranges[..], false));
      // }

      let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
      print!("{}", escaped);
    }
  }
}
