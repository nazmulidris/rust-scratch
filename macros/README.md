# How to create procedural macros

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
     - A `Cargo.toml` file.
     - A `src` folder w/ a `lib.rs` file inside of it.
2. You now have to This declares this newly created crate as a dependency of your main
   project.
   - Add the following to your main project's `Cargo.toml` file:
     ```toml
     [dependencies]
     macro_lib = { path = "macro_lib" }
     ```
3. You can now use the code in this `macro_lib` crate by importing them in the code of
   your main like so: `use macro_lib::*`.

# Resources to learn from

Declarative macros

1. [Little book of Rust macros](https://danielkeep.github.io/tlborm/book/pim-README.html)
2. [Great YT video on declarative macros](https://youtu.be/q6paRBbLgNw)

Procedural macros

1. Proc macro workshop:
   <https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md>
2. Excellent overview video: <https://youtu.be/g4SYTOc8fL0>
3. Macro how to:
   <https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros>
4. Macro how to:
   <https://doc.rust-lang.org/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes>
5. Add lib crate for macros:
   <https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8>
6. `lib.rs` restriction:
   <https://users.rust-lang.org/t/how-to-import-procedural-macros-that-is-not-in-lib-rs/58323/9>
7. Quote: <https://docs.rs/quote>
8. Syn: <https://docs.rs/syn>

Procedural macros workshop (🔥 add to open tabs)

- Workshop derive builder problem:
  <https://github.com/dtolnay/proc-macro-workshop/blob/master/README.md#derive-macro-derivebuilder>
- Solution hints for builder problem:
  <https://github.com/dtolnay/proc-macro-workshop/blob/master/builder/tests/01-parse.rs>

# References

1. https://blog.logrocket.com/procedural-macros-in-rust/
2. https://dev.to/dandyvica/rust-procedural-macros-step-by-step-tutorial-36n8
3. https://www.reddit.com/r/rust/comments/4lgb2o/newbie_question_multiple_library_crates_in_a/
4. https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
5. https://doc.rust-lang.org/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes
6. https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
