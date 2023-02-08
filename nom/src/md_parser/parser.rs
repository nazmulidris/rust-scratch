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
use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*,
    IResult,
};

/// All the Markdown literals that are used to perform parsing.
pub mod constants {
    pub const HEADING_CHAR: char = '#';
    pub const UNKNOWN_LANGUAGE: &str = "__UNKNOWN_LANGUAGE__";
    pub const SPACE: &str = " ";
    pub const PERIOD: &str = ".";
    pub const UNORDERED_LIST: &str = "-";
    pub const BITALIC_1: &str = "***";
    pub const BITALIC_2: &str = "___";
    pub const BOLD_1: &str = "**";
    pub const BOLD_2: &str = "__";
    pub const ITALIC_1: &str = "*";
    pub const ITALIC_2: &str = "_";
    pub const BACKTICK: &str = "`";
    pub const LEFT_BRACKET: &str = "[";
    pub const RIGHT_BRACKET: &str = "]";
    pub const LEFT_PAREN: &str = "(";
    pub const RIGHT_PAREN: &str = ")";
    pub const LEFT_IMG: &str = "![";
    pub const RIGHT_IMG: &str = "]";
    pub const NEW_LINE: &str = "\n";
    pub const CODE_BLOCK: &str = "```";
}

/// Skip rustfmt for this module: <https://stackoverflow.com/a/67289474/2085356>. It is cleaner
/// to write parsers w/out rustfmt reformatting them. This module is just to localize this directive
/// to rustfmt to suspend reformatting.
#[rustfmt::skip]
pub mod parser_impl {
    use crate::constants::*;
    use super::*;

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
    pub fn parse_markdown(input: &str) -> IResult<&str, MarkdownDocument> {
        many0(
            /* Each of these parsers end up scanning until EOL. */
            alt((
                map(parse_heading,                 MarkdownBlockElement::Heading),
                map(parse_unordered_list,          MarkdownBlockElement::UnorderedList),
                map(parse_ordered_list,            MarkdownBlockElement::OrderedList),
                map(parse_code_block,              MarkdownBlockElement::Codeblock),
                map(parse_markdown_text_until_eol, MarkdownBlockElement::Line),
            ))
        )(input)
    }

    pub fn parse_bold_italic(input: &str) -> IResult<&str, &str> {
        alt((
            delimited(/* start */ tag(BITALIC_1), /* output */ is_not(BITALIC_1), /* end */ tag(BITALIC_1)),
            delimited(/* start */ tag(BITALIC_2), /* output */ is_not(BITALIC_2), /* end */ tag(BITALIC_2)),
        ))(input)
    }

    pub fn parse_bold(input: &str) -> IResult<&str, &str> {
        alt((
            delimited(/* start */ tag(BOLD_1), /* output */ is_not(BOLD_1), /* end */ tag(BOLD_1)),
            delimited(/* start */ tag(BOLD_2), /* output */ is_not(BOLD_2), /* end */ tag(BOLD_2)),
        ))(input)
    }

    pub fn parse_italic(input: &str) -> IResult<&str, &str> {
        alt((
            delimited(/* start */ tag(ITALIC_1), /* output */ is_not(ITALIC_1), /* end */ tag(ITALIC_1)),
            delimited(/* start */ tag(ITALIC_2), /* output */ is_not(ITALIC_2), /* end */ tag(ITALIC_2))
        ))(input)
    }

    pub fn parse_code(input: &str) -> IResult<&str, &str> {
        delimited(
            /* start */ tag(BACKTICK), /* output */ is_not(BACKTICK), /* end */ tag(BACKTICK)
        )(input)
    }

    pub fn parse_link(input: &str) -> IResult<&str, (&str, &str)> {
        pair(
            delimited(/* start */ tag(LEFT_BRACKET), /* output */ is_not(RIGHT_BRACKET), /* end */ tag(RIGHT_BRACKET)),
            delimited(/* start */ tag(LEFT_PAREN),   /* output */ is_not(RIGHT_PAREN),   /* end */ tag(RIGHT_PAREN)),
        )(input)
    }

