# Type Theory Primer: Values as Types and Generic over Values

<!--
Copyright (c) 2026 Nazmul Idris. All rights reserved.
Licensed under the Apache License, Version 2.0.
-->

## Overview: What to Expect

In this video, we explore **ADT Const Params**—an unstable but powerful Rust feature that
allows you to use enum variants as generic parameters. We are moving beyond simple
generics into the realm of **Type Theory** and **Compiler Engineering**.

**If you watch this video, you will learn:**

1.  **Three Design Patterns**: We compare Runtime Enums, the Trait-based Typestate
    pattern, and ADT Const Params.
2.  **Type Theory Fundamentals**: We dive into Sum Types, Type Families, and
    Monomorphization Coordinates.
3.  **Hardware Proof**: We use `cargo-asm` to prove that the compiler physically deletes
    unreachable code through Algebraic Branch Pruning.

---

## The Problem: The High-Frequency Buffer Dilemma

In systems programming, we often face a conflict between **Safety** and **Performance**.

Imagine we are building a `Buffer` that stores byte data. We have two distinct operational
requirements:

1.  **Strict Mode**: Every time we push data, we MUST validate that it is valid UTF-8.
    This is critical when handling untrusted user input from a network socket.
2.  **Raw Mode**: We skip all validation. We trust the data source (e.g., a local cache)
    and want the CPU to move bytes as fast as possible without any branching or scanning.

**The Engineering Goal**: We want a single, ergonomic `Buffer` API that supports both
modes. Crucially, the "Raw Mode" must have **zero runtime overhead**—it should be as if
the validation code doesn't even exist in the binary.

We will solve this problem using three different approaches, culminating in the most
powerful solution: **ADT Const Params**.

---

## Scene 1: Approach 1 - The Runtime Check (The Hidden Tax)

The most common approach is to use a runtime flag (an enum field) to decide which path to
take.

```rust
pub enum EncodingPolicy { Raw, Strict }

pub struct Buffer {
    policy: EncodingPolicy,
    data: Vec<u8>,
}

impl Buffer {
    pub fn push(&mut self, incoming: &[u8]) {
        // ❌ The Hidden Tax:
        // Even if this Buffer instance is set to 'Raw', the CPU must
        // evaluate this branch on every single 'push' call.
        if let EncodingPolicy::Strict = self.policy {
            if std::str::from_utf8(incoming).is_err() {
                panic!("Invalid UTF-8!");
            }
        }
        self.data.extend_from_slice(incoming);
    }
}
```

---

## Scene 2: Approach 2 - The Trait Pattern (The Boilerplate Tax)

To move the decision to compile-time, we can use the **Typestate Pattern** with traits.
This removes the runtime branch but introduces a significant amount of boilerplate and
code fragmentation.

```rust
trait Policy { fn validate(data: &[u8]); }

struct Raw;
impl Policy for Raw { fn validate(_: &[u8]) {} } // No-op

struct Strict;
impl Policy for Strict {
    fn validate(d: &[u8]) { assert!(std::str::from_utf8(d).is_ok()); }
}

pub struct Buffer<P: Policy> {
    _marker: std::marker::PhantomData<P>,
    data: Vec<u8>,
}

impl<P: Policy> Buffer<P> {
    pub fn push(&mut self, incoming: &[u8]) {
        P::validate(incoming); // ✅ Zero runtime overhead
        self.data.extend_from_slice(incoming);
    }
}
```

**The Downside**: We have to define a trait, multiple marker structs, and multiple `impl`
blocks just to manage a simple binary choice.

---

## Scene 3: Approach 3 - ADT Const Params (The Physical Truth)

ADT Const Params allow us to use a value (a `const` enum variant) as a generic parameter.
This centralizes all logic within a single enum while maintaining type-level distinction.

```rust
#![feature(adt_const_params)]
use std::marker::ConstParamTy;

#[derive(Debug, PartialEq, Eq, ConstParamTy)]
pub enum EncodingPolicy { Raw, Strict }

pub struct Buffer<const P: EncodingPolicy> {
    data: Vec<u8>,
}

impl<const P: EncodingPolicy> Buffer<{ P }> {
    pub fn push(&mut self, incoming: &[u8]) {
        // ✅ The compiler treats 'P' as a constant.
        if P == EncodingPolicy::Strict {
            if std::str::from_utf8(incoming).is_err() {
                panic!("Invalid UTF-8!");
            }
        }
        self.data.extend_from_slice(incoming);
    }
}
```

---

## Scene 4: Type Theory Primer - Values as Coordinates

### 1. Generics over Values (Values as Types)

Standard Rust generics, such as `Vec<T>`, are **generics over types**. The compiler uses
the **Type** `T` as a "coordinate" to create a concrete implementation via
monomorphization.

**ADT Const Params** allow us to use **generics over values**. Instead of providing a
type, we provide a specific value of an **Algebraic Data Type (ADT)**. The compiler uses
the **Value** of the enum variant as the "coordinate" to create a concrete implementation
via monomorphization.

### 2. Sum Types at the Type Level

An `enum` is known in type theory as a **Sum Type** because its total state space is the
sum of its variants ($A + B$). By using it as a const generic, we "lift" this sum from the
value level (runtime) to the type level (compile-time).

### 3. Type Families

A struct using ADT Const Params is not just one type; it is a **Type Family**. Each
variant of the policy enum acts as a coordinate that identifies a unique, **disjoint
member** of that family. To the compiler, the following are distinct and different types:

- `Buffer<{EncodingPolicy::Raw}>`
- `Buffer<{EncodingPolicy::Strict}>`

### 4. The Algebra of Branch Pruning

