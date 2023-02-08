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

pub fn translate(md: Vec<MarkdownBlockElement>) -> String {
    md.iter()
        .map(|bit| match bit {
            MarkdownBlockElement::Heading((size, line)) => translate_header(size, line.to_vec()),
            MarkdownBlockElement::UnorderedList(lines) => translate_unordered_list(lines.to_vec()),
            MarkdownBlockElement::OrderedList(lines) => translate_ordered_list(lines.to_vec()),
            MarkdownBlockElement::Codeblock((lang, code)) => translate_codeblock(lang, code),
            MarkdownBlockElement::Line(line) => translate_line(line.to_vec()),
        })
        .collect::<Vec<String>>()
        .join("")
}

fn translate_bold(input: &str) -> String {
    format!("<b>{input}</b>")
}

fn translate_italic(input: &str) -> String {
    format!("<i>{input}</i>")
}

fn translate_inline_code(input: &str) -> String {
    format!("<code>{input}</code>")
}

fn translate_link(link_text: &str, url: &str) -> String {
    format!("<a href=\"{url}\">{link_text}</a>")
}

fn translate_image(link_text: &str, url: &str) -> String {
    format!("<img src=\"{url}\" alt=\"{link_text}\" />")
}

fn translate_list_elements(lines: Vec<MarkdownLineOfText>) -> String {
    lines
        .iter()
        .map(|line| format!("<li>{}</li>", translate_text(line.to_vec())))
        .collect::<Vec<String>>()
        .join("")
}

fn translate_header(heading_level: &HeadingLevel, text: MarkdownLineOfText) -> String {
    let heading_level_number = (*heading_level) as u8;
    format!(
        "<h{}>{}</h{}>",
        heading_level_number,
        translate_text(text),
        heading_level_number
    )
}

fn translate_unordered_list(lines: Vec<MarkdownLineOfText>) -> String {
    format!("<ul>{}</ul>", translate_list_elements(lines.to_vec()))
}

fn translate_ordered_list(lines: Vec<MarkdownLineOfText>) -> String {
    format!("<ol>{}</ol>", translate_list_elements(lines.to_vec()))
}

fn translate_codeblock(lang: &str, text: &str) -> String {
    format!("<pre><code class=\"lang-{lang}\">{text}</code></pre>")
}

fn translate_line(text: MarkdownLineOfText) -> String {
    let line = translate_text(text);
    if !line.is_empty() {
        format!("<p>{line}</p>")
    } else {
        line
    }
}

fn translate_text(text: MarkdownLineOfText) -> String {
    text.iter()
        .map(|part| match part {
            MarkdownInlineElement::Bold(text) => translate_bold(text),
            MarkdownInlineElement::Italic(text) => translate_italic(text),
            MarkdownInlineElement::BoldItalic(text) => translate_italic(&translate_bold(text)),
            MarkdownInlineElement::InlineCode(code) => translate_inline_code(code),
            MarkdownInlineElement::Link((text, url)) => translate_link(text, url),
            MarkdownInlineElement::Image((text, url)) => translate_image(text, url),
            MarkdownInlineElement::Plaintext(text) => text.to_string(),
        })
        .collect::<Vec<String>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_bold() {
        assert_eq!(translate_bold("bold af"), String::from("<b>bold af</b>"));
    }

    #[test]
    fn test_translate_italic() {
        assert_eq!(
            translate_italic("italic af"),
            String::from("<i>italic af</i>")
        );
    }

    #[test]
    fn test_translate_inline_code() {
        assert_eq!(
            translate_inline_code("code af"),
            String::from("<code>code af</code>")
        );
    }

    #[test]
    fn test_translate_link() {
        assert_eq!(
            translate_link("click me!", "https://github.com"),
            String::from("<a href=\"https://github.com\">click me!</a>")
        );
    }

    #[test]
    fn test_translate_image() {
        assert_eq!(
            translate_image("alt text", "https://github.com"),
            String::from("<img src=\"https://github.com\" alt=\"alt text\" />")
        );
    }

    #[test]
    fn test_translate_text() {
        let x = translate_text(vec![
            MarkdownInlineElement::Plaintext(
                "Foobar is a Python library for dealing with word pluralization.",
            ),
            MarkdownInlineElement::Bold("bold"),
            MarkdownInlineElement::Italic("italic"),
            MarkdownInlineElement::InlineCode("code"),
            MarkdownInlineElement::Link(("tag", "https://link.com")),
            MarkdownInlineElement::Image(("tag", "https://link.com")),
            MarkdownInlineElement::Plaintext(". the end!"),
        ]);
        assert_eq!(x, String::from("Foobar is a Python library for dealing with word pluralization.<b>bold</b><i>italic</i><code>code</code><a href=\"https://link.com\">tag</a><img src=\"https://link.com\" alt=\"tag\" />. the end!"));
        let x = translate_text(vec![]);
        assert_eq!(x, String::from(""));
    }

    #[test]
    fn test_translate_header() {
        assert_eq!(
            translate_header(
                &HeadingLevel::Heading1,
                vec![MarkdownInlineElement::Plaintext("Foobar")]
            ),
            String::from("<h1>Foobar</h1>")
        );
    }

    #[test]
    fn test_translate_list_elements() {
        assert_eq!(
            translate_list_elements(vec![
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
            ]),
            String::from("<li>Foobar</li><li>Foobar</li><li>Foobar</li><li>Foobar</li>")
        );
    }

    #[test]
    fn test_translate_unordered_list() {
        assert_eq!(
            translate_unordered_list(vec![
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
            ]),
            String::from("<ul><li>Foobar</li><li>Foobar</li><li>Foobar</li><li>Foobar</li></ul>")
        );
    }

    #[test]
    fn test_translate_ordered_list() {
        assert_eq!(
            translate_ordered_list(vec![
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
                vec![MarkdownInlineElement::Plaintext("Foobar")],
            ]),
            String::from("<ol><li>Foobar</li><li>Foobar</li><li>Foobar</li><li>Foobar</li></ol>")
        );
    }

    #[test]
    fn test_translate_codeblock() {
        assert_eq!(
            translate_codeblock(
                "python",
                r#"
import foobar

foobar.pluralize(\'word\') # returns \'words\'
foobar.pluralize(\'goose\') # returns \'geese\'
foobar.singularize(\'phenomena\') # returns \'phenomenon\'
"#
            ),
            String::from(
                r#"<pre><code class="lang-python">
import foobar

foobar.pluralize(\'word\') # returns \'words\'
foobar.pluralize(\'goose\') # returns \'geese\'
foobar.singularize(\'phenomena\') # returns \'phenomenon\'
</code></pre>"#
            )
        );
    }

    #[test]
    fn test_translate_line() {
        assert_eq!(
            translate_line(vec![
                MarkdownInlineElement::Plaintext("Foobar"),
                MarkdownInlineElement::Bold("Foobar"),
                MarkdownInlineElement::Italic("Foobar"),
                MarkdownInlineElement::InlineCode("Foobar"),
            ]),
            String::from("<p>Foobar<b>Foobar</b><i>Foobar</i><code>Foobar</code></p>")
        );
    }
}