    pub fn parse_image(input: &str) -> IResult<&str, (&str, &str)> {
        pair(
            delimited(/* start */ tag(LEFT_IMG),   /* output */ is_not(RIGHT_IMG),   /* end */ tag(RIGHT_IMG)),
            delimited(/* start */ tag(LEFT_PAREN), /* output */ is_not(RIGHT_PAREN), /* end */ tag(RIGHT_PAREN)),
        )(input)
    }

    /// There must be at least one match. We want to match many things that are not any of our
    /// special tags, but since we have no tools available to match and consume in the negative case
    /// (without regex) we need to match against our (start) tags, then consume one char; we repeat
    /// this until we run into one of our special characters (start tags) then we return this slice.
    pub fn parse_plaintext(input: &str) -> IResult<&str, &str> {
        recognize(
            many1(
                preceded(
                    /* prefix - discarded */
                    not(
                        /* starts with special characters */
                        alt((
                            tag(BITALIC_1), tag(BITALIC_2),
                            tag(BOLD_1), tag(BOLD_2), tag(ITALIC_1), tag(ITALIC_2),
                            tag(BACKTICK), tag(LEFT_BRACKET), tag(LEFT_IMG), tag(NEW_LINE)
                        ))
                    ),
                    /* output - keep char */
                    anychar,
                )
            )
        )(input)
    }

    /// Parse a single chunk of markdown text [MarkdownInlineElement] in a single line.
    pub fn parse_markdown_inline(input: &str) -> IResult<&str, MarkdownInlineElement> {
        alt((
            map(parse_italic,       MarkdownInlineElement::Italic),
            map(parse_bold,         MarkdownInlineElement::Bold),
            map(parse_bold_italic,  MarkdownInlineElement::BoldItalic),
            map(parse_code,  MarkdownInlineElement::InlineCode),
            map(parse_image,        MarkdownInlineElement::Image),
            map(parse_link,         MarkdownInlineElement::Link),
            map(parse_plaintext,    MarkdownInlineElement::Plaintext),
        ))(input)
    }

    pub fn parse_markdown_text_until_eol(input: &str) -> IResult<&str, MarkdownLineOfText> {
        terminated(
            /* output inline markdown */ many0(parse_markdown_inline),
            /* ends with (discarded) */ tag(NEW_LINE)
        )(input)
    }

    /// Matches one or more `#` chars, consumes it, and outputs [HeadingLevel].
    pub fn parse_heading_tag(input: &str) -> IResult<&str, HeadingLevel> {
        map(
            terminated(
                /* output `#`+ */ take_while1(|it| it == constants::HEADING_CHAR),
                /* ends with (discarded) */ tag(constants::SPACE)
            ),
            |it: &str| HeadingLevel::from(it.len()),
        )(input)
    }

    /// This combines a tuple of the heading tag and text until EOL.
    pub fn parse_heading(input: &str) -> IResult<&str, (HeadingLevel, MarkdownLineOfText)> {
        tuple(
            (parse_heading_tag, parse_markdown_text_until_eol)
        )(input)
    }

    /// Matches `- ` & consumes it.
    pub fn parse_unordered_list_tag(input: &str) -> IResult<&str, &str> {
        terminated(
            /* output `-` */ tag(UNORDERED_LIST),
            /* ends with (discarded) */ tag(SPACE)
        )(input)
    }

    pub fn parse_unordered_list_element(input: &str) -> IResult<&str, MarkdownLineOfText> {
        preceded(
            /* prefix (discarded) */ parse_unordered_list_tag,
            /* output */ parse_markdown_text_until_eol
        )(input)
    }

    pub fn parse_unordered_list(input: &str) -> IResult<&str, Vec<MarkdownLineOfText>> {
        many1(
            parse_unordered_list_element
        )(input)
    }

