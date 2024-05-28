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
  easier to understand. Here are some resources where you can learn more about the
  Typestate Pattern in Rust:

# References

Functional typed design patterns:
- https://arxiv.org/pdf/2307.07069

More info on typestate:
- enums and typestate (and limitations): https://gemini.google.com/app/5bd7fed51858cb4d
- https://willcrichton.net/rust-api-type-patterns/typestate.html
- https://ruk.si/notes/rust/typestate/
- https://rustype.github.io/notes/notes/rust-typestate-series/rust-typestate-part-1
- https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html
- https://yoric.github.io/post/rust-typestate/
