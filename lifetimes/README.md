# lifetimes

Please read the article for this repo on
[developerlife.com](https://developerlife.com/2024/09/02/rust-lifetimes/).

And watch the videos here:

- [Rust lifetimes](https://youtu.be/eIJxAEcle7E)
- [Rust subtyping and variance](https://youtu.be/HRlpYXi4E-M)

# References

- Rustonomicon on [ownership](https://doc.rust-lang.org/nomicon/ownership.html):
  - [Aliasing](https://doc.rust-lang.org/nomicon/aliasing.html),
    [Lifetimes](https://doc.rust-lang.org/nomicon/lifetimes.html),
    [Lifetime Limits](https://doc.rust-lang.org/nomicon/lifetime-mismatch.html).
  - Use language `reference` and `referent`; similar to `pointer` and `pointee`.
  - Use this for [`for<'a>`](https://doc.rust-lang.org/nomicon/hrtb.html) aka higher-rank trait
    bounds (HRTB)
- Video outlining how to think about lifetimes with the
  [mental model](https://youtu.be/gRAVZv7V91Q?si=F4hKzDl6Ax983Spd) of valid memory locations
- This book has
  [fantastic visualizations](https://rust-book.cs.brown.edu/ch04-02-references-and-borrowing.html#references-change-permissions-on-paths)
  to understand Liveness, lifetime, permissions imposed by ownership model / borrow checker. Related
  [video](https://www.youtube.com/live/u85bozA3bv0?si=2DXaSyeVrc3ZCWYk)
- Google course on Rust
  [lifetimes section](https://google.github.io/comprehensive-rust/lifetimes.html)
- C/C++, pointers, and Rust benefits [video](https://youtu.be/2q3RLffSvEc?si=D6uQu_g-KdAOGjsc)
- June language to provide
  [some idea of how to describe](https://www.sophiajt.com/search-for-easier-safe-systems-programming/)
  Rust lifetimes.

# Copy and Clone

- <https://doc.rust-lang.org/std/marker/trait.Copy.html>
- <https://doc.rust-lang.org/std/clone/trait.Clone.html>

In Rust, a super trait is a trait that is required to be implemented by any type that implements
another trait. When you define a trait with a super trait, you are essentially saying that the
implementing type must also implement the super trait in order to fulfill the requirements of the
trait being defined.

For example, in the declaration `pub trait Copy: Clone {}`, the `Copy` trait is defined to require
that any type implementing `Copy` must also implement the `Clone` trait. This means that `Clone` is
the super trait of `Copy`.

A good analogy to help reason about super traits is to think of a hierarchy in a family tree:

- **Super Trait (Parent)**: Imagine `Clone` as a parent in a family. The parent has certain
  characteristics (methods) that all children (subtraits) must inherit.
- **Sub Trait (Child)**: Now, think of `Copy` as a child trait that inherits from the `Clone`
  parent. For any child to be recognized as a member of the family (i.e., to implement `Copy`), it
  must also possess the characteristics of the parent (i.e., implement `Clone`).

In this analogy:

- If you want to be a `Copy`, you must first be a `Clone`.
- Just like a child must inherit traits from their parent, a type must implement the methods defined
  in the `Clone` trait to be considered a `Copy`.

This relationship helps ensure that any type that is `Copy` can also be cloned, which is important
for the semantics of copying values in Rust.

## `&mut T` is not copy _due to lifetimes rules_

<https://doc.rust-lang.org/std/marker/trait.Copy.html#when-cant-my-type-be-copy>

Some types can’t be copied safely. For example, copying `&mut T` would create an aliased mutable
reference. Copying [String](https://doc.rust-lang.org/std/string/struct.String.html) would duplicate
responsibility for managing the [String](https://doc.rust-lang.org/std/string/struct.String.html)’s
buffer, leading to a double free.

Generalizing the latter case, any type implementing
[Drop](https://doc.rust-lang.org/std/ops/trait.Drop.html) can’t be `Copy`, because it’s managing
some resource besides its own [size_of::<T>](https://doc.rust-lang.org/std/mem/fn.size_of.html)
bytes.

If you try to implement `Copy` on a struct or enum containing non-`Copy` data, you will get the
error [E0204](https://doc.rust-lang.org/error_codes/E0204.html).

# Pedagogical examples

Prefer to use existing real world code. If that's not possible then create some examples.

1. Provide example of allocating something in the caller. Call the callee and pass this thing and
   get a slice into it back. Use this
   ([UnicodeString](https://github.com/r3bl-org/r3bl-open-core/blob/main/core/src/tui_core/graphemes/mod.rs))
   as the main example of allocating memory and dropping it in a parent scope, then have a struct
   that is only valid as long as that memory allocation in the parent scope hasn't been dropped.

1. Provide real examples of
   [splitting borrows](https://doc.rust-lang.org/nomicon/borrow-splitting.html) (like in editor
   engine and buffer). Rust understands that you can safely split a mutable reference into
   subfields.

1. Throw in `Cow` examples as well from `r3bl_tui`. related to borrows and mutating them (in
   `r3bl_tui`).

1. Provide examples of [`PhantomData`](https://doc.rust-lang.org/nomicon/phantom-data.html) and when
   to use it.

1. [`'static: 'a: 'b`](https://doc.rust-lang.org/nomicon/subtyping.html) - what does this mean?

- Thread spawn closure has `'static` lifetime, which means that there is no guarantee that join will
  be called and it will be dropped, after the spawn function ends.
- This is motivation for [scoped threads](https://doc.rust-lang.org/stable/std/thread/fn.scope.html)
  ...

1. `Copy` and `Clone` and `Arc` non-derive impl of `Clone` to handle lifetimes of inner types.

1. [`Drop`](https://doc.rust-lang.org/nomicon/dropck.html)
