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

```rust
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

> 💡 Here's an example from the syn repo that shows you how to read in a Rust file and
> dump it into a syn AST:
> [dump-syntax](https://github.com/dtolnay/syn/blob/master/examples/dump-syntax/src/main.rs).
>
> 📜 You can find all the syn examples in this
> [repo](https://github.com/dtolnay/syn/tree/master/examples).
>
> 📜 You can find the solution to the proc macro workshop
> [here](https://github.com/jonhoo/proc-macro-workshop).

The rough idea is that we will have to parse "things" into this `proc_macro2::TokenStream`
in order to manipulate them. They can be parsed into this AST from:

1. [Strings](https://docs.rs/syn/0.15.44/syn/parse/index.html#the-synparse-functions),
2. Input to a derive macro,
3. Input to an attribute macro,
4. Input to a function like macro,
5. And even other ASTs generated by `quote!()` using
   [`parse_quite!()`](https://docs.rs/syn/0.15.44/syn/macro.parse_quote.html).

In order to do this parsing you have to use the
[`syn::parse*` functions](https://docs.rs/syn/0.15.44/syn/parse/index.html#the-synparse-functions).
When using any of them (macro form or otherwise) you have to provide the type that you
want the `TokenStream` to be parsed **into**. So here are some examples of what this looks
like.

1. This is how you parse a `TokenStream` into a `DeriveInput` using the
   `parse_macro_input!()` function (eg: in a derive macro):

   ```rust
   pub fn derive_proc_macro_impl(input: TokenStream) -> TokenStream {
     let DeriveInput {
       ident: struct_name_ident,
       data,
       generics,
       ..
     } = parse_macro_input!(input as DeriveInput); // Same as: syn::parse(input).unwrap();
     ...
   }
   ```

2. This is how you parse a string into a `proc_macro2::TokenStream` using the
   `parse_str()` function. Note that we have to provide the type that we want the `String`
   to be parsed **into** via the turbofish syntax, in this case `syn::Type`.

   ```rust
   let traits: Vec<&str> = vec!["std::default::Default", "std::fmt::Debug"];
   syn::parse_str::<syn::Type>(&traits.join(" + ")).unwrap();
   ```

3. It is possible to provide your own implementation of the `Parse` trait and hand it to
   syn to extract the AST you want out of the input `TokenStream`. The syn docs have an
   example of this [here](https://docs.rs/syn/0.15.44/syn/parse/index.html#example).
   There's also a [`Parser`
   trait](https://docs.rs/syn/0.15.44/syn/parse/index.html#the-parser-trait) that you can
   implement which allows you greater control over the parsing process.

## Writing a proc macro of any kind

There are 3 kinds of proc macros. Once you've created a new library crate for them inside
your project, you can do the following.

> 📜 This article will provide examples of each of these types of macros. You can find
> them all in this [repo](https://github.com/nazmulidris/rust_scratch/blob/main/macros/).

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

## Example of a simple function-like macro that dumps the AST

Let's start our procedural macro journey w/ something very simple. It's a macro that
doesn't really emit any token stream. It just prints out the AST of the input as debug. So
we won't be using `quote!()` but we will be using syn.

We will start by turning this one line function that's represented by this string literal.

```rust
let output_token_stream_str = "fn foo() -> u32 { 42 }";
```

The first thing we must do is define the macro in the `lib.rs` file.

```rust
extern crate proc_macro;
use proc_macro::TokenStream;

mod ast_viz_debug;

