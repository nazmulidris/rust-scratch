# Generic Associated Types

More info: https://minikin.me/blog/rust-type-system-deep-dive

TL;DR:

- Generic associated types (GATs) are a feature in Rust that allows you to define
  associated types that can take lifetime parameters that are derived from the context in
  which they're used and not just tied to `Self`.
- The lifetime from the context has to be smaller than the lifetime of `Self`, ie, `Self`
  needs to outlive the context generally.
- This will not allow you to get around returning a reference to something that is created
  in the block of a called function, to the caller. That is still not allowed in Rust.

Here's an example:

```rust
trait Iter {
    type Item<'a>
    where Self: 'a;
    fn get<'a>(&'a self) -> Option<Self::Item<'a>>;
}
```

In Rust, the line `type Item<'a> where Self: 'a;` is:

- Defining an associated type `Item` for the trait `Iter`, with a lifetime parameter `'a`.
- The where `Self: 'a` constraint specifies that the type implementing the `Iter`
  trait must outlive the lifetime `'a`.
  - And `'a` is derived from the calling context.
  - While `Self` lifetime is the lifetime of the type implementing the trait.

Here's a breakdown of what this means:

- Associated Type: `Item` is an associated type of the `Iter` trait. This means that any
  type that implements the `Iter` trait will provide a concrete type for `Item`.

- Lifetime Parameter: The `<'a>` part indicates that `Item` is generic over a lifetime `'a`.
  This allows the associated type to depend on the lifetime of references that are used in
  the trait methods.

- Where Clause: The `where Self: 'a` constraint means that the implementing type (denoted as
  `Self`) must live at least as long as the lifetime `'a`. In other words, it ensures that the
  type implementing the trait can be safely referenced for the duration of `'a`.

This is particularly useful in scenarios where the associated type might involve
references that are tied to the lifetime `'a`. By enforcing the constraint `Self: 'a`, it
guarantees that the implementing type can be used safely in contexts where references with
that lifetime are involved.

Here's a code example that allows you to return a reference (`str` slice) to a `String`.
This is in the `lib.rs` file.

```rust
trait Iter {
    type Item<'a> where Self: 'a;

    fn g_get<'a>(&'a self) -> Option<Self::Item<'a>>;
}

pub struct MyContainer {
    value: String
}

impl Iter for MyContainer {
    type Item<'a> = &'a str where Self: 'a;

    fn g_get<'a>(&'a self) -> Option<Self::Item<'a>> {
        Some(&self.value.as_ref())
    }
}

fn main() {
    let underlying_value = String::from("abcd");
    let wrapper = MyContainer {
        value: underlying_value,
    };
    println!("{:?}", wrapper.g_get());
    assert_eq!(wrapper.g_get(), Some("abcd"));
}
```