Because the "choice" in our **Sum Type** is fixed at compile-time for any given member of
the type family, the compiler can apply **algebraic simplification**. When it sees a
`match` or `if` on a `const` value, it "multiplies the unreachable branches by zero,"
physically removing them from the binary. This is how we achieve **Zero Runtime
Overhead**.

---

## Scene 5: Observing the Machine (The Proof)

We prove the algebra by monomorphizing two disjoint members of the family and inspecting
the generated assembly using `cargo-asm`.

```rust
#[no_mangle]
pub fn push_raw(b: &mut Buffer<{EncodingPolicy::Raw}>, d: &[u8]) {
    b.push(d);
}

#[no_mangle]
pub fn push_strict(b: &mut Buffer<{EncodingPolicy::Strict}>, d: &[u8]) {
    b.push(d);
}
```

**Observation:**

1. **push_strict**: The assembly contains a `call` instruction to the UTF-8 validation
   logic.
2. **push_raw**: The `call` is gone. The `if` statement is gone. The logic has been
   pruned. The machine code is a raw memory copy.

---

## Scene 6: Real-World Deep Dive - ScopedMutex

We can see these patterns in action in the `r3bl-open-core` library's `ScopedMutex`. It
combines **ADT Const Params**, **Supertrait Bounds**, and a **Thread-Local State Machine**
to provide deadlock prevention with zero boilerplate.

### 1. Bypassing the Orphan Rule with Supertrait Bounds

Rust's **Orphan Rule** prevents us from adding methods to types defined in other crates
(like `std::sync::Mutex`). The common solution is an **Extension Trait**.

However, `ScopedMutex` uses a clever "Supertrait Bound" pattern to make the extension
feel native while keeping the implementation generic.

```rust
// From r3bl-open-core: tui/src/core/common/scoped_mutex/mutex_ext.rs
pub trait MutexExt<S>: Sized + Into<Mutex<S>> {
    fn into_scoped_mutex_panic_on_any_lock_nesting(self) 
        -> ScopedMutex<S, { DeadlockPreventionPolicy::PanicOnAnyLockNesting }> 
    {
        ScopedMutex { state: self.into() }
    }
    // ... other variants
}

// Blanket implementation for the actual Mutex struct
impl<S> MutexExt<S> for Mutex<S> {}
```

**The Magic**: By requiring `Into<Mutex<S>>` as a supertrait, the trait "knows" it is
operating on a `Mutex`. This allows the default implementation to use `self.into()` to
consume the mutex and wrap it in a `ScopedMutex`.

This works because of two standard library features:
1. **Reflexive `From` Implementation**: The stdlib provides `impl<T> From<T> for T`.
   This means any type `T` can be created from itself.
2. **Automatic `Into`**: The stdlib also provides `impl<T, U> Into<U> for T where U: From<T>`.

Since `Mutex<S>` can be created from `Mutex<S>` (reflexively), it automatically
implements `Into<Mutex<S>>`. This satisfies the supertrait bound "for free," allowing
our extension trait to "bind" itself to the `Mutex` struct without boilerplate.

### 2. ADT Const Params as Strategy Selectors

The `ScopedMutex` uses a `DeadlockPreventionPolicy` enum to determine how to handle
nested locks on the same thread.

```rust
#[derive(Debug, PartialEq, Eq, ConstParamTy)]
pub enum DeadlockPreventionPolicy {
    PanicOnAnyLockNesting,      // Strict: No nesting allowed
    PanicOnSpecificLockNesting, // Flexible: Nesting allowed, but not recursion
    OptOut                      // High-performance: No checks
}

pub struct ScopedMutex<S, const POLICY: DeadlockPreventionPolicy> {
    pub(super) state: Mutex<S>,
}
```

Just like our `Buffer` example, this allows the compiler to prune the "check" logic
entirely when the `OptOut` policy is used, while providing different safety guarantees
for the other variants—all within the same struct definition.

### 3. The Shared Ledger: A Thread-Local State Machine

To enforce these policies across *different* mutex instances, each thread maintains a
`SharedLedger`. This is a state machine that tracks which locks are currently held.

```rust
thread_local! {
    pub static THREAD_LOCAL_LEDGER: RefCell<SharedLedger> = const {
        RefCell::new(SharedLedger { held_policy: None, addresses: None })
    };
}

impl SharedLedger {
    pub fn try_acquire<S, const POLICY: DeadlockPreventionPolicy>(
        &mut self,
        mutex: &ScopedMutex<S, POLICY>
    ) -> Result<(), Error> {
        // 1. If POLICY is OptOut, return OK immediately (Branch Pruned!)
        // 2. If already holding PanicOnAny, Error!
        // 3. If holding PanicOnSpecific, check if this address is already held.
        // ...
    }
}
```

**The Physical Truth**: Because `POLICY` is a const generic, the `try_acquire` call inside
`ScopedMutex::read` or `write` can be partially evaluated at compile-time. If you use
`OptOut`, the entire thread-local lookup and state machine logic is physically deleted
from that specific monomorphized version of the function.

---

## Conclusion: The Finality of the Type System

ADT Const Params represent the ultimate expression of Rust's philosophy: **High-level
abstractions that compile down to the most efficient physical reality.**

By treating values as types, we gain the ergonomics of a centralized enum, the safety of
type-level contracts, and the performance of hand-optimized machine code.

---

### Links & Resources

- [ScopedMutex Deep Dive (Reference)](https://github.com/r3bl-org/r3bl-open-core/blob/main/tui/src/core/common/scoped_mutex/scoped_mutex_public_api.rs)
- [Rust Unstable Book: adt_const_params](https://doc.rust-lang.org/unstable-book/language-features/adt-const-params.html)
