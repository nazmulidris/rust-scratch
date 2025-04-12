# Upcast and downcast traits

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Trait upcasting (access a supertrait from a subtrait)](#trait-upcasting-access-a-supertrait-from-a-subtrait)
- [Trait downcasting (access a concrete type from a dyn trait object)](#trait-downcasting-access-a-concrete-type-from-a-dyn-trait-object)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Trait upcasting (access a supertrait from a subtrait)

Here's a new Rust 1.86.0 feature:

- https://blog.rust-lang.org/2025/04/03/Rust-1.86.0.html#trait-upcasting

It allows code like this, which provides the ability to coerce a subtrait object to a
supertrait object without needing to use `as`:

```rust
trait SubTrait: Supertrait {}
trait Supertrait {}

fn upcast(x: &dyn SubTrait) -> &dyn Supertrait {
    x
}
```

The same would work with any other kind of (smart-)pointer, like
`Arc<dyn Trait> -> Arc<dyn Supertrait>` or `*const dyn Trait -> *const dyn Supertrait`.

Previously this would have required a workaround in the form of an upcast method in the
`Trait` itself, for example `fn as_supertrait(&self) -> &dyn Supertrait`, and this would
work only for one kind of reference/pointer. Such workarounds are not necessary anymore.

## Trait downcasting (access a concrete type from a dyn trait object)

This code allows a concrete type to be accessed from a `dyn` trait object. Note the
`impl dyn Trait` syntax to define an `impl` block on the trait object itself.

```rust
use std::any::Any;

trait MyAny: Any {}

impl dyn MyAny {
    fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

impl MyAny for i32 {}
impl MyAny for String {}

fn main() {
    let x: Box<dyn MyAny> = Box::new(10_i32);
    let y: Box<dyn MyAny> = Box::new("hello".to_string());

    if let Some(i) = x.downcast_ref::<i32>() {
        println!("x is an i32: {}", i);
    }

    if let Some(s) = y.downcast_ref::<String>() {
        println!("y is a String: {}", s);
    }

    if x.downcast_ref::<String>().is_none() {
        println!("x is not a String");
    }
}
```

The primary use case for `impl dyn Trait` is to add utility methods to trait objects that
are universally applicable, regardless of the concrete type behind the trait object. Here
are some specific scenarios where it's particularly useful:

1. Downcasting and Type Introspection:

   As demonstrated in the previous examples, `impl dyn Trait` is excellent for providing
   downcasting capabilities. If you have a trait object and you need to recover the
   original concrete type, you can add a downcast_ref or downcast_mut method to the
   `impl dyn Trait` block. This is especially useful when you have a heterogeneous
   collection of trait objects and you need to perform type-specific operations on some of
   them.

2. Common Utility Methods:

   If you have certain utility functions that are relevant to all trait objects of a
   particular trait, you can add them to the `impl dyn Trait` block. For example, you
   might want to add a to_json method that serializes the trait object to JSON, or a
   debug_print method that prints debugging information.

3. Trait Object Extensions:

   Sometimes, you might want to add functionality to a trait object that's not part of the
   original trait definition. This can be useful for extending the functionality of
   existing traits without modifying their original definitions. For example, if you have
   a trait that handles network communications, you might add a function to log the data
   that is being sent, that is not part of the original trait.

4. Error Handling:

   You might want to add methods to the `impl dyn Trait` block that provide standardized
   error handling for trait objects. For example, you could add a method that returns a
   custom error type if a trait object operation fails.

5. Working with Heterogeneous Collections:

   When you have a collection of trait objects of the same trait, you can use the methods
   added to the `impl dyn Trait` block to perform operations on all the objects in the
   collection. This can simplify code that needs to work with diverse types that share a
   common trait.
