# Generic Associated Types

- [More info](https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md)

## From RFC: Remove the need for unsafe code to implement traits whose associated types have lifetimes derived from the context of the call

Consider the following trait as a representative motivating example:

```rust
trait StreamingIterator {
    type Item<'a>;
    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}
```

This trait is very useful - it allows for a "kind of" `Iterator` which yields values which
have a lifetime tied to the lifetime of the reference passed to `next()`.

- A particular obvious use case for this trait would be an iterator over a vector which
  yields overlapping, mutable subslices with each iteration.
- Using the standard `Iterator` interface, such an implementation would be invalid,
  - because: each slice would be required to exist for as long as the iterator,
  - rather: than for as long as the borrow initiated by `next()`.

This trait cannot be expressed in Rust as it exists today, because it depends on a sort of
higher-kinded polymorphism. This RFC would extend Rust to include that specific form of
higher-kinded polymorphism, which is referred to here as associated type constructors.
This feature has a number of applications, but the primary application is along the same
lines as the `StreamingIterator` trait: defining traits which yield types which have a
lifetime tied to the local borrowing of the receiver type.

## TL;DR: The lifetime of the item yielded by an iterator should not be tied to the lifetime of the iterator

- Generic associated types (GATs) are a feature in Rust that allows you to define
  associated types that can take lifetime parameters that are derived from the context in
  which they're used and not just tied to `Self`.
- The lifetime from the context has to be smaller than the lifetime of `Self`, ie, `Self`
  needs to outlive the context generally.
- This will not allow you to get around returning a reference to something that is created
  in the block of a called function, to the caller. That is still not allowed in Rust.

## Example 1: Simple

```rust
trait Iter {
    type Item<'a>
    where Self: 'a;
    fn get<'a>(&'a self) -> Option<Self::Item<'a>>;
}
```

In Rust, the line `type Item<'a> where Self: 'a;` is:

- Defining an associated type `Item` for the trait `Iter`, with a lifetime parameter `'a`.
- The where `Self: 'a` constraint specifies that the type implementing the `Iter` trait
  must outlive the lifetime `'a`.
  - And `'a` is derived from the calling context.
  - While `Self` lifetime is the lifetime of the type implementing the trait.

Here's a breakdown of what this means:

- Associated Type: `Item` is an associated type of the `Iter` trait. This means that any
  type that implements the `Iter` trait will provide a concrete type for `Item`.

- Lifetime Parameter: The `<'a>` part indicates that `Item` is generic over a lifetime
  `'a`. This allows the associated type to depend on the lifetime of references that are
  used in the trait methods.

- Where Clause: The `where Self: 'a` constraint means that the implementing type (denoted
  as `Self`) must live at least as long as the lifetime `'a`. In other words, it ensures
  that the type implementing the trait can be safely referenced for the duration of `'a`.

This is particularly useful in scenarios where the associated type might involve
references that are tied to the lifetime `'a`. By enforcing the constraint `Self: 'a`, it
guarantees that the implementing type can be used safely in contexts where references with
that lifetime are involved.

Here's a code example that allows you to return a reference (`str` slice) to a `String`.
This is in the `ex_1.rs` file.

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

## Example 2: Advanced

Here's a code example that allows you to return a mutable reference (`&mut [usize]`) to a
`Vec<usize>`. This is in the `ex_2.rs` file.

```rust
trait StreamingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

struct OverlappingSlices<'a> {
    data: &'a mut [usize],
    window_size: usize,
    position: usize,
}

impl<'a> OverlappingSlices<'a> {
    fn new(data: &'a mut [usize], window_size: usize) -> Self {
        Self {
            data,
            window_size,
            position: 0,
        }
    }
}

impl<'a> StreamingIterator for OverlappingSlices<'a> {
    type Item<'b>
        = &'b mut [usize]
    where
        Self: 'b;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>> {
        if self.position + self.window_size <= self.data.len() {
            let slice = &mut self.data[self.position..self.position + self.window_size];
            self.position += 1;
            Some(slice)
        } else {
            None
        }
    }
}
```
