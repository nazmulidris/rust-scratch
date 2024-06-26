# typestate-pattern

The Typestate Pattern in Rust is a way to manage objects that go through different states
in their lifecycle. It leverages Rust's powerful type system to enforce these states and
transitions between them, making your code safer and more predictable.

Here are the key ideas behind the Typestate Pattern:

- *States as structs*: Each possible state of the object is represented by a separate
  struct. This lets you associate specific methods and data with each state.
- *Transitions with ownership*: Methods that transition the object to a new state consume
  the old state and return a value representing the new state. Rust's ownership system
  ensures you can't accidentally use the object in an invalid state.
- *Encapsulated functionality*: Methods are only available on the structs representing the
  valid states. This prevents you from trying to perform actions that aren't allowed in
  the current state.

Benefits of using the Typestate Pattern:

- *Safer code*: By statically checking types at compile time, the compiler prevents you from
  accidentally using the object in an invalid state. This leads to fewer runtime errors
  and more robust code.
- *Improved readability*: The code becomes more self-documenting because the valid state
  transitions are encoded in the types themselves.
- *Clearer APIs*: By separating functionality based on state, APIs become more intuitive and
  easier to understand.

## More resources on typestate pattern and others in Rust

- [Functional typed design patterns](https://arxiv.org/pdf/2307.07069).
- [Enums and typestate (and limitations)](https://gemini.google.com/app/5bd7fed51858cb4d).
- [Type-Driven API Design in Rust](https://willcrichton.net/rust-api-type-patterns/typestate.html).
- [Rust typestate notes](https://ruk.si/notes/rust/typestate/).
- [Rusty Typestates - Starting Out](https://rustype.github.io/notes/notes/rust-typestate-series/rust-typestate-part-1).
- [The Embedded Rust Book - Typestate programming](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html).
- [Typestates in Rust](https://yoric.github.io/post/rust-typestate/).

## YT video

Timecodes:
00:01 typestate-pattern in Rust
00:03 paper: typed design patterns for the functional typestateera
00:05 create a new crate for the live coding video
00:07 ex1.rs - model states using enum for InputEvent
00:22 ex2.rs - model EditorEvent, and transition InputEvent into EditorEvent
00:34 ex3.rs - typestate pattern using structs for HttpResponse
00:58 ex3_1.rs - typestate pattern using enums and PhantomData (not structs)

