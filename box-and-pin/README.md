# box-and-pin

## Why do we need both Box and Pin?

[std::pin::Pin] is used all the time in async code, especially for cancel safety. And
[Box] is used everywhere for trait pointers.

This repo, article, and video is a deep dive of just these 2 small concepts using the
repo below (w/ richly formatted test output) as a guide.
- There are 3 examples in the repo that can be used to showcase this topic!
- And you have to use formatted output to get clarity on what's going on!

## Run the tests to see the output

To run these tests, in watch mode, use the following command:
```sh
cargo watch -x "test --lib -- --show-output"
```

To restrict the tests to just a specific test, eg `move_a_box`, use the following
command, in watch mode:
```sh
cargo watch -x "test --lib -- --show-output move_a_box"
```

To run a test just once, run:
```sh
cargo test --lib -- --show-output
```

## Formatting pointers

To format pointers in Rust, use the formatting trait
[`{:p}`](https://doc.rust-lang.org/std/fmt/#formatting-traits). You can format a
pointer by using two approaches:
1. Get the address of the pointer using [`std::ptr::addr_of!`] and then format it
   using `{:p}`. Eg: `let x = 1; println!("{:p}", std::ptr::addr_of!(x));`
2. Get a reference to the pointer using `&` and then format it using `{:p}`. Eg: `let
   x = 1; println!("{:p}", &x);`

## What is a smart pointer?

Smart pointers in Rust are data structures that act like pointers but also have
additional metadata and capabilities. They provide a level of abstraction over raw
pointers, offering features like ownership management, reference counting, and
more. Smart pointers often manage ownership of the data they point to, ensuring
proper deallocation when no longer needed.

## Memory allocation, stack and heap

- [Great visualization](https://courses.grainger.illinois.edu/cs225/fa2022/resources/stack-heap/)
