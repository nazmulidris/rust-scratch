# newtype design pattern, and `into Impl<T>`

- `ex_1.rs`
   - Example of 2D layout and sizing without using `newtype` design pattern and the
     pitfalls. Use `usize` everywhere and encounter issues.

- `ex_2.rs`
   - Use `newtype` design pattern to create core types that will be composed to make other
     types: `X`, `Y`, and `Width`, `Height` types. This allows us to get around the orphan
     rule for traits.

- `ex_3.rs`
   - Use `newtype` design pattern to create a `Point` type and a `Size` type by composing
     the core `X`, `Y`, `Width`, and `Height` types. This eliminates all the potential
     problems we saw in `ex_1.rs`, since bad code won't even compile! We can't use these
     structs in the wrong way if we tried.

- `ex_4.rs`
   - Extend this example to use `into Impl<T>` to create constructor functions for `X`,
     `Y`, `Width`, `Height`, types that can be constructed from `usize`, `f64`, and
     `String` types. This makes constructors and function arguments more flexible. Don't
     do this for `Point` and `Size` types, since we want to use `AddAssign` and `Add` for
     this.

- `ex_5.rs`
   - Extend this example to use `AddAssign` to add `X` and `Y` together to make a `Point`.
     And use `Add` to add `Width` and `Height` together to make a `Size`.

- `ex_6.rs`
   - Allow conversions between `Width` <-> `X` and `Height` <-> `Y` types. Generally index
     + 1 equals size. And size - 1 equals index. Do this conversion in the type system.
     Create a trait to check bounds containment and overflow.

- `ex_7.rs`
   - Impl `Drop` as a way to do cleanup when a `Point` or `Size` goes out of scope.

- r3bl-open-core examples
   - Show real world examples of where all of the above is done in `r3bl-open-core` repo.
