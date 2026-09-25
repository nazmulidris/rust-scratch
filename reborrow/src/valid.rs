// =============================================================================
// Conceptual Model: Reborrowing as a "Shadow Variable"
// =============================================================================
//
// Think of reborrowing as a specialized form of variable shadowing.
//
// 1. Shorter Lifespan:
//    When you enter an inner block with `{ let x = ...; }`, the inner variable
//    shadows the outer one for a shorter duration. Similarly, a reborrow creates
//    a short-lived child reference (`&mut *parent`) that only lives as long as
//    the callee function runs.
//
// 2. Dormancy / Inaccessibility:
//    While shadowed, the outer variable cannot be directly named or accessed.
//    Likewise, while a child reborrow is active, the parent reference is frozen
//    and inaccessible in the caller.
//
// 3. Restoration:
//    When the inner scope finishes, the outer variable becomes available again.
//    Likewise, when the child reborrow finishes, the parent reference emerges
//    with full exclusive access restored.
//
// 4. The Key Difference from Variable Shadowing:
//    Ordinary shadowing binds a completely separate value in a new memory slot.
//    A reborrow delegates access to the exact same memory location. Any mutation
//    performed through the child reference persists and is seen by the parent
//    reference once it wakes back up.
//
// 5. What would happen WITHOUT reborrowing?
//    Because `&mut T` does NOT implement `Copy`, passing `self` to another method
//    would be a permanent MOVE. The caller would lose `self` forever upon calling
//    any helper method, resulting in `error[E0382]: use of moved value: self`.
//    Without reborrowing, `&mut T` would effectively be a single-use token.
//
// 6. Is reborrowing only for mutable references (`&mut T`)?
//    No, but it is indispensable for them:
//    - For shared references (`&T`), they already implement `Copy`, so ordinary
//      assignment freely duplicates them. However, reborrowing (`&*shared`) still
//      occurs to shorten lifetimes (subtyping) or trigger Deref coercions
//      (such as converting `&Vec<T>` to `&[T]`).
//    - For mutable references (`&mut T`), reborrowing is strictly mandatory
//      because `&mut T` cannot be copied.
//    - Cross-type reborrowing (downgrading) also exists: reborrowing an `&mut T`
//      as a temporary `&T` (`&*mut_ref`), which freezes the mutable reference
//      until all shared readers finish.
// =============================================================================

#[derive(Debug, Default)]
pub struct Player {
    score: u8,
}

impl Player {
    pub fn add_score(&mut self, points: u8) {
        self.score += points;

        // Calling another `&mut self` method is completely legal due to REBORROWING.
        //
        // What happens behind the scenes:
        //
        // 1. Desugaring:
        //    Rust desugars `self.print_score()` into `Player::print_score(&mut *self)`.
        //
        // 2. Child borrow creation:
        //    Instead of moving `self` (which would consume it permanently), Rust creates
        //    a temporary, short-lived child reference `&mut *self` with a shorter lifetime.
        //
        // 3. Parent borrow suspension:
        //    While `print_score()` executes, the parent reference `self` in `add_score()`
        //    is temporarily suspended (like a shadow binding). It cannot be used directly
        //    until the child borrow ends.
        //
        // 4. Aliasing guarantee preserved:
        //    Because `add_score()` is paused on the call stack waiting for `print_score()`
        //    to return, at no point do two references touch the memory simultaneously.
        //    Aliasing XOR Mutability is never violated.
        //
        // 5. Parent borrow restoration:
        //    As soon as `print_score()` returns, the child borrow is destroyed. The parent
        //    reference `self` wakes up and regains full exclusive mutable access.
        self.print_score();

        // `self` is restored and fully usable again here.
        // For example, `self.score += 1;` is completely valid here.
    }

    pub fn print_score(&mut self) {
        println!("Score is now: {:#?}", self.score);
    }
}

pub fn run() {
    // `p` must be declared `mut` to allow calling methods that require `&mut self`.
    let mut p = Player { score: 0 };
    println!("Initial: {p:?}.");

    // Calling `add_score` initiates the outer mutable borrow.
    p.add_score(10);
    println!("Updated: {p:?}.");
}

