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

/// This is the main parser entry point. It takes a string slice and if it can be parsed,
/// returns a [MarkdownDocument] that represents the parsed Markdown.
///
/// 1. [MarkdownLineOfText] roughly corresponds to a line of parsed text.
/// 2. [MarkdownDocument] contains all the blocks that are parsed from a Markdown string slice.
///
/// Each item in this [MarkdownDocument] corresponds to a block of Markdown
/// [MarkdownBlockElement], which can be one of the following variants:
/// 1. heading (which contains a [HeadingLevel] & [MarkdownLineOfText]),
/// 2. ordered & unordered list (which itself contains a [Vec] of [MarkdownLineOfText],
/// 3. code block (which contains string slices of the language & code),
/// 4. line (which contains a [MarkdownLineOfText]).
#[rustfmt::skip]
pub fn parse_markdown(input: &str) -> IResult<&str, MarkdownDocument> {
    many0(
        /* Each of these parsers end up scanning until EOL. */
        alt((
            map(parse_block_heading,                 MarkdownBlockElement::Heading),
            map(parse_block_unordered_list,          MarkdownBlockElement::UnorderedList),
            map(parse_block_ordered_list,            MarkdownBlockElement::OrderedList),
            map(parse_block_code,                    MarkdownBlockElement::Codeblock),
            map(parse_block_markdown_text_until_eol, MarkdownBlockElement::Line),
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
                    MarkdownBlockElement::Heading((
                        1.into(),
                        vec![MarkdownInlineElement::Plaintext("Foobar")]
                    )),
                    MarkdownBlockElement::Line(vec![]),
                    MarkdownBlockElement::Line(vec![MarkdownInlineElement::Plaintext(
                        "Foobar is a Python library for dealing with word pluralization."
                    )]),
                    MarkdownBlockElement::Line(vec![]),
                    MarkdownBlockElement::Codeblock(("bash", "pip install foobar\n")),
                    MarkdownBlockElement::Line(vec![]),
                    MarkdownBlockElement::Heading((
                        HeadingLevel::Heading2,
                        vec![MarkdownInlineElement::Plaintext("Installation")]
                    )),
                    MarkdownBlockElement::Line(vec![]),
                    MarkdownBlockElement::Line(vec![
                        MarkdownInlineElement::Plaintext("Use the package manager "),
                        MarkdownInlineElement::Link(("pip", "https://pip.pypa.io/en/stable/")),
                        MarkdownInlineElement::Plaintext(" to install foobar."),
                    ]),
                    MarkdownBlockElement::Codeblock((
                        "python",
                        r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                    )),
                ]
            ))
        )
    }
}