#[proc_macro]
pub fn fn_macro_ast_viz_debug(input: TokenStream) -> TokenStream {
  ast_viz_debug::fn_proc_macro_impl(input)
}
```

Let's write the `ast_viz_debug.rs` file next.

```rust
/// https://docs.rs/syn/1.0.52/syn/macro.parse_macro_input.html
pub fn fn_proc_macro_impl(_input: TokenStream) -> TokenStream {
  let output_token_stream_str = "fn foo() -> u32 { 42 }";
  let output = output_token_stream_str.parse().unwrap();

  let ast_item_fn: ItemFn = parse_str::<ItemFn>(output_token_stream_str).unwrap();
  viz_ast(ast_item_fn);

  output
}
```

Here's the function `viz_ast` that we'll use to print out the AST.

```rust
fn viz_ast(ast: ItemFn) {
  // Simply dump the AST to the console.
  let ast_clone = ast.clone();
  eprintln!("{} => {:#?}", style_primary("Debug::ast"), ast_clone);

  // Parse AST to dump some items to the console.
  let ItemFn {
    attrs,
    vis,
    sig,
    block,
  } = ast;

  eprintln!(
    "{} ast_item_fn {{ attrs.len:{}, vis:{}, sig:'{}' stmt: '{}' }}",
    style_primary("=>"),
    style_prompt(&attrs.len().to_string()),
    style_prompt(match vis {
      syn::Visibility::Public(_) => "public",
      syn::Visibility::Crate(_) => "crate",
      syn::Visibility::Restricted(_) => "restricted",
      syn::Visibility::Inherited => "inherited",
    }),
    style_prompt(&sig.ident.to_string()),
    style_prompt(&match block.stmts.first() {
      Some(stmt) => {
        let expr_str = stmt.to_token_stream().to_string().clone();
        expr_str
      }
      None => "empty".to_string(),
    }),
  );
}
```

> ⚡ To learn more about syn APIs, check out the following links:
>
> - <https://docs.rs/syn/1.0.52/syn/fn.parse_str.html>
> - <https://docs.rs/syn/1.0.52/syn/struct.ItemFn.html>
> - <https://docs.rs/syn/1.0.52/syn/struct.Attribute.html>
> - <https://docs.rs/syn/1.0.52/syn/enum.Visibility.html>
> - <https://docs.rs/syn/1.0.52/syn/struct.Signature.html>
> - <https://docs.rs/syn/1.0.52/syn/struct.Block.html>
> - <https://docs.rs/syn/1.0.52/syn/enum.Stmt.html>
> - <https://github.com/dtolnay/proc-macro-workshop#debugging-tips>

To test this function we can write the following test.

```rust
use my_proc_macros_lib::fn_macro_ast_viz_debug;

#[test]
fn test_proc_macro() {
  fn_macro_ast_viz_debug!();
  assert_eq!(foo(), 42);
}
```

- We can watch this test run using this script:
  `./cargo-watch-one-test.fish test_fn_macro_ast_viz_debug`
- We can watch the macros generated by this test expanded using this script:
  `./cargo-watch-macro-expand-one-test.fish test_fn_macro_ast_viz_debug`

> 📜 You can find another example of a function like procedural macro from the syn docs
> called [`lazy-static`](https://github.com/dtolnay/syn/tree/master/examples/lazy-static).
> It shows how to parse a custom syntax.

## TODO: 🎗️ Example of a complex function-like macro that generates manager of things

- TODO: take a look at https://github.com/dtolnay/syn/tree/master/examples/lazy-static

## Example of a simple derive macro that adds a method to a struct

We are going to come up w/ a made-up derive macro called `Describe` just for our
pedagogical purposes.

1. This derive macro will add a method to an annotated struct, enum, or union called
   `Describe` which simply returns a `String` that contains the names of the fields in the
   struct.
2. We will then extend this derive macro to handle generics.

### Test for expected output

Here are some simple cases that we should be able to handle in our initial implementation.

```rust
use my_proc_macros_lib::Describe;

#[test]
fn test_proc_macro() {
  #[derive(Describe)]
  struct MyStruct {
    my_string: String,
    my_enum: MyEnum,
    my_number: i32,
  }

  #[derive(Describe)]
  enum MyEnum {
    MyVariant1,
  }

  let foo = MyStruct {
    my_string: "Hello".to_string(),
    my_enum: MyEnum::MyVariant1,
    my_number: 42,
  };
  let foo = foo.describe();
  assert_eq!(
    foo,
    "MyStruct is a struct with these named fields: my_string, my_enum, my_number"
  );
}s
```

> ⚡ To run this test from the
> [repo](https://github.com/nazmulidris/rust_scratch/blob/main/macros/tests/test_derive_macro_describe.rs),
> in watch mode you can execute the following script:
> `./cargo-watch-one-test.fish test_derive_macro_describe`.

### Watch macro expansion

As we are developing this macro it is really useful not only to have the tests running (in
watch mode) but also have the macro expansion running in watch mode.

> ⚡ To run the macro expansion related to this test from the
> [repo](https://github.com/nazmulidris/rust_scratch/blob/main/macros/tests/test_derive_macro_describe.rs),
> in watch mode you can execute the following script:
> `./cargo-watch-macro-expand-one-test.fish test_derive_macro_describe`.

### Naive implementation

Let's implement this derive macro in a naive way. We won't handle generics, that will
happen [later](#better-implementation-that-handles-generics).

We have to define a function in `lib.rs` which will use the function that we will write
here.

```rust
extern crate proc_macro;
use proc_macro::TokenStream;

