/*
 *   Copyright (c) 2023 Nazmul Idris
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

use crate::*;
use nom::{branch::*, combinator::*, multi::*, IResult};

/// This is the main parser entry point. It takes a string slice and if it can be parsed, returns a
/// [Document] that represents the parsed Markdown.
///
/// 1. [Fragments] roughly corresponds to a line of parsed text.
/// 2. [Document] contains all the blocks that are parsed from a Markdown string slice.
///
/// Each item in this [Document] corresponds to a block of Markdown [Block], which can be one of the
/// following variants:
/// 1. heading (which contains a [Level] & [Fragments]),
/// 2. ordered & unordered list (which itself contains a [Vec] of [Fragments],
/// 3. code block (which contains string slices of the language & code),
/// 4. line (which contains a [Fragments]).
#[rustfmt::skip]
pub fn parse_markdown(input: &str) -> IResult<&str, Document> {
    many0(
        /* Each of these parsers end up scanning until EOL. */
        alt((
            map(parse_block_heading,                 Block::Heading),
            map(parse_block_unordered_list,          Block::UnorderedList),
            map(parse_block_ordered_list,            Block::OrderedList),
            map(parse_block_code,                    Block::CodeBlock),
            map(parse_block_markdown_text_until_eol, Block::Text),
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown() {
        assert_eq!(
            parse_markdown(
                r#"# Foobar

Foobar is a Python library for dealing with word pluralization.

```bash
pip install foobar
```
## Installation

Use the package manager [pip](https://pip.pypa.io/en/stable/) to install foobar.
```python
import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
```"#
            ),
            Ok((
                "",
                vec![
                    Block::Heading((1.into(), vec![Fragment::Plain("Foobar")])),
                    Block::Text(vec![]),
                    Block::Text(vec![Fragment::Plain(
                        "Foobar is a Python library for dealing with word pluralization."
                    )]),
                    Block::Text(vec![]),
                    Block::CodeBlock(CodeBlock::from(("bash", "pip install foobar\n"))),
                    Block::Text(vec![]),
                    Block::Heading((Level::Heading2, vec![Fragment::Plain("Installation")])),
                    Block::Text(vec![]),
                    Block::Text(vec![
                        Fragment::Plain("Use the package manager "),
                        Fragment::Link(("pip", "https://pip.pypa.io/en/stable/")),
                        Fragment::Plain(" to install foobar."),
                    ]),
                    Block::CodeBlock(CodeBlock::from((
                        "python",
                        r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                    ))),
                ]
            ))
        )
    }
}
