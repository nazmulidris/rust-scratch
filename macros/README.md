# Procedural macros

| Macro type                 | Capabilities & limitations                                                                                               |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Declarative                | Can't handle generics, patterns capture items as _wholes_ and can't be broken down in the macro body                     |
| Procedural - function like | Operates on the code passed inside parenthesis of invocation to produce new token stream.                                |
| Procedural - derive        | Can't touch token stream of annotated `struct` or `enum`, only add new token stream below; can declare helper attributes |
| Procedural - attribute     | Like function-like, replaces token stream of annotated item (not just `struct` or `enum`)                                |

**Declarative macros** have many limitations but are easier to use. If you have simple use
cases they work great, since they are so easy to write.

**Procedural macros** are powerful and require some more work to setup in an existing
project (you have to create a new library create just for them and they all have to be
declared in the `lib.rs` file).

## Learning

- Overview

  - [Excellent overview video](https://youtu.be/g4SYTOc8fL0)

- Books / articles

  - [Macro how to](https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros)
  - [Macro how to](https://doc.rust-lang.org/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes)

- Workshop

  - [Proc macro workshop](https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md)

- Technical guides to getting things working

  - [Tutorial - Add lib crate for macros](https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8)
  - [Tutorial - Add lib crate for macros](https://blog.logrocket.com/procedural-macros-in-rust/)
  - [`lib.rs` restriction](https://users.rust-lang.org/t/how-to-import-procedural-macros-that-is-not-in-lib-rs/58323/9)
  - [Quote](https://docs.rs/quote)
  - [Syn](https://docs.rs/syn)

- Procedural macros workshop
  - [Workshop derive builder problem](https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md#derive-macro-derivebuilder)
  - [Solution hints for builder problem](https://github.com/dtolnay/proc-macro-workshop/blob/master/builder/tests/01-parse.rs)

## How to add a proc macro lib to your existing Rust project

Rust has two kinds of macros: declarative and procedural. Declarative macros are made
using `macro_rules!`. This section is about procedural macros which are the imperative
style of creating Rust macros.

The complication of using procedural macros is that they are not allowed to be used in the
same crate where your code lives. This requires us to create a new library create inside
our existing Rust project.

Here are the steps that we must take starting in our existing Rust project (which maybe a
lib or bin or both project).

1. Create a new crate inside our existing Rust project.
   - Run the following command to create a new `macro_lib` crate inside your existing
     project.
     ```shell
     $ cargo new --lib macro_lib
     ```
   - Inside the newly created `macro_lib` folder you will find:
     - A `Cargo.toml` file. Make sure to add these dependencies to this file:
       - `quote`
       - `syn`
       - `proc-macro2`
     - A `src` folder w/ a `lib.rs` file inside of it. All proc macro functions (annotated
       w/ `#[proc_macro]`) must be defined in this file and no other.
2. You now have to This declares this newly created crate as a dependency of your main
   project.
   - Add the following to your main project's `Cargo.toml` file:
     ```toml
     [dependencies]
     macro_lib = { path = "macro_lib" }
     ```
3. You can now use the code in this `macro_lib` crate by importing them in the code of
   your main like so: `use macro_lib::*`.

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

## Resources

Declarative macros

1. [Little book of Rust macros](https://danielkeep.github.io/tlborm/book/pim-README.html)
2. [Great YT video on declarative macros](https://youtu.be/q6paRBbLgNw)

References

1. https://blog.logrocket.com/procedural-macros-in-rust/
2. https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8
3. https://www.reddit.com/r/rust/comments/4lgb2o/newbie_question_multiple_library_crates_in_a/
4. https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
5. https://doc.rust-lang.org/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes
6. https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