mod describe;

#[proc_macro_derive(Describe)]
pub fn derive_macro_describe(input: TokenStream) -> TokenStream {
  describe::derive_proc_macro_impl(input)
}
```

Now to create the `describe.rs` file which will have the `derive_proc_macro_impl`
function. This macro has to to be able to do the following things:

- For a `struct` or `enum` annotated with `#[derive(Describe)]` it will generate a method
  called `describe` which will return a `String` containing the names of the fields (named
  and unnamed) in the struct or enum.
- For a `union` annotated with `#[derive(Describe)]` it will generate a method called
  `describe` which will return a `String` containing the names of all the named fields in
  the union.

Here's what we have so far.

```rust
pub fn derive_proc_macro_impl(input: TokenStream) -> TokenStream {
  let DeriveInput {
    ident,
    data,
    ..
  } = parse_macro_input!(input as DeriveInput);

  let description_str = match data {
    Struct(my_struct) => gen_description_str_for_struct(my_struct),
    Enum(my_enum) => gen_description_str_for_enum(my_enum),
    Union(my_union) => gen_description_str_for_union(my_union),
  };

  quote! { /* todo */ }
}
```

Here's what the implementation of the `gen_description_str_for_struct` function looks
like.

```rust
fn gen_description_str_for_struct(my_struct: DataStruct) -> String {
  match my_struct.fields {
    Named(fields) => handle_named_fields(fields),
    Unnamed(fields) => handle_unnamed_fields(fields),
    Unit => handle_unit(),
  }
}

fn handle_named_fields(fields: FieldsNamed) -> String {
  let my_named_field_idents = fields.named.iter().map(|it| &it.ident);
  format!(
    "a struct with these named fields: {}",
    quote! {#(#my_named_field_idents), *}
  )
}

fn handle_unnamed_fields(fields: FieldsUnnamed) -> String {
  let my_unnamed_fields_count = fields.unnamed.iter().count();
  format!("a struct with {} unnamed fields", my_unnamed_fields_count)
}

fn handle_unit() -> String {
  format!("a unit struct")
}
```

And finally, here are the remainder of the functions.

```rust
fn gen_description_str_for_enum(my_enum: DataEnum) -> String {
  let my_variant_idents = my_enum.variants.iter().map(|it| &it.ident);
  format!(
    "an enum with these variants: {}",
    quote! {#(#my_variant_idents),*}
  )

fn gen_description_str_for_union(my_union: DataUnion) -> String {
  handle_named_fields(my_union.fields)
}
```

We actually haven't generated a token stream yet. We will do that in the next step using
`quote!` macro.

```rust
quote! {
  impl #generics #ident #generics #where_clause {
    fn describe(&self) -> String {
      let mut string = String::from(stringify!(#ident));
      string.push_str(" is ");
      string.push_str(#description_str);
      string
    }
  }
}
.into()
```

The `quote!` macro is incredibly powerful and it has a lot of smarts built into it which
we will see when we implement generics support next.

### Better implementation that handles generics

Here's an example of what a simple `Generics` object looks like when generated from
`struct Point<T> { ... }`.

1. The `Generics.params[0]` is a `TypeParam`, which is our `T`.
2. It contains a an `ident` which is the `T` identifier in our `struct Point<T> { ... }`.

```rust
Generics {
    lt_token: Some(
        Lt,
    ),
    params: [
        Type(
            TypeParam {
                attrs: [],
                ident: Ident {
                    ident: "T",
                    span: #0 bytes(706..707),
                },
                colon_token: None,
                bounds: [],
                eq_token: None,
                default: None,
            },
        ),
    ],
    gt_token: Some(
        Gt,
    ),
    where_clause: None,
}
```

Here's a function that we can use to parse this `Generics` object.