    pub fn parse_ordered_list_tag(input: &str) -> IResult<&str, &str> {
        terminated(
            /* output */ terminated(/* output */ digit1, /* ends with (discarded) */ tag(PERIOD)),
            /* ends with (discarded) */ tag(SPACE),
        )(input)
    }

    pub fn parse_ordered_list_element(input: &str) -> IResult<&str, MarkdownLineOfText> {
        preceded(
            /* prefix (discarded) */ parse_ordered_list_tag,
            /* output */ parse_markdown_text_until_eol
        )(input)
    }

    pub fn parse_ordered_list(input: &str) -> IResult<&str, Vec<MarkdownLineOfText>> {
        many1(
            parse_ordered_list_element
        )(input)
    }

    pub fn parse_code_block(input: &str) -> IResult<&str, (/* lang */ &str, /* body */ &str)> {
        tuple(
            (parse_code_block_lang, parse_code_block_body)
        )(input)
    }

    pub fn parse_code_block_body(input: &str) -> IResult<&str, &str> {
        delimited(
            /* start */ tag(NEW_LINE),
            /* output */ is_not(CODE_BLOCK),
            /* end */ tag(CODE_BLOCK)
        )(input)
    }

    pub fn parse_code_block_lang(input: &str) -> IResult<&str, &str> {
        alt((
            // Either - Successfully parse both code block language & text.
            preceded(
                /* prefix - discarded */ tag(CODE_BLOCK),
                /* output */ parse_plaintext
            ),
            // Or - Fail to parse language, use unknown language instead.
            map(
                tag(CODE_BLOCK),
                |_| constants::UNKNOWN_LANGUAGE
            ),
        ))(input)
    }
}

#[cfg(test)]
mod md_parser_tests {
    use super::*;
    use nom::{error::Error, error::ErrorKind, Err as NomErr};
    use parser_impl::*;

