# nom parser combinator
<a id="markdown-nom-parser-combinator" name="nom-parser-combinator"></a>


<!-- TOC -->

- [Introduction w/ simple example](#introduction-w-simple-example)
- [Learn by doing](#learn-by-doing)
- [Documentation](#documentation)
- [Recipes](#recipes)

<!-- /TOC -->

## Introduction w/ simple example
<a id="markdown-introduction-w%2F-simple-example" name="introduction-w%2F-simple-example"></a>


nom is a parser combinator library for Rust. You can write small functions that parse a specific
part of your input, and then combine them to build a parser that parses the whole input.

Here's an example for parsing
[hex color codes](https://developer.mozilla.org/en-US/docs/Web/CSS/color).

> Make sure to run this code using `cargo test -- --nocapture` to see the test output in your
> terminal.

```rust
//! This module contains a parser that parses a hex color string into a [Color] struct.
//! The hex color string can be in the following format: `#RRGGBB`, eg: `#FF0000` for red.

use std::num::ParseIntError;
use nom::{bytes::complete::*, combinator::*, error::*, sequence::*, IResult, Parser};

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

/// Helper functions to match and parse hex digits. These are not [Parser] implementations.
mod helper_fns {
    use super::*;

    /// This function is used by [map_res] and it returns a [Result], not [IResult].
    pub fn parse_str_to_hex_num(input: &str) -> Result<u8, std::num::ParseIntError> {
        u8::from_str_radix(input, 16)
    }

    /// This function is used by [take_while_m_n] and as long as it returns `true` items will be
    /// taken from the input.
    pub fn match_is_hex_digit(c: char) -> bool {
        c.is_ascii_hexdigit()
    }

    pub fn parse_hex_seg(input: &str) -> IResult<&str, u8> {
        map_res(
            take_while_m_n(2, 2, match_is_hex_digit),
            parse_str_to_hex_num,
        )(input)
    }
}

/// These are [Parser] implementations that are used by [hex_color_no_alpha].
mod intermediate_parsers {
    use super::*;

    /// Call this to return function that implements the [Parser] trait.
    pub fn gen_hex_seg_parser_fn<'input, E>() -> impl Parser<&'input str, u8, E>
    where
        E: FromExternalError<&'input str, ParseIntError> + ParseError<&'input str>,
    {
        map_res(
            take_while_m_n(2, 2, helper_fns::match_is_hex_digit),
            helper_fns::parse_str_to_hex_num,
        )
    }
}

/// This is the "main" function that is called by the tests.
fn hex_color_no_alpha(input: &str) -> IResult<&str, Color> {
    // This tuple contains 3 ways to do the same thing.
    let it = (
        helper_fns::parse_hex_seg, // This is preferred.
        intermediate_parsers::gen_hex_seg_parser_fn(),
        map_res(
            take_while_m_n(2, 2, helper_fns::match_is_hex_digit),
            helper_fns::parse_str_to_hex_num,
        ),
    );
    let (input, _) = tag("#")(input)?;
    let (input, (red, green, blue)) = tuple(it)(input)?; // same as `it.parse(input)?`
    Ok((input, Color { red, green, blue }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_color() {
        let mut input = String::new();
        input.push_str("#2F14DF");
        input.push('🔅');

        let result = dbg!(hex_color_no_alpha(&input));

        let Ok((remainder, color)) = result else { panic!(); };
        assert_eq!(remainder, "🔅");
        assert_eq!(color, Color::new(47, 20, 223));
    }

    #[test]
    fn parse_invalid_color() {
        let result = dbg!(hex_color_no_alpha("🔅#2F14DF"));
        assert!(result.is_err());
    }
}
```

So what does this code do?

1. This string can be parsed: `#2F14DF🔅`. However, this string can't `🔅#2F14DF`.

How does this code work?

1. The key concept in nom is the `Parser` trait which is implemented for any `FnMut` that accepts an
   input and returns an `IResult<Input, Output, Error>`.
   - So if you write a simple function w/ the signature
     `fn(input: Input) -> IResult<Input, Output, Error>` then you are good to go! You just need to
     call `parse()` on the `Input` type and this will kick off the parsing.
   - Alternatively, you can just call the function directly via `tuple(...)(input)?`.
2. The `hex_color` function is the main function that orchestrates all the other functions to parse
   an `input: &str` and turn it into a `(&str, Color)`.
   - The `tag` combinator function is used to match the `#` character. This means that if the input
     doesn't start with `#`, the parser will fail (which is why `🔅#2F14DF` fails). It returns the
     remaining input after `#`.
   - The `parse_hex_num_fn` function is added to a tuple. An extension function on this tuple
     `parse` is called w/ the `input` (thus far). This is used to parse the hex number.
     - It returns the remaining input after the hex number which is why `#2F14DF🔅` returns `🔅` as
       the first item in the tuple.
     - The second item in the tuple is the parsed color string turned into a `Color` struct.
3. The `parse_hex_num_fn` actually does the work of parsing the hex strings, and it's signature
   tells nom that `parse` can use it to parse things. The return type of
   `IResult<Input, Output, Error>` is the key here. It uses the `map_res` combinator to combine:
   - `take_while_m_n`: This combinator takes a range of characters (2-2) and a function
     `is_hex_digit_fn` that determines if the character is a hex digit.
   - `from_hex_fn`: This function takes the hex number and turns it into a `u8`. It returns a
     `Result<u8, std::num::ParseIntError>`.

## Learn by doing
<a id="markdown-learn-by-doing" name="learn-by-doing"></a>


- There are lots of great examples of varying levels of complexity in the `src` folder in this repo.
- So if you run `cargo test -- --nocapture` you will see them all run.
- Most of these examples are written as tests. Their files are loaded by `main.rs`.
  - `convert_vec_string_for_parsing.rs`
  - `parse_simple_hex.rs`
  - `parse_simple_css.rs`
  - `parse_natural_lang.rs`
- The `md_parser` folder contains a fully functional Markdown parser (and isn't written as a test
  but a real module that you can use in your projects that need a Markdown parser). This module is
  linked via `lib.rs` (and not `main.rs`).

## Documentation
<a id="markdown-documentation" name="documentation"></a>


- Useful:
  - Videos:
    - [Intro from the author 7yrs old](https://youtu.be/EXEMm5173SM)
    - Nom 7 deep dive videos:
      - [Parsing name, age, and preference from natural language input](https://youtu.be/Igajh2Vliog)
      - [Parsing number ranges](https://youtu.be/Xm4jrjohDN8)
      - [Parsing lines of text](https://youtu.be/6b2ymQWldoE)
    - Nom 6 videos (deep dive into how nom combinators themselves are constructed):
      - [Deep dive, Part 1](https://youtu.be/zHF6j1LvngA)
      - [Deep dive, Part 2](https://youtu.be/9GLFJcSO08Y)
  - Tutorials:
    - [Build a JSON parser using nom7](https://codeandbitters.com/lets-build-a-parser/)
    - [Excellent beginner to advanced](https://github.com/benkay86/nom-tutorial)
    - [Write a parser from scratch](https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md)
  - Reference docs:
    - [What combinator or parser to use?](https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md)
    - [docs.rs](https://docs.rs/nom/7.1.3/nom/)
    - [Upgrading to nom 5.0](https://github.com/rust-bakery/nom/blob/main/doc/upgrading_to_nom_5.md)
- Less useful:
  - [README](https://github.com/rust-bakery/nom)
  - [nom crate](https://crates.io/crates/nom)

## Recipes
<a id="markdown-recipes" name="recipes"></a>


These are both snippets to parse a line of text:

```rust
let NEW_LINE = "$$$";
let EOL = "&&&";
let (input, output) =
    delimited(
        tag(NEW_LINE),
        recognize(many0(anychar)),
        tag(EOL)
    )(input)?;
```

```rust
let NEW_LINE = "$$$";
let EOL = "&&&";
let (input, (_, output, _)) =
    tuple((
        opt(tag(NEW_LINE)),
        take_until(EOL),
        tag(EOL)
    ))(input)?;
```
