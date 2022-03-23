---
title: "Guide to Rust procedural macros"
author: Nazmul Idris
date: 2022-03-23 15:00:00+00:00
excerpt: |
  Procedural macros are a way for you to extend the Rust complier and provide plugins
  that you can use to extend the language. They allow you to reduce the need to write
  manual boilerplate code, and even allow you to create your own DSL (domain specific
  language). This article goes into the details of create the 3 kinds of procedural macros
  in Rust.
layout: post
categories:
  - Rust
  - CLI
  - CC
---

<img class="post-hero-image" src="{{ 'assets/rust-proc-macro.svg' | relative_url }}"/>

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## What are procedural macros

**Procedural macros** are a way for you to extend the Rust complier and provide plugins
that you can use to extend the language. They are really powerful and require some more
work to setup in an existing project (you have to create a new library create just for
them and they all have to be declared in the `lib.rs` file). Here are the key benefits of
procedural macros:

- Minimize the amount of manual work you have to do in order to generate boilerplate code
  🎉. This is similar to
  [annotation processing](https://developerlife.com/2020/07/11/annotation-processing-kotlin-android/)
  in Java and Kotlin.
- You can create your own domain specific language like React JSX in Rust 🎉. Create your
  own
  [DSL (domain specific language)](https://developerlife.com/2020/04/04/kotlin-dsl-intro/)
  like in Kotlin and babel and JavaScript.

**Declarative macros** have many limitations but are easier to use. If you have simple use
cases they work great, since they are so easy to write.

> 💡 Here are some resources to help you w/ learning declarative macros. This article is
> dedicated to procedural macros.
>
> 1. [Little book of Rust macros](https://danielkeep.github.io/tlborm/book/pim-README.html)
> 2. [Great YT video on declarative macros](https://youtu.be/q6paRBbLgNw)

Here's a summary:

| Macro type                 | Capabilities & limitations                                                                                               |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Declarative                | Can't handle generics, patterns capture items as _wholes_ and can't be broken down in the macro body                     |
| Procedural - function like | Operates on the code passed inside parenthesis of invocation to produce new token stream.                                |
| Procedural - derive        | Can't touch token stream of annotated `struct` or `enum`, only add new token stream below; can declare helper attributes |
| Procedural - attribute     | Like function-like, replaces token stream of annotated item (not just `struct` or `enum`)                                |

## How to add a proc macro lib crate to your existing project

Rust has two kinds of macros: declarative and procedural. Declarative macros are made
using `macro_rules!` inline in your code w/out creating a new lib crate. This article is
about procedural macros which are the imperative style of creating Rust macros.

> 🤔 One complication with using procedural macros is that they are not allowed to be used
> in the same crate where your code lives. This requires us to create a new library create
> inside our existing Rust project.

The first step in using procedural macros is to create a new library crate.

Here are the steps that we must take starting in our existing Rust project (which maybe a
lib or bin or both project).

1. Create a new crate inside our existing Rust project.

- Run the following command to create a new `my_proc_macros_lib` crate inside your
  existing project.
  ```shell
  $ cargo new --lib my_proc_macros_lib
  ```
- Inside the newly created `my_proc_macros_lib` folder you will find:
  - A `Cargo.toml` file. Make sure to add these dependencies to this file:
    - `quote = "*"`
    - `syn = { version = "*", features = ["extra-traits"] }`
    - `proc-macro2 = "*"`
  - A `src` folder w/ a `lib.rs` file inside of it. All proc macro functions (annotated w/
    `#[proc_macro]`) must be defined in this file and no other. You can however import
    code from other modules just like normal. You can think of this file as a place where
    you "export" the definitions of your macros to other crates. Kind of like a registry
    or manifest of procedural macros in this lib crate that the Rust compiler can discover
    and use easily.

2. You now have to This declares this newly created crate as a dependency of your main
   project.

- Add the following to your main project's `Cargo.toml` file:
  ```toml
  [dependencies]
  my_proc_macros_lib = { path = "my_proc_macros_lib" }
  ```

3. You can now use the code in this `my_proc_macros_lib` crate by importing them in the
   code of your main like so: `use my_proc_macros_lib::*`.

Here's an example of a `Cargo.toml` for the proc macro lib crate:

```toml
[package]
name = "my_proc_macros_lib"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_proc_macros_lib"
path = "src/lib.rs"
proc-macro = true

[dependencies]
# https://github.com/dtolnay/proc-macro-workshop#debugging-tips
syn = { version = "*", features = ["extra-traits"] }
quote = "*"
proc-macro2 = "*"
r3bl_rs_utils = "*"
```

> 🗜️ It is also a good idea to install `cargo expand` to see what your code your macros
> actually expand into. You will need two things:
>
> 1. `cargo install cargo-expand` which installs `cargo expand`.
> 2. `rustup toolchain install nightly` which installs the Rust nightly toolchain that's
>    needed by `cargo expand`.
>
> Then you can run a command like the following
> `cargo expand --test test_derive_macro_describe` to expand the test
> `test_derive_macro_describe`.
>
> 👀 To watch for changes in your code and run the above command, you can install
> `cargo install cargo-watch` and then run:
> `cargo watch -x 'expand --test test_derive_macro_describe'`.
>
> 1. A script is provided called `cargo-watch-macro-expand-one-test.fish` which does this
>    for the test that you give that script as an argument.
> 2. Another script is provided called `cargo-watch-one-test.fish` which watches for
>    changes in your and then runs the test you give that script as an argument.

## What does a syn AST look like?

Before writing macros, let's talk about how we need to think about things:

1. Instead of working w/
   [`TokenStream`](https://doc.rust-lang.org/proc_macro/struct.TokenStream.html)s, we will
   work w/ an
   [AST (abstract syntax tree)](https://en.wikipedia.org/wiki/Abstract_syntax_tree)
   generated by [`syn::*`](https://github.com/dtolnay/syn/tree/master/examples) functions
   and macros. This will make our life much easier.

2. We will then walk parts of this tree and generate code using
   [`quote!`](https://docs.rs/quote/1.0.7/quote/macro.quote.html) which will generate a
   new `TokenStream` that will then be returned by our procedural macro.

Let's take a look at what an AST actually looks like. Here's an example of what you get
from parsing the string `"fn foo() -> u32 { 42 }"` using
[`syn::parse_str()`](https://docs.rs/syn/1.0.52/syn/fn.parse_str.html):

```text
    attrs: [],
    vis: Inherited,
    sig: Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: Fn,
        ident: Ident {
            ident: "foo",
            span: #5 bytes(91..125),
        },
        generics: Generics {
            lt_token: None,
            params: [],
            gt_token: None,
            where_clause: None,
        },
        paren_token: Paren,
        inputs: [],
        variadic: None,
        output: Type(
            RArrow,
            Path(
                TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: [
                            PathSegment {
                                ident: Ident {
                                    ident: "u32",
                                    span: #5 bytes(91..125),
                                },
                                arguments: None,
                            },
                        ],
                    },
                },
            ),
        ),
    },
    block: Block {
        brace_token: Brace,
        stmts: [
            Expr(
                Lit(
                    ExprLit {
                        attrs: [],
                        lit: Int(
                            LitInt {
                                token: 42,
                            },
                        ),
                    },
                ),
            ),
        ],
    },
}
```

Here's an example from the syn repo that shows you how to read in a Rust file and dump it
into a syn AST:
[dump-syntax](https://github.com/dtolnay/syn/blob/master/examples/dump-syntax/src/main.rs).

> ⚡ You can find all the syn examples
> [here](https://github.com/dtolnay/syn/tree/master/examples).

## Writing a proc macro of any kind

There are 3 kinds of proc macros. Once you've created a new library crate for them inside
your project, you can do the following.

```rust
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn my_fn_like_proc_macro(input: TokenStream) -> TokenStream {
  // 1. Use syn to parse the input tokens into a syntax tree.
  // 2. Use quote to generate new tokens based on what we parsed.
  // 3. Return the generated tokens.
  input
}

#[proc_macro_derive(MyDerive)]
pub fn my_derive_proc_macro(input: TokenStream) -> TokenStream {
  // 1. Use syn to parse the input tokens into a syntax tree.
  // 2. Generate new tokens based on the syntax tree. This is additive to the `enum` or
  //    `struct` that is annotated (it doesn't replace them).
  // 3. Return the generated tokens.
  input
}

#[proc_macro_attribute]
pub fn log_entry_and_exit(args: TokenStream, input: TokenStream) -> TokenStream {
  // 1. Use syn to parse the args & input tokens into a syntax tree.
  // 2. Generate new tokens based on the syntax tree. This will replace whatever `item` is
  //    annotated w/ this attribute proc macro.
  // 3. Return the generated tokens.
  input
}
```

## Example of a simple procedural macro that dumps the AST

- TODO: `fn_macro_ast_viz_debug.rs` goes here
- TODO: test using `./cargo-one.fish test_fn_macro_ast_viz_debug`

## Example of a simple declarative macros that adds a method to a struct

- TODO: `derive_macro_describe.rs` goes here
- TODO: testing using `./cargo-one.fish test_derive_macro_describe`

## Learning resources

- Overview

  - [Excellent overview video](https://youtu.be/g4SYTOc8fL0)

- Books / articles

  - [Macro how to](https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros)
  - [Macro how to](https://doc.rust-lang.org/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes)

- Workshop

  - [Proc macro workshop](https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md)

- Technical guides to getting things working

  - [Tutorial - Add lib crate for macros](https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8)
  - [`lib.rs` restriction](https://users.rust-lang.org/t/how-to-import-procedural-macros-that-is-not-in-lib-rs/58323/9)
  - [Quote](https://docs.rs/quote)
  - [Syn](https://docs.rs/syn)

- Procedural macros workshop
  - [Workshop derive builder problem](https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md#derive-macro-derivebuilder)
  - [Solution hints for builder problem](https://github.com/dtolnay/proc-macro-workshop/blob/master/builder/tests/01-parse.rs)