    #[test]
    fn test_parse_italic() {
        assert_eq!(parse_italic("*here is italic*"), Ok(("", "here is italic")));

        assert_eq!(parse_italic("_here is italic_"), Ok(("", "here is italic")));

        assert_eq!(
            parse_italic("*here is italic"),
            Err(NomErr::Error(Error {
                input: "*here is italic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic("here is italic*"),
            Err(NomErr::Error(Error {
                input: "here is italic*",
                code: ErrorKind::Tag,
            }))
        );

        assert_eq!(
            parse_italic("here is italic"),
            Err(NomErr::Error(Error {
                input: "here is italic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic("*"),
            Err(NomErr::Error(Error {
                input: "*",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic("**"),
            Err(NomErr::Error(Error {
                input: "**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic("**we are doing bold**"),
            Err(NomErr::Error(Error {
                input: "**we are doing bold**",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_bold_italic() {
        assert_eq!(
            parse_bold_italic("***here is bitalic***"),
            Ok(("", "here is bitalic"))
        );

        assert_eq!(
            parse_bold("***here is bitalic"),
            Err(NomErr::Error(Error {
                input: "***here is bitalic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("here is bitalic***"),
            Err(NomErr::Error(Error {
                input: "here is bitalic***",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold_italic("___here is bitalic___"),
            Ok(("", "here is bitalic"))
        );

        assert_eq!(
            parse_bold_italic("___here is bitalic"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold_italic("here is bitalic___"),
            Err(NomErr::Error(Error {
                input: "here is bitalic___",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_bold() {
        assert_eq!(parse_bold("**here is bold**"), Ok(("", "here is bold")));

        assert_eq!(parse_bold("__here is bold__"), Ok(("", "here is bold")));

        assert_eq!(
            parse_bold("**here is bold"),
            Err(NomErr::Error(Error {
                input: "**here is bold",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("here is bold**"),
            Err(NomErr::Error(Error {
                input: "here is bold**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("here is bold"),
            Err(NomErr::Error(Error {
                input: "here is bold",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("****"),
            Err(NomErr::Error(Error {
                input: "****",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("**"),
            Err(NomErr::Error(Error {
                input: "**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("*"),
            Err(NomErr::Error(Error {
                input: "*",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("*this is italic*"),
            Err(NomErr::Error(Error {
                input: "*this is italic*",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_code() {
        assert_eq!(parse_bold("**here is bold**\n"), Ok(("\n", "here is bold")));
        assert_eq!(
            parse_code("`here is code"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_code("here is code`"),
            Err(NomErr::Error(Error {
                input: "here is code`",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_code("``"),
            Err(NomErr::Error(Error {
                input: "`",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_code("`"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_link() {
        assert_eq!(
            parse_link("[title](https://www.example.com)"),
            Ok(("", ("title", "https://www.example.com")))
        );
        assert_eq!(
            parse_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_image() {
        assert_eq!(
            parse_image("![alt text](image.jpg)"),
            Ok(("", ("alt text", "image.jpg")))
        );
        assert_eq!(
            parse_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_plaintext() {
        assert_eq!(parse_plaintext("1234567890"), Ok(("", "1234567890")));
        assert_eq!(parse_plaintext("oh my gosh!"), Ok(("", "oh my gosh!")));
        assert_eq!(parse_plaintext("oh my gosh!["), Ok(("![", "oh my gosh")));
        assert_eq!(parse_plaintext("oh my gosh!*"), Ok(("*", "oh my gosh!")));
        assert_eq!(
            parse_plaintext("*bold baby bold*"),
            Err(NomErr::Error(Error {
                input: "*bold baby bold*",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("[link baby](and then somewhat)"),
            Err(NomErr::Error(Error {
                input: "[link baby](and then somewhat)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("`codeblock for bums`"),
            Err(NomErr::Error(Error {
                input: "`codeblock for bums`",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("![ but wait theres more](jk)"),
            Err(NomErr::Error(Error {
                input: "![ but wait theres more](jk)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("here is plaintext"),
            Ok(("", "here is plaintext"))
        );
        assert_eq!(
            parse_plaintext("here is plaintext!"),
            Ok(("", "here is plaintext!"))
        );
        assert_eq!(
            parse_plaintext("here is plaintext![image starting"),
            Ok(("![image starting", "here is plaintext"))
        );
        assert_eq!(
            parse_plaintext("here is plaintext\n"),
            Ok(("\n", "here is plaintext"))
        );
        assert_eq!(
            parse_plaintext("*here is italic*"),
            Err(NomErr::Error(Error {
                input: "*here is italic*",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("**here is bold**"),
            Err(NomErr::Error(Error {
                input: "**here is bold**",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("`here is code`"),
            Err(NomErr::Error(Error {
                input: "`here is code`",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("[title](https://www.example.com)"),
            Err(NomErr::Error(Error {
                input: "[title](https://www.example.com)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("![alt text](image.jpg)"),
            Err(NomErr::Error(Error {
                input: "![alt text](image.jpg)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Eof
            }))
        );
    }

    #[test]
    fn test_parse_markdown_inline() {
        assert_eq!(
            parse_markdown_inline("*here is italic*"),
            Ok(("", MarkdownInlineElement::Italic("here is italic")))
        );
        assert_eq!(
            parse_markdown_inline("**here is bold**"),
            Ok(("", MarkdownInlineElement::Bold("here is bold")))
        );
        assert_eq!(
            parse_markdown_inline("`here is code`"),
            Ok(("", MarkdownInlineElement::InlineCode("here is code")))
        );
        assert_eq!(
            parse_markdown_inline("[title](https://www.example.com)"),
            Ok((
                "",
                (MarkdownInlineElement::Link(("title", "https://www.example.com")))
            ))
        );
        assert_eq!(
            parse_markdown_inline("![alt text](image.jpg)"),
            Ok((
                "",
                (MarkdownInlineElement::Image(("alt text", "image.jpg")))
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is plaintext!"),
            Ok(("", MarkdownInlineElement::Plaintext("here is plaintext!")))
        );
        assert_eq!(
            parse_markdown_inline("here is some plaintext *but what if we italicize?"),
            Ok((
                "*but what if we italicize?",
                MarkdownInlineElement::Plaintext("here is some plaintext ")
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is some plaintext \n*but what if we italicize?"),
            Ok((
                "\n*but what if we italicize?",
                MarkdownInlineElement::Plaintext("here is some plaintext ")
            ))
        );
        assert_eq!(
            parse_markdown_inline("\n"),
            Err(NomErr::Error(Error {
                input: "\n",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_markdown_inline(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Eof
            }))
        );
    }

    #[test]
    fn test_parse_markdown_text() {
        assert_eq!(parse_markdown_text_until_eol("\n"), Ok(("", vec![])));
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext\n"),
            Ok((
                "",
                vec![MarkdownInlineElement::Plaintext("here is some plaintext")]
            ))
        );
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                "",
                vec![
                    MarkdownInlineElement::Plaintext("here is some plaintext "),
                    MarkdownInlineElement::Italic("but what if we italicize?"),
                ]
            ))
        );
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext *but what if we italicize?* I guess it doesn't **matter** in my `code`\n"),
            Ok(("", vec![
                MarkdownInlineElement::Plaintext("here is some plaintext "),
                MarkdownInlineElement::Italic("but what if we italicize?"),
                MarkdownInlineElement::Plaintext(" I guess it doesn't "),
                MarkdownInlineElement::Bold("matter"),
                MarkdownInlineElement::Plaintext(" in my "),
                MarkdownInlineElement::InlineCode("code"),
            ]))
        );
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                "",
                vec![
                    MarkdownInlineElement::Plaintext("here is some plaintext "),
                    MarkdownInlineElement::Italic("but what if we italicize?"),
                ]
            ))
        );
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext *but what if we italicize?"),
            Err(NomErr::Error(Error {
                input: "*but what if we italicize?",
                code: ErrorKind::Tag
            })) // Ok(("*but what if we italicize?", vec![MarkdownInline::Plaintext(String::from("here is some plaintext "))]))
        );
    }

    #[test]
    fn test_parse_header_tag() {
        assert_eq!(parse_heading_tag("# "), Ok(("", 1.into())));
        assert_eq!(parse_heading_tag("### "), Ok(("", 3.into())));
        assert_eq!(parse_heading_tag("# h1"), Ok(("h1", 1.into())));
        assert_eq!(parse_heading_tag("# h1"), Ok(("h1", 1.into())));
        assert_eq!(
            parse_heading_tag(" "),
            Err(NomErr::Error(Error {
                input: " ",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_heading_tag("#"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_header() {
        assert_eq!(
            parse_heading("# h1\n"),
            Ok(("", (1.into(), vec![MarkdownInlineElement::Plaintext("h1")])))
        );
        assert_eq!(
            parse_heading("## h2\n"),
            Ok(("", (2.into(), vec![MarkdownInlineElement::Plaintext("h2")])))
        );
        assert_eq!(
            parse_heading("###  h3\n"),
            Ok((
                "",
                (3.into(), vec![MarkdownInlineElement::Plaintext(" h3")])
            ))
        );
        assert_eq!(
            parse_heading("###h3"),
            Err(NomErr::Error(Error {
                input: "h3",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_heading("###"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_heading(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_heading("#"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(parse_heading("# \n"), Ok(("", (1.into(), vec![]))));
        assert_eq!(
            parse_heading("# test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list_tag() {
        assert_eq!(parse_unordered_list_tag("- "), Ok(("", "-")));
        assert_eq!(
            parse_unordered_list_tag("- and some more"),
            Ok(("and some more", "-"))
        );
        assert_eq!(
            parse_unordered_list_tag("-"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag("-and some more"),
            Err(NomErr::Error(Error {
                input: "and some more",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag("--"),
            Err(NomErr::Error(Error {
                input: "-",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list_element() {
        assert_eq!(
            parse_unordered_list_element("- this is an element\n"),
            Ok((
                "",
                vec![MarkdownInlineElement::Plaintext("this is an element")]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(
                r#"- this is an element
- this is another element
"#
            ),
            Ok((
                "- this is another element\n",
                vec![MarkdownInlineElement::Plaintext("this is an element")]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(parse_unordered_list_element("- \n"), Ok(("", vec![])));
        assert_eq!(
            parse_unordered_list_element("- "),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_element("- test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_element("-"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list() {
        assert_eq!(
            parse_unordered_list("- this is an element"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list("- this is an element\n"),
            Ok((
                "",
                vec![vec![MarkdownInlineElement::Plaintext("this is an element")]]
            ))
        );
        assert_eq!(
            parse_unordered_list(
                r#"- this is an element
- here is another
"#
            ),
            Ok((
                "",
                vec![
                    vec![MarkdownInlineElement::Plaintext("this is an element")],
                    vec![MarkdownInlineElement::Plaintext("here is another")]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_ordered_list_tag() {
        assert_eq!(parse_ordered_list_tag("1. "), Ok(("", "1")));
        assert_eq!(parse_ordered_list_tag("1234567. "), Ok(("", "1234567")));
        assert_eq!(
            parse_ordered_list_tag("3. and some more"),
            Ok(("and some more", "3"))
        );
        assert_eq!(
            parse_ordered_list_tag("1"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag("1.and some more"),
            Err(NomErr::Error(Error {
                input: "and some more",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag("1111."),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Digit
            }))
        );
    }

    #[test]
    fn test_parse_ordered_list_element() {
        assert_eq!(
            parse_ordered_list_element("1. this is an element\n"),
            Ok((
                "",
                vec![MarkdownInlineElement::Plaintext("this is an element")]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(
                r#"1. this is an element
1. here is another
"#
            ),
            Ok((
                "1. here is another\n",
                vec![MarkdownInlineElement::Plaintext("this is an element")]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Digit
            }))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Digit
            }))
        );
        assert_eq!(parse_ordered_list_element("1. \n"), Ok(("", vec![])));
        assert_eq!(
            parse_ordered_list_element("1. test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_element("1. "),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_element("1."),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_ordered_list() {
        assert_eq!(
            parse_ordered_list("1. this is an element\n"),
            Ok((
                "",
                vec![vec![MarkdownInlineElement::Plaintext("this is an element")]]
            ))
        );
        assert_eq!(
            parse_ordered_list("1. test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list(
                r#"1. this is an element
2. here is another
"#
            ),
            Ok((
                "",
                vec![
                    vec!(MarkdownInlineElement::Plaintext("this is an element")),
                    vec![MarkdownInlineElement::Plaintext("here is another")]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_codeblock() {
        assert_eq!(
            parse_code_block(
                r#"```bash
pip install foobar
```"#
            ),
            Ok((
                "",
                (
                    "bash",
                    r#"pip install foobar
"#
                )
            ))
        );
        assert_eq!(
            parse_code_block(
                r#"```python
import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
```"#
            ),
            Ok((
                "",
                (
                    "python",
                    r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                )
            ))
        );
        // assert_eq!(
        // 	parse_code_block("```bash\n pip `install` foobar\n```"),
        // 	Ok(("", "bash\n pip `install` foobar\n"))
        // );
    }

    #[test]
    fn test_parse_codeblock_no_language() {
        assert_eq!(
            parse_code_block(
                r#"```
pip install foobar
```"#
            ),
            Ok((
                "",
                (
                    "__UNKNOWN_LANGUAGE__",
                    r#"pip install foobar
"#
                )
            ))
        );
    }

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
