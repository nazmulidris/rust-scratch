# newtype design pattern, and `into Impl<T>`

## ex_1.rs
Example of 2D layout and sizing without using `newtype` design pattern and the pitfalls.
Use `usize` everywhere and encounter issues.

## ex_2.rs
Example of using `newtype` design pattern to create a `Point` type and a `Size` type.
This allows us to use `Point` and `Size` in a type-safe manner. These are composed of
`X`, `Y`, and `Width`, `Height` types.

## ex_3.rs
Extend this example to use `into Impl<T>` to create `Point` and `Size` types that can
be constructed from `usize`, `f64`, and `String` types. This makes constructors and
function arguments more flexible.

## ex_4.rs
Extend this example to use `AddAssign` to add `X` and `Y` together to make a `Point`.
And use `Add` to add `Width` and `Height` together to make a `Size`.

## ex_5.rs
Allow conversions between `Point` and `Size` types. Generally index + 1 equals size.
And size - 1 equals index. Do this conversion in the type system. Create a trait to
check bounds containment and overflow.

## ex_6.rs
Impl `Drop` as a way to do cleanup when a `Point` or `Size` goes out of scope.

## r3bl-open-core examples
Show real world examples of where all of the above is done in `r3bl-open-core` repo.
