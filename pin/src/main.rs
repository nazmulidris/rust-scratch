/*
 *   Copyright (c) 2024 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

/// More info:
/// - [Jon's video on pin and async](https://www.youtube.com/watch?v=DkMwYxfSYNQ)
/// - [Pin docs](https://doc.rust-lang.org/std/pin/index.html#projections-and-structural-pinning)
/// - [pin-project crate to make this easy](https://docs.rs/pin-project/latest/pin_project/)
///   - [`#[pin_project`](https://docs.rs/pin-project/latest/pin_project/attr.pin_project.html)
/// - [Async Rust book](https://rust-lang.github.io/async-book/)
fn main() {
    not_pinned_example();
    pinned_example();
    unmovable_buffer_example();
    use_pin_project();
}

#[derive(PartialEq, Debug)]
enum Location {
    Moved,
    NotMoved,
}

fn not_pinned_example() {
    #[derive(Default)]
    struct AddrTracker {
        previous_address: Option<usize>,
    }

    impl AddrTracker {
        // If we haven't checked the addr of self yet, store the current
        // address. If we have, confirm that the current address is the same
        // as it was last time, or else panic.
        fn check_for_move_out_of_previous_address(&mut self) -> Location {
            /* `self` is the current_pointer */
            let current_pointee = self as *mut Self;
            let current_addr = current_pointee as usize;
            match self.previous_address {
                None => {
                    self.previous_address = Some(current_addr);
                    Location::NotMoved
                }
                Some(prev_addr) => match prev_addr == current_addr {
                    true => Location::NotMoved,
                    false => Location::Moved,
                },
            }
        }
    }

    // Create a tracker. This is not yet in an address sensitive state.
    let mut tracker = AddrTracker::default();

    assert_eq!(
        // This will cause tracker to store the initial address.
        tracker.check_for_move_out_of_previous_address(),
        Location::NotMoved
    );

    // Here we move the variable. This carries a semantic move, and may therefore also
    // come with a mechanical memory *move*.
    let mut tracker2 = tracker;

    // Has moved!
    assert_eq!(
        tracker2.check_for_move_out_of_previous_address(),
        Location::Moved
    );
}

fn pinned_example() {
    use std::marker::PhantomPinned;
    use std::pin::{pin, Pin};

    #[derive(Default)]
    struct AddrTracker {
        previous_address: Option<usize>,

        /// Remove auto-implemented `Unpin` bound to mark this type as having some
        /// address-sensitive state. This is essential for our expected pinning
        /// guarantees to work, and is discussed more below.
        _pin: PhantomPinned,
    }

    impl AddrTracker {
        fn check_for_move_out_of_previous_address(self: Pin<&mut Self>) -> Location {
            let current_pointee_reference = &*self;
            let as_constant_pointer = current_pointee_reference as *const Self;
            let current_address = as_constant_pointer as usize;

            match self.previous_address {
                Some(previous_address) => match previous_address == current_address {
                    true => Location::NotMoved,
                    false => Location::Moved,
                },
                None => {
                    // Safety: we do not move out of `self`.
                    // This won't work: `self.get_mut().previous_address = Some(current_address);`
                    let self_data_mut = unsafe { self.get_unchecked_mut() };
                    self_data_mut.previous_address = Some(current_address);
                    Location::NotMoved
                }
            }
        }
    }

    // 1. Create the value, not yet in an address-sensitive state.
    let tracker = AddrTracker::default();

    // 2. Pin the value. Put it behind a pinning pointer, thus putting it into an
    //    address-sensitive state.
    let mut ptr_to_pinned_tracker: Pin<&mut AddrTracker> = pin!(tracker);
    assert_eq!(
        ptr_to_pinned_tracker
            .as_mut()
            .check_for_move_out_of_previous_address(),
        Location::NotMoved
    );

    // Trying to access `tracker` or pass `ptr_to_pinned_tracker` to anything that
    // requires mutable access to a non-pinned version of it will no longer compile.

    // 3. We can now assume that the tracker value will never be moved.
    assert_eq!(
        ptr_to_pinned_tracker
            .as_mut()
            .check_for_move_out_of_previous_address(),
        Location::NotMoved
    );
}

