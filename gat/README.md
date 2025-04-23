# GAT and HRTB

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Generic Associated Types](#generic-associated-types)
  - [From RFC: Remove the need for unsafe code to implement traits whose associated types have lifetimes derived from the context of the call or are generic but the traits are not](#from-rfc-remove-the-need-for-unsafe-code-to-implement-traits-whose-associated-types-have-lifetimes-derived-from-the-context-of-the-call-or-are-generic-but-the-traits-are-not)
  - [TL;DR: The trait does not have to be generic over types or lifetimes, but the associated type can](#tldr-the-trait-does-not-have-to-be-generic-over-types-or-lifetimes-but-the-associated-type-can)
  - [Example 1: Simple lifetime](#example-1-simple-lifetime)
  - [Example 2: Advanced lifetime with iterator](#example-2-advanced-lifetime-with-iterator)
  - [Example 3: Generic associated type with type parameter](#example-3-generic-associated-type-with-type-parameter)
- [Higher-Ranked Trait Bounds (HRTBs) example](#higher-ranked-trait-bounds-hrtbs-example)
  - [Example 1](#example-1)
    - [Practical use case](#practical-use-case)
    - [Explanation](#explanation)
    - [Explanation of the test](#explanation-of-the-test)
    - [Summary](#summary)
- [GAT with lifetime parameter vs. HRTB](#gat-with-lifetime-parameter-vs-hrtb)
  - [Clarification: GAT with Lifetime vs. HRTB](#clarification-gat-with-lifetime-vs-hrtb)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Generic Associated Types

- [More info](https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md)

### From RFC: Remove the need for unsafe code to implement traits whose associated types have lifetimes derived from the context of the call or are generic but the traits are not

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

### TL;DR: The trait does not have to be generic over types or lifetimes, but the associated type can

- The lifetime of the item yielded by an iterator should not be tied to the lifetime of
  the iterator.
  - Generic associated types (GATs) are a feature in Rust that allows you to define
    associated types that can take lifetime parameters that are derived from the context
    in which they're used and not just tied to `Self`.
  - The lifetime from the context has to be smaller than the lifetime of `Self`, ie,
    `Self` needs to outlive the context generally.
  - This will not allow you to get around returning a reference to something that is
    created in the block of a called function, to the caller. That is still not allowed in
    Rust.
  - Related examples: `ex_1.rs`, `ex_2.rs`
- The associated type can be generic over types, but the trait itself does not. This
  allows you to define a trait that can yield different types based on the context in
  which it's used, without making the entire trait generic.
  - Related example: `ex_3.rs`

### Example 1: Simple lifetime

```rust
trait Iter {
    type Item<'a>
    where
        Self: 'a;

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
    type Item<'a>
    where
        Self: 'a;

    fn g_get<'a>(&'a self) -> Option<Self::Item<'a>>;
}

pub struct MyContainer {
    value: String
}

impl Iter for MyContainer {
    type Item<'a> = &'a str
    where
        Self: 'a;

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

### Example 2: Advanced lifetime with iterator

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
    type Item<'b> = &'b mut [usize]
    where Self: 'b;

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

### Example 3: Generic associated type with type parameter

Here's an example of a trait with a generic associated type, but the trait itself is not
generic. This is in the `ex_3.rs` file.

```rust
trait Transformer {
    type Output<T>;

    fn transform<T>(&self, input: T) -> Self::Output<T>;
}

struct Wrapper;

impl Transformer for Wrapper {
    type Output<T> = Option<T>;

    fn transform<T>(&self, input: T) -> Self::Output<T> {
        Some(input)
    }
}

#[test]
fn test_transformer() {
    let wrapper = Wrapper;
    let result: Option<i32> = wrapper.transform(42);
    let result: Option<&str> = wrapper.transform("hello");
}
```

1. **Trait Definition**:

   - The `Transformer` trait defines an associated type `Output<T>` that takes a generic
     type `T`.
   - The trait itself does not take any generic parameters.

2. **Implementation**:

   - The `Wrapper` struct implements the `Transformer` trait.
   - The associated type `Output<T>` is defined as `Option<T>`.

3. **Usage**:

   - The `transform` method takes an input of type `T` and returns an `Option<T>` as the
     output.
   - This demonstrates how the associated type can depend on a generic type argument
     without the trait itself being generic.

4. **Test**:
   - The test case verifies the functionality by transforming an integer and a string,
     ensuring the generic behavior works as expected.

## Higher-Ranked Trait Bounds (HRTBs) example

HRTBs allow you to specify that a function or trait is valid for all possible lifetimes.
This is done using the `for<'a>` syntax.

### Example 1

In the example below, we define a function `F` that works for any lifetime `'a`.

```rust
fn apply_to_ref<F>(f: F)
where
    F: for<'any> Fn(&'any str),
{
    f("example");
}
```

#### Practical use case

A realistic use case for this function is when you need to apply a transformation or
operation to strings with varying lifetimes, such as logging, validation, or formatting.
The `for<'a>` syntax ensures that the function `f` can handle references with any
lifetime, making it flexible for different contexts.

The following examples demonstrate how to use HRTBs in a practical scenario. This example
is in `ex_4.rs`. Consider a logging system where you want to log messages with varying
lifetimes:

```rust
trait LogMessage {
    fn as_str(&self) -> &str;
}

impl LogMessage for &str {
    fn as_str(&self) -> &str {
        self
    }
}

impl LogMessage for String {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

struct Logger;

impl Logger {
    pub fn log<M>(&self, msg_ref: &M)
    where
        M: for<'any> LogMessage,
    {
        let log_entry = msg_ref.as_str();
        println!("LOG: {}", log_entry);
    }
}

fn process_message<'ret, 'msg, F, M>(message: &'msg M, processor: F) -> &'ret str
where
    M: for<'any> LogMessage + 'msg,
    'msg: 'ret, // 'm (lifetime of message) must outlive 'b (returned ref)
    F: FnOnce(&'msg M) -> &'ret str,
{
    processor(message)
}
```

#### Explanation

1. `LogMessage` trait and implementations:

- The `LogMessage` trait defines a simple contract: any type that implements it must be
  able to produce a `&str` through the `as_str` method. The lifetime of this `&str` is
  implicitly tied to the lifetime of `self`.
- We provide implementations for both `&str` and `String`. For `&str`, `as_str` simply
  returns itself. For `String`, it returns a string slice of the `String`'s content.

2. `Logger` struct and `log()` method:

- The `Logger` struct is straightforward.
- The `log()` method is generic over a type `M`. The crucial part is the `where` clause:
  `M: for<'any> LogMessage`. This is a Higher-Rank Trait Bound (HRTB). It means that for
  any possible lifetime `'any` that the caller might choose, the type `M` must implement
  the `LogMessage` trait.
- By taking `&M` as input, the `log()` function can work with references to types that
  implement `LogMessage`, regardless of the specific lifetime of that reference. The
  `as_str()` method is then called on this reference to get the string slice for logging.

3. `process_message()` function:

- This function is the core of the example demonstrating the need for HRTBs. It's generic
  over lifetimes:
  - `'ret` (the lifetime of the returned `&str`),
  - `'msg` (the lifetime of the input message),
  - a function type `F` (the processor closure),
  - and a type `M` (the type of the message).
- `message: &'msg M`: The `process_message()` function takes a reference to a message with
  lifetime `'msg`.
- `processor: F`: The processor is a closure that takes a reference to `M` with lifetime
  `'msg` and returns a `&str` with lifetime `'ret`.
- `where M: for<'any> LogMessage + 'msg`:
  - `M: for<'any> LogMessage`: This ensures that the message type can be used with the
    `Logger::log` function, which has the same HRTB requirement.
  - `+ 'msg`: This bound ensures that the type `M` lives at least as long as the lifetime
    `'msg` of the reference we are holding.
- `'msg: 'ret`: **This is the crucial lifetime constraint**. It states that the lifetime
  of the borrowed message (`'msg`) must outlive or be equal to the lifetime of the
  returned `&str` (`'ret`). This is essential for the borrow checker to ensure that the
  returned reference is always valid.
- The function calls the processor closure with a reference to the message and returns the
  `&str` that the closure produces.

#### Explanation of the test

Here's the test:

```rust
#[test]
fn test() {
    let logger = Logger;

    let short_lived_string = String::from("Temporary message");
    let processed_ref: &str = process_message(&short_lived_string, |msg| {
        logger.log(msg);
        msg.as_str()
    });

    println!("Processed message: {}", processed_ref);

    {
        let very_short_lived = "Ephemeral";
        let processed_ephemeral: &str = process_message(&very_short_lived, |msg| {
            logger.log(msg);
            msg.as_str()
        });
        println!("Processed ephemeral: {}", processed_ephemeral);
    }
}
```

This test function is relevant to the example because it directly demonstrates the core
problem that Higher-Rank Trait Bounds (HRTBs) are designed to solve: allowing a function
(`Logger::log()`) to work with data (`LogMessage` implementations) that have different and
potentially short lifetimes.

- The test creates messages with different lifetimes: a `String` (`short_lived_string`)
  with a lifetime tied to the test function's scope, and a string literal
  (`very_short_lived`) which has a `'static` lifetime but is used within a smaller scope.
- It passes these messages (via references) to the `process_message` function.
- Inside `process_message`, these messages are then passed to the `logger.log` function.
- The fact that `logger.log` can successfully handle both types of messages within the
  closures of `process_message` – without being restricted to a single, specific lifetime
  – showcases the power and necessity of the `for<'any> LogMessage` HRTB in its
  definition.
- The test proves that the `Logger` can indeed log messages regardless of their underlying
  lifetime, which is essential for the `process_message` function to operate correctly
  with various kinds of input.

This test directly validates that the `Logger::log` function, thanks to its HRTB, can
handle `LogMessage` implementations with the diverse lifetimes encountered within the
`process_message` function, illustrating the practical benefit of HRTBs in enabling
lifetime polymorphism.

#### Summary

- Why `for<'any>` is required in `Logger::log`:

  - The `for<'any> LogMessage` bound in `Logger::log` is essential for its flexibility. It
    allows the `log` function to accept references (`&M`) to any type `M` that can produce
    a log message, regardless of how long that `M` or the reference to it lives. Without
    the HRTB, the `log` function would be restricted to a specific lifetime, making it
    much less useful in scenarios like `process_message` where the lifetime of the message
    being logged can vary.

- Why the explicit lifetimes in `process_message` work:

  - The explicit lifetimes `'ret` and `'msg`, along with the `'msg: 'ret` constraint, are
    crucial for the borrow checker to understand the relationship between the lifetime of
    the input message and the lifetime of the `&str` returned by `process_message`. This
    ensures that the returned reference is always valid and doesn't outlive the data it
    borrows from, even when interacting with functions like `logger.log` that have HRTBs.

This pattern is useful in scenarios where you need to process or transform data with
varying lifetimes, such as logging, serialization, or applying generic operations to
borrowed data.

## GAT with lifetime parameter vs. HRTB

A practical application of Generic Associated Types (GATs) with lifetime parameters is in
scenarios where you need to manage resources with lifetimes tied to a parent resource. For
example, consider a database connection pool where each connection's lifetime is tied to
the lifetime of the pool. Here's how GATs can be used to model this:

```rust
trait ConnectionPool {
    type Connection<'a>
    where
        Self: 'a;

    fn get_connection<'a>(&'a self) -> Self::Connection<'a>;
}

struct DatabasePool {
    connections: Vec<String>, // Simulating connections as strings
}

impl ConnectionPool for DatabasePool {
    type Connection<'a> = &'a str
    where
        Self: 'a;

    fn get_connection<'a>(&'a self) -> Self::Connection<'a> {
        self.connections.get(0).map(|conn| conn.as_str()).unwrap()
    }
}

fn main() {
    let pool = DatabasePool {
        connections: vec![String::from("db_connection_1")],
    };

    let connection = pool.get_connection();
    println!("Using connection: {}", connection);
}
```

### Clarification: GAT with Lifetime vs. HRTB

This example demonstrates a **GAT with a lifetime parameter**, not an HRTB. The
distinction is as follows:

- **GAT with Lifetime**:

  - The `Connection<'a>` associated type is defined with a lifetime parameter `'a`.
  - The lifetime `'a` is derived from the context in which the method `get_connection` is
    called.
  - The `where Self: 'a` constraint ensures that the implementing type (`DatabasePool`)
    outlives the borrowed connection.

- **HRTB (Higher-Ranked Trait Bound)**:
  - Uses the `for<'a>` syntax to specify that a function or trait is valid for **all
    possible lifetimes**.
  - The lifetime is **universally quantified**, meaning it is not tied to the caller's
    context but must work for any lifetime.

This example does not use the `for<'a>` syntax and is therefore not an HRTB. Instead, it
showcases how GATs can be used to define associated types with lifetimes tied to the
context of their usage.