```rust
fn parse_generics(generics: &Generics) -> Option<Ident> {
  if let Some(generic_param) = generics.params.first() {
    // https://docs.rs/syn/1.0.52/syn/enum.GenericParam.html
    match generic_param {
      syn::GenericParam::Type(ref param) => Some(param.ident.clone()),
      syn::GenericParam::Lifetime(_) => unimplemented!(),
      syn::GenericParam::Const(_) => unimplemented!(),
    }
  } else {
    None
  }
}
```

And then we could use this in our procedural macro, like so:

```rust
let parsed_generics = parse_generics(&generics);
match parsed_generics {
  Some(ref _generic_ident) => {
    quote! {
      impl <#parsed_generics> #ident <#parsed_generics> {
        fn describe(&self) -> String {
          let mut string = String::from(stringify!(#ident));
          string.push_str(" is ");
          string.push_str(#description);
          string
        }
      }
    }
    .into() // Convert from proc_macro2::TokenStream to TokenStream.
  }
  None => {
    quote! {
      impl #ident  {
        fn describe(&self) -> String {
          let mut string = String::from(stringify!(#ident));
          string.push_str(" is ");
          string.push_str(#description);
          string
        }
      }
    }
    .into() // Convert from proc_macro2::TokenStream to TokenStream.
  }
}
```

This might provide some insight into how the `Generics` object itself is structured, but
there is no need to do any of this, since `quote!()` is awesome 🤯.

### Using quote!

Here's a mental model for using `quote!()`:

1. If you don't include the "thing" that you want to see in generated code, then it will
   be left out.
2. Conversely, if you want to see it in the generated code, then include it explicitly!

So, to handle generics, where you can have multiple types and where clauses, here's the
simple code 🎉.

```rust
pub fn derive_proc_macro_impl(input: TokenStream) -> TokenStream {
  let DeriveInput {
    ident,
    data,
    generics,
    ..
  } = parse_macro_input!(input as DeriveInput);

  let where_clause = &generics.where_clause;

  let description_str = match data {
    Struct(my_struct) => gen_description_str_for_struct(my_struct),
    Enum(my_enum) => gen_description_str_for_enum(my_enum),
    Union(my_union) => gen_description_str_for_union(my_union),
  };

  quote! {
    impl #generics #ident #generics #where_clause {
      fn describe(&self) -> String {
        let mut string = String::from(stringify!(#ident));
        string.push_str(" is ");
        string.push_str(#description_str);
        string
      }
    }
  }
  .into()
}
```