fn unmovable_buffer_example() {
    use std::marker::PhantomPinned;
    use std::pin::Pin;
    use std::ptr::NonNull;

    /// This is a self-referential struct because `self.slice` points into `self.data`.
    struct Unmovable {
        /// Backing buffer.
        data: [u8; 64],
        /// Points at `self.data` which we know is itself non-null. Raw pointer because we
        /// can't do this with a normal reference.
        slice: NonNull<[u8]>,
        /// Suppress `Unpin` so that this cannot be moved out of a `Pin` once constructed.
        _pin: PhantomPinned,
    }

    impl Unmovable {
        /// Create a new `Unmovable`.
        ///
        /// To ensure the data doesn't move we place it on the heap behind a pinning Box.
        /// Note that the data is pinned, but the `Pin<Box<Self>>` which is pinning it can
        /// itself still be moved. This is important because it means we can return the
        /// pinning pointer from the function, which is itself a kind of move!
        fn new() -> Pin<Box<Self>> {
            let res = Unmovable {
                data: [0; 64],
                // We only create the pointer once the data is in place otherwise it will
                // have already moved before we even started.
                slice: NonNull::from(&[]),
                _pin: PhantomPinned,
            };
            // First we put the data in a box, which will be its final resting place
            let mut boxed = Box::new(res);

            // Then we make the slice field point to the proper part of that boxed data.
            // From now on we need to make sure we don't move the boxed data.
            boxed.slice = NonNull::from(&boxed.data);

            // To do that, we pin the data in place by pointing to it with a pinning
            // (`Pin`-wrapped) pointer.
            //
            // `Box::into_pin` makes existing `Box` pin the data in-place without moving
            // it, so we can safely do this now *after* inserting the slice pointer above,
            // but we have to take care that we haven't performed any other semantic moves
            // of `res` in between.
            let pin = Box::into_pin(boxed);

            println!("Unmovable created at {:p}", pin);

            // Now we can return the pinned (through a pinning Box) data.
            pin
        }
    }

    let unmovable: Pin<Box<Unmovable>> = Unmovable::new();

    // The inner pointee `Unmovable` struct will now never be allowed to move. Meanwhile,
    // we are free to move the pointer around.
    let still_unmoved = unmovable;
    assert_eq!(still_unmoved.slice, NonNull::from(&still_unmoved.data));

    // We cannot mutably dereference a `Pin<Ptr>` unless the pointee is `Unpin` or we use
    // unsafe. Since our type doesn't implement `Unpin`, this will fail to compile:
    // let mut new_unmoved = Unmovable::new();
    // std::mem::swap(&mut *still_unmoved, &mut *new_unmoved);
}

fn use_pin_project() {
    use pin_project::pin_project;
    use std::pin::pin;
    use std::pin::Pin;

    #[pin_project]
    struct Struct {
        /// This field will be pinned in place.
        #[pin]
        pinned_field: String,

        /// This is a regular field.
        unpinned_field: String,

        /// This is the address.
        unpinned_address: Option<usize>,
    }

    impl Struct {
        fn modify_fields(self: Pin<&mut Self>) {
            // Calculate the address of the current pointee.
            let current_address = {
                let current_pointee_reference = &*self;
                let as_constant_pointer = current_pointee_reference as *const Self;
                as_constant_pointer as usize
            };

            // `project()` consumes `self`.
            let this = self.project();

            // Modify the unpinned field.
            let unpinned_field = this.unpinned_field;
            unpinned_field.push_str("hello");

            // Modify the pinned field.
            let mut pinned_field = this.pinned_field;
            pinned_field.as_mut().push_str("world");

            // Set the unpinned address field of the underlying pinned pointee. If moved,
            // then panic!().
            match this.unpinned_address {
                // This will panic! if the the address of the pointee has changed.
                Some(previous_address) => {
                    assert_eq!(*previous_address, current_address);
                }
                // No previous address, so set it.
                None => {
                    this.unpinned_address.replace(current_address);
                }
            }
        }
    }

    let mut my_pinned_struct = pin!(Struct {
        pinned_field: String::new(),
        unpinned_field: String::new(),
        unpinned_address: None
    });

    my_pinned_struct.as_mut().modify_fields();
    println!("{:?}", my_pinned_struct.pinned_field);
    println!("{:?}", my_pinned_struct.unpinned_field);
    println!("{:?}", my_pinned_struct.unpinned_address);

    let my_pin2 = my_pinned_struct;
    println!("{:?}", my_pin2.pinned_field);
    println!("{:?}", my_pin2.unpinned_field);
    println!("{:?}", my_pin2.unpinned_address);
}
