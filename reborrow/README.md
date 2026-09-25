# Understanding Rust Reborrowing: `valid.rs` vs `invalid.rs`

This repository demonstrates how Rust's borrow checker handles mutable references
(`&mut T`), the mechanics of **reborrowing**, and the precise boundary between valid
sequential delegation and illegal concurrent aliasing.

---

## 1. The Core Problem: Why Reborrowing Exists

In Rust, references have two core rules:

1. Shared references (`&T`) implement `Copy`. You can duplicate them freely.
2. Mutable references (`&mut T`) do **not** implement `Copy`. They represent exclusive
   access.

Under normal move semantics, any non-`Copy` value is consumed when passed to a function:

```rust
// Hypothetical Rust without reborrowing:
fn add_score(&mut self) {
    self.log_score(); // If this MOVED `self`, `self` would be destroyed!
    self.score += 1;  // ERROR: use of moved value `self`
}
```

If passing `&mut self` moved the reference, mutable references would be single-use tokens.
You would have to return `&mut self` from every helper method just to keep using it.

To solve this, Rust provides **reborrowing**.

---

## 2. What is a Reborrow?

A reborrow creates a new, temporary reference from an existing reference by dereferencing
it and borrowing the target again:

```rust
&mut *original_reference
```

Whenever you pass an `&mut T` to a function expecting `&mut T`, or invoke a method taking
`&mut self`, the compiler automatically inserts this reborrow for you.

### Key Mechanics:

- **Child Borrow Creation**: The compiler spawns a new reference with a shorter lifetime
  (sub-lifetime).
- **Parent Suspension**: The original parent reference is temporarily frozen and cannot be
  used while the child reference is active.
- **Parent Restoration**: When the child reference finishes (e.g., when the called
  function returns), the child is destroyed and the parent reference wakes back up with
  full exclusive permissions.

---

## 3. The "Shadow Variable" Analogy

Reborrowing behaves very much like block-scoped **variable shadowing**:

| Concept           | Variable Shadowing (`{ let x = ...; }`)                                            | Reborrowing (`&mut *parent`)                                                                          |
| :---------------- | :--------------------------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------- |
| **Lifespan**      | Inner variable shadows the outer variable for a shorter duration.                  | Child reference lives only for the duration of the inner block or function call.                      |
| **Accessibility** | Outer variable cannot be accessed directly while shadowed.                         | Parent reference is suspended (dormant) while the child borrow is active.                             |
| **Restoration**   | When the inner scope exits, the outer variable is accessible again.                | When the child reference ends, the parent reference is restored.                                      |
| **Memory Target** | **Different memory**: Modifying the inner binding does not change the outer value. | **Same memory**: Any mutation made through the child reference persists and is visible to the parent. |

---

## 4. Does Reborrowing Apply Only to `&mut T`?

No, but it is indispensable for `&mut T`:

- **Shared References (`&T`)**: Reborrowing (`&*shared`) occurs during lifetime subtyping
  or `Deref` coercions (such as converting `&Vec<T>` to `&[T]`), though simple assignment
  can also just `Copy` the reference.
- **Mutable References (`&mut T`)**: Reborrowing is strictly mandatory to prevent moves.
- **Cross-Type Reborrowing (Downgrading)**: You can reborrow an `&mut T` as a temporary
  `&T` (`&*mut_ref`). This freezes the mutable reference into read-only mode until all
  shared readers finish.

---

## 5. Walkthrough of the Examples

### Example 1: [src/valid.rs](src/valid.rs) (Sequential Reborrowing)

This example demonstrates calling an `&mut self` helper method from inside an `&mut self`
method:

```rust
impl Player {
    pub fn add_score(&mut self, points: i32) {
        self.score += points;
        self.log_score(); // Perfectly legal!
    }

    pub fn log_score(&mut self) {
        println!("Score is now: {}", self.score);
    }
}
```

#### What this accomplishes:

- Demonstrates that `self.log_score()` desugars to `Player::log_score(&mut *self)`.
- Proves that multiple `&mut self` methods can call one another sequentially without
  borrow errors.
- Explains why the **call stack** and **reborrowing** work together: while `log_score`
  runs on the stack, `add_score` is paused. At no point in time do two references access
  the underlying memory simultaneously.

---

### Example 2: [src/invalid.rs](src/invalid.rs) (Concurrent Aliasing Violations)

This example demonstrates the exact boundary where the borrow checker steps in to prevent
illegal aliasing.

```rust
// Attempt 1: Passing two &mut to the same function simultaneously
bar(
    &mut c,
    &mut c // COMPILER ERROR: cannot borrow `c` as mutable more than once at a time
);

// Attempt 2: Creating overlapping mutable references in the same scope
let r1 = &mut c;
let r2 = &mut c; // COMPILER ERROR: cannot borrow `c` as mutable more than once at a time
r2.inc();
r1.inc(); // r1 is used here, so r1 was active when r2 was created
```

#### What this accomplishes:

- **Demonstrates true concurrent aliasing**: In both cases, two mutable references would
  exist in the same execution frame with overlapping lifetimes, allowing simultaneous
  read/write paths to the same memory.
- **Contrasts with reborrowing**: In `valid.rs`, the child borrow suspends the parent. In
  `invalid.rs`, both borrows attempt to remain active and usable in the exact same scope,
  violating Rust's fundamental invariant: **Aliasing XOR Mutability**.

---

## 6. Summary Cheat Sheet

| Pattern                    | Code Pattern                                     | Compiler Status | Explanation                                                         |
| :------------------------- | :----------------------------------------------- | :-------------- | :------------------------------------------------------------------ |
| **Reborrow (Method Call)** | `self.helper()` inside `&mut self`               | **Allowed**     | Child borrow suspends parent for the duration of the call.          |
| **Reborrow (Block Scope)** | `{ let r2 = &mut *r1; }`                         | **Allowed**     | `r1` is suspended while `r2` is used; `r1` resumes after `r2` ends. |
| **Simultaneous Arguments** | `foo(&mut x, &mut x)`                            | **Rejected**    | Both references would be active in the callee at the same time.     |
| **Overlapping Borrows**    | `let r1 = &mut x; let r2 = &mut x; use(r1, r2);` | **Rejected**    | Two active mutable references in the same scope.                    |