> 📜 Here's the source code for `describe.rs` from its
> [repo](https://github.com/nazmulidris/rust_scratch/blob/main/macros/my_proc_macros_lib/src/describe.rs).

Here are some tips and tricks for using `quote!()`:

1. Sometimes it is easier to start w/ a `String` or `Vec<String>` (which you can `join()`
   into a `String`), then parse that into a `TokenStream` using `syn::parse_str()`. Then
   pass that to `quote!()`. And example is if you wanted to add an arbitrary number of
   trait bounds to an existing `where` clause. It is just easier to manipulate the new
   trait bounds as a `String`, parse it into a `TokenStream`, and then use `quote!()` to
   add that to the existing `where` clause. Here's an example from
   [`builder.rs`](https://github.com/nazmulidris/rust_scratch/blob/main/macros/my_proc_macros_lib/src/builder.rs#L152).

   ```rust
   let traits: Vec<&str> = vec!["std::default::Default", "std::fmt::Debug"];
   syn::parse_str::<syn::Type>(&traits.join(" + ")).unwrap();
   ```

2. You can also use
   [`syn::parse_quote!()`](https://docs.rs/syn/0.15.44/syn/macro.parse_quote.html) to get
   a `TokenStream` from a `quote!()` expression, if it is just easier to generate a
   `quote!()` expression instead of using `String`, etc.
3. Repeating patterns in `quote!()` can be tricky to reason about. The best way to get a
   feel for how it works is to try various things and as soon as you run into some road
   blocks, think about generating `TokenStream`s manually, and then passing them to
   `quote!()`.

## Example of a complex derive macro that generates a builder

Now that we have seen a relatively simple derive procedural macro, let's look at a more
complex one that implements the builder pattern and supports generics. There are two
things this macro has to do:

1. Generate the `<Foo>Builder` struct that simply copies all the fields of the annotated
   struct.
2. Generate the impl block for the `<Foo>Builder` struct. It needs the following:
   1. Setter methods for each named field of the `<Foo>` struct.
   2. A `new()` method that returns a `<Foo>Builder` struct.
   3. A `build()` method that returns a `<Foo>` struct.

> 📜 You can get the source code for this example from its repo
> [here](https://github.com/nazmulidris/rust_scratch/blob/main/macros/my_proc_macros_lib/src/builder.rs).
> And you can get the source for the test
> [here](https://github.com/nazmulidris/rust_scratch/blob/main/macros/tests/test_derive_macro_builder.rs).

We need to make an entry in `lib.rs` for it, like so:

```rust
#[proc_macro_derive(Builder)]
pub fn
derive_macro_builder(input: TokenStream) -> TokenStream {
  builder::derive_proc_macro_impl(input)
}
```

Then we need to make a `builder.rs` file which contains the implementation of the derive
macro.

```rust
pub fn derive_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let DeriveInput {
    ident: struct_name_ident,
    data,
    generics,
    ..
  }: DeriveInput = parse_macro_input!(input as DeriveInput);

  let required_trait_bounds: Vec<&str> = vec!["std::default::Default", "std::fmt::Debug"];

  // Only generate code for struct.
  if data.is_struct() {
    with_data_struct_make_ts(&data, &|data_struct| {
      let builder_name_ident = struct_name_ident.from_string("{}Builder");

      let gen_props_setter_fns_ts =
        transform_named_fields_into_setter_fns_ts(data_struct);

      let gen_props_ts = transform_named_fields_to_props_ts(data_struct);

      let doc_struct_str = format!(
        " Implements the [builder pattern] for [`{}`].\n [builder pattern]: {}",
        &struct_name_ident, BUILDER_DOC_URL
      );

      let gen_props_with_defaults_ts =
        transform_named_fields_to_props_with_defaults_ts(data_struct);

      let new_or_modified_where_clause_ts =
        if does_where_clause_exist(&generics.where_clause) {
          add_trait_bounds_to_existing_where_clause_ts(
            &generics.where_clause,
            &required_trait_bounds,
          )
        } else {
          make_new_where_clause_with_default_trait_bounds_for_named_fields(data_struct)
        };

      let build_set_named_fields_ts = build_fn_set_named_fields_ts(data_struct);

      quote! {
        #[doc = #doc_struct_str]
        impl #generics #builder_name_ident #generics #new_or_modified_where_clause_ts {
          pub fn new() -> Self {
            Self {
              #gen_props_with_defaults_ts
            }
          }

          pub fn build(mut self) -> #struct_name_ident #generics {
            #struct_name_ident {
              #build_set_named_fields_ts
            }
          }

          #gen_props_setter_fns_ts
        }

        struct #builder_name_ident #generics #new_or_modified_where_clause_ts {
          #gen_props_ts
        }
      }
    })
  } else {
    quote! {}
  }
  .into()
}
```

Here's the test for the derive macro, `test_derive_macro_builder.rs`. They have to cover
all the different kinds of structs that we might encounter, some that have generics, some
that don't.

```rust
#[test]
fn test_proc_macro_struct_and_enum() {
  #[derive(Builder)]
  struct MyStruct {
    my_string: String,
    my_enum: MyEnum,
    my_number: i32,
  }

  enum MyEnum {
    MyVariant1,
  }

  impl Default for MyEnum {
    fn default() -> Self { MyEnum::MyVariant1 }
  }
}

#[test]
fn test_proc_macro_no_where_clause() {
  #[derive(Builder)]
  struct Point<X, Y> {
    x: X,
    y: Y,
  }

  let my_pt: Point<i32, i32> = PointBuilder::new()
    .set_x(1 as i32)
    .set_y(2 as i32)
    .build();

  assert_eq!(my_pt.x, 1);
  assert_eq!(my_pt.y, 2);
}

#[test]
fn test_proc_macro_generics() {
  #[derive(Builder)]
  struct Point<X, Y>
  where
    X: std::fmt::Display + Clone,
    Y: std::fmt::Display + Clone,
  {
    x: X,
    y: Y,
  }

  let my_pt: Point<i32, i32> = PointBuilder::new()
    .set_x(1 as i32)
    .set_y(2 as i32)
    .build();

  assert_eq!(my_pt.x, 1);
  assert_eq!(my_pt.y, 2);
}
```

Now that we have the skeleton of the entire thing, let's look at some details of how this
is implemented. It's worth taking a closer look at the
[`utils` module](https://github.com/nazmulidris/rust_scratch/blob/main/macros/my_proc_macros_lib/src/utils/mod.rs#),
since these contain re-usable functions that are leveraged to construct the final macro.

One pattern used here is extending some syn and proc_macro2 types with a new method.

1. The `syn::Data` type is extended w/ a method `is_struct` that can be used to check
   whether it contains a `struct` or not.
2. `proc_macro2::Ident` type is extended w/ a method `from_string` that can be used to
   create a `proc_macro2::Ident` from a string.

And there are some nice functions in `syn_parser_helpers.rs` that make it easier for us to
create lambdas that operate on named fields in the struct. We can use these to easily
create a `proc_macro2::TokenStream` that will do various things like:

1. Create a props for the `<Foo>Builder` `struct`.
2. Generate setter functions for the impl block of the `<Foo>Builder` `struct`.
3. Generate `where` clauses that add trait bounds to the existing or new `where` clause.

Please review the sources in detail to get a better understanding of how this is
implemented. One of the interesting things that this builder macro does is that it adds
trait bounds to the existing `where` clause. This is done to make sure that the
`<Foo>Builder` `struct` implements the `Default` trait for the `Foo` struct. It also adds
a trait bound for `Debug`. Here's a snippet of that.

```rust
let required_trait_bounds: Vec<&str> = vec!["std::default::Default", "std::fmt::Debug"];

fn add_trait_bounds_to_existing_where_clause_ts(
  where_clause: &Option<syn::WhereClause>,
  traits: &Vec<&str>,
) -> proc_macro2::TokenStream {
  // Must parse the `traits.join("+")` string into a [syn::Type].
  let joined_traits: syn::Type =
    syn::parse_str::<syn::Type>(&traits.join(" + ")).unwrap();

  let where_clause_ts = match where_clause {
    Some(where_clause) => {
      let where_predicate_punctuated_list = &where_clause.predicates;

      let modified_where_predicates_ts = where_predicate_punctuated_list
        .iter()
        .map(
          |where_predicate| match where_predicate {
            syn::WherePredicate::Type(_) => {
              quote! { #where_predicate + #joined_traits }
            }
            _ => quote! {},
          },
        )
        .collect::<Vec<_>>();

      quote! { where #(#modified_where_predicates_ts),* }
    }
    None => {
      quote! {}
    }
  };

  return where_clause_ts;
}
```

> 👀 Here are the scripts you can run to watch the macro expansion and test results as you
> make changes.
>
> - We can watch this test run using this script:
>   `./cargo-watch-one-test.fish test_derive_macro_builder`
> - We can watch the macros generated by this test expanded using this script:
>   `./cargo-watch-macro-expand-one-test.fish test_derive_macro_builder`

## TODO: 🎗️ Example of a simple attribute procedural macro that ???

- TODO: https://doc.rust-lang.org/book/ch19-06-macros.html#attribute-like-macros
- TODO: https://stackoverflow.com/a/52593373/2085356

> 📜 You can find another example of a attribute procedural macro from the syn docs called
> [`trace-var`](https://github.com/dtolnay/syn/tree/master/examples/trace-var).

## Learning resources

- Overview

  - [Excellent overview video](https://youtu.be/g4SYTOc8fL0)

- Books / articles

  - [Macro how to](https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros)
  - [Macro how to](https://doc.rust-lang.org/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes)

- Workshop

  - [Proc macro workshop](https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md)
  - [Proc macro workshop solutions](https://github.com/jonhoo/proc-macro-workshop)

- Technical guides to getting things working

  - [Tutorial - Add lib crate for macros](https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8)
  - [`lib.rs` restriction](https://users.rust-lang.org/t/how-to-import-procedural-macros-that-is-not-in-lib-rs/58323/9)
  - [Quote](https://docs.rs/quote)
  - [Syn](https://docs.rs/syn)

- Procedural macros workshop
  - [Workshop derive builder problem](https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md#derive-macro-derivebuilder)
  - [Solution hints for builder problem](https://github.com/dtolnay/proc-macro-workshop/blob/master/builder/tests/01-parse.rs)

## Wrapping up

> 📜 You can find all the examples of procedural macros shown in this article in this
> [repo](https://github.com/nazmulidris/rust_scratch/blob/main/macros/).
