// Copyright (c) 2026 Nazmul Idris. Licensed under Apache License, Version 2.0.

# Video Title: Rust Arrays, Slices, and hardware

## Introduction (The Hook)

_[Visual: Your face/camera]_

When we write Rust, we usually think about types, traits, and the borrow checker. We think
about memory as this infinite, flat tape measure where we can grab any single byte we
want. But the silicon underneath doesn't work like that at all.

Today, we are going deep. We're going to look at how the physical realities of your CPU,
the 64-bit data bus, and RAM cache lines directly dictate how your Rust code runs. We're
going to bridge the gap between software and physical hardware, and I'll show you how to
write Rust that works _with_ the silicon—a concept called Mechanical Sympathy.

---

## Setup (Prepare the Workspace)

_[Action: Run the following in a terminal]_

```bash
cd $(mktemp -d)
cargo new --bin arrays-slices-hardware
mkdir -p src/bin/
mv src/main.rs src/bin/ex_1.rs
code-insiders -n arrays-slices-hardware
```

Open `src/bin/ex_1.rs`.

---

## Scene 1: The Software Illusion (Arrays vs Slices)

_[Visual: Switch to IDE / Terminal]_

Let's start with the most basic data structure: the array. The software illusion is that
an array is just a list of items. But in Rust, the 'Size' of an array is physical.

- An Array `[u64; 1000]` is just 8,000 contiguous bytes. There is no 'length' field stored
  anywhere in RAM. The compiler simply 'knows' it's 1000 items and bakes that number
  directly into the machine code's loops and offsets. It is 'Metadata-less' at runtime.
- A Slice `&[u64]` is a Wide Pointer. It is 16 bytes on the stack: 8 bytes for the memory
  address and 8 bytes for the length. It carries its metadata with it at runtime.

_[Action: Live code the following]_

Create a new file `src/bin/ex_1.rs`.

```rust
fn main() {
    let my_data = [0u64; 1000];
    println!("Physical size of array: {} bytes", std::mem::size_of_val(&my_data));

    let my_slice: &[u64] = &my_data;
    println!("Physical size of slice: {} bytes", std::mem::size_of_val(&my_slice));
}
```

_[Action: Run the code]_

Run the code using `cargo run --bin ex_1`. Observe the 8000 vs 16 output.

Look at this output. When we define `[u64; 1000]`, there is no pointer. It is literally
8,000 continuous bytes. But the slice `&[u64]` is only 16 bytes. By using slices, we pass
a 16-byte manifest (the address and length) instead of copying the entire 8,000-byte
payload.

Crucially, in both cases, the size of the individual element (`u64` = 8 bytes) is never
stored in memory. The compiler calculates the offsets and bakes the '8-byte' math directly
into the CPU instructions. But where do these 8,000 bytes and 16 bytes actually live? For
that, we need to look at the physical truth.

---

## Scene 2: The Physical Truth (The Stack and the Pointer)

_[Action: Live code the following]_

Create a new file `src/bin/ex_2.rs`.

We see the size difference, but what does this actually look like in the silicon? Let's
prove that passing an array by value is a literal physical copy, and then we'll
deconstruct that 16-byte wide pointer.

```rust
fn main() {
    let my_data = [10, 20, 30, 40];
    println!("Array OUTSIDE function address: {:p}", &my_data);

    process_array(my_data);
    process_slice(&my_data);
}

fn process_array(arr: [u64; 4]) {
    println!("Array INSIDE function address:  {:p}", &arr);
}

fn process_slice(slice: &[u64]) {
    // {:p} on a slice shows the address it points TO
    println!("Slice POINTING to address:      {:p}", slice);

    // Deconstruct the 16-byte 'Wide Pointer'
    // into its two physical components (Address and Length)
    let (address, length): (usize, usize) = unsafe {
        std::mem::transmute(slice)
    };
    println!("Wide Pointer HEX: [Addr: 0x{:x}, Len: 0x{:x}]", address, length);
}
```

_[Action: Run the code]_

Run the code using `cargo run --bin ex_2`. Observe the Hex addresses.

Look at the output. The address inside `process_array` is different! Why? Because the
Stack is a physical region of memory.

_[The Mechanics: Deconstructing the Wide Pointer & The Stack]_

1.  The Stack Frame: Every local variable and return value slot is pre-calculated at
    compile time. When a function starts, it emits a single CPU instruction to move the
    Stack Pointer (`rsp`) down by that exact size. This 'carves out' a private buffer in
    RAM for that function.
2.  The Physical Copy: When we passed `my_data` by value, the CPU had to `memcpy` those 32
    bytes into the new stack frame. This is why the address changed—it points to a new
    part of silicon on a RAM chip!
3.  Physical Limits: The stack is not infinite. On Linux, you can see your process limit
    by running `ulimit -s` (usually 8MB).
4.  Thread Stacks: Each thread gets its own independent stack. In Rust, you can customize
    this using `std::thread::Builder::new().stack_size(...)`. You can also set the
    RUST_MIN_STACK environment variable globally.
5.  No Quota: Thread stacks are not carved out of the 8MB process limit; they are separate
    allocations in virtual memory.
6.  Transmute Trick: We used `std::mem::transmute` to 'peek' behind the curtain. It
    doesn't move or change a single bit in memory; it only changes the mental model the
    compiler uses. We took the 16 bytes of the slice and told the compiler to show them as
    two `usize` integers.

_[The Foundation: Why 64-bit? (The Pointer Limit)]_

Why is a pointer 8 bytes? Because on my Intel(R) Core(TM) i7-14700, the Registers are
exactly 64 bits wide.

_[What is a Register?]_

To understand Mechanical Sympathy, think of your CPU as the Porsche Factory in Flacht,
where they build GT3s.

- The Warehouse (RAM): This is where all your car parts (data) are stored in massive
  storage bins. But you cannot build a GT3 engine inside the warehouse.
- The Logistics Train (Data Bus): Parts must be pulled from the warehouse, loaded onto a
  logistics train (the physical wires), and brought onto the factory floor.
- The Assembly Station (Registers): This is the destination—the actual workbench where the
  technician assembles the engine. The CPU can only perform math and logic here. On my
  i7-14700, this workbench is exactly 64 bits (8 bytes) wide. Every 'kit' of parts you
  work on must fit into that 64-bit workbench slot.

_[Deep Dive: The 'How do we know the address?' Paradox]_

You might wonder: if the address is baked into the code, how does it run on different
machines with different RAM layouts?

1.  The Binary and the Target Triple: The compiler emits an ELF (Executable and Linkable
    Format) on Linux. This binary is physically tied to a Target Triple (e.g.,
    `x86_64-unknown-linux-gnu`).
2.  The OS Loader: When you run your program, the OS 'Loader' reads this binary and carves
    out a Process—a private, virtual container. The OS maps the binary's code sections
    into RAM and sets up the Stack and Heap.
3.  Relative Offsets: The compiler uses Stack-Relative Addressing. It says: 'The array
    starts 32 bytes away from the current Stack Pointer (rsp)'.
4.  Virtual Memory: Every process lives in an illusion created by the MMU (Memory
    Management Unit). This is a physical piece of CPU Hardware that works with the OS
    Kernel's Page Tables to translate Virtual Addresses to Physical RAM Slots.

---

## Scene 3: The Return Journey and the Pin Problem

_[Action: Live code the following]_

Create a new file `src/bin/ex_3.rs`.

What about returning values? If I return a large array, does it move? And what happens to
pointers pointing _at_ that data?

```rust
fn create_array() -> [u64; 2] {
    let retval = [1, 2];
    println!("Inside create_array(): {:p}", &retval);
    retval
}

fn main() {
    let data = create_array();
    println!("Inside main:           {:p}", &data);

    let final_move = data;
    println!("After final move:    {:p}", &final_move);
}
```

_[Action: Run the code]_

Run the code using `cargo run --bin ex_3`. Compare the addresses.

Wait, the address of `retval` inside `create_array()` and `data` in `main` are the same!
This is the compiler being smart. But look at `final_move`—the address physically changed
because it moved to a new spot on the stack!

_[The Mechanics: Return Value Optimization (RVO)]_

When a function returns a large array, the caller carves out space on its stack _before_
the call. It passes a hidden 'secret pointer' to the function. The function then writes
directly into the caller's memory. This is why the address of `retval` and `data` is
identical—the compiler skipped the copy!

_[The Problem: The Pin Paradox]_

But as soon as we did `let final_move = data`, the address changed. Now imagine a
Self-Referential Struct: a struct where Field B is a pointer to Field A. If you 'move'
that struct (like we did with `final_move`), Field B is now pointing to a dead address on
the old stack frame. This is why `Pin` exists. `Pin` is a physical contract that says:
'This data will never change its memory address again.'

[Deep Dive: Async/Await and the State Machine] Why does `async/await` care about this?

1.  _Task_: When you run your async function or async block in an async runtime like
    `tokio`, it is wrapped in a `Task` (a `tokio` type that manages `Future`s).
2.  _Desugaring_: To make this work, the compiler desugars your async function or async
    block into a normal Rust struct that implements the `Future` trait. This `Future`
    struct is the State Machine.
3.  _Persistence_: When the expression (in your code) that ends in `.await` is executed,
    your async function or async block pauses and yields control back to the executor. The
    `Future` struct becomes the permanent home for your local variables so they can
    persist while the async function or async block is paused.
4.  _Resume_: When your code hits `.await`, the `Future` is polled immediately. If the
    resource isn't ready, it returns `Pending` and selects the appropriate event source
    (like the Reactor or a Timer) to register its `Waker` with.
5.  _Parked Task_: While waiting, your `Task` is effectively Parked. It is removed from
    the executor's 'Run Queue' and sits dormant in RAM. The technician (CPU) has moved to
    another assembly station to work on a different car.
6.  _Reschedule_: Once a resource is ready, something must trigger the `Waker` (the
    Reactor for I/O, the Timer Wheel for sleep, or Internal State for channels). This
    tells the executor to un-park your `Task` and put it back on the queue to be polled
    again.
7.  _Self-Reference_: Your local variables don't move back to the stack; they stay put in
    the `Future` struct. The CPU uses the `self` pointer to continue work exactly where it
    left off. If one variable is a pointer to another, the struct is Self-Referential.
8.  _Safety_: If this `Future` moves in memory, the internal pointers break. `Pin` is the
    physical contract that says this move will never happen, keeping your async function
    or async block safe.

---

## Scene 4: Hardware Alignment and The Memory Holes

_[Action: Live code the following]_

Create a new file `src/bin/ex_4.rs`.

The CPU's 'logistics train' (the data bus) is designed for 8-byte standardized crates. It
doesn't move loose items; it only moves these crates. Let's prove the compiler is forced
to leave 'dead space' in a crate to ensure that large parts (like a `u64` engine block)
never get split across two different shipments.

```rust
use std::mem::{size_of, align_of};

#[repr(C)]
struct Hole {
    a: u8,   // 1 byte
    b: u64,  // 8 bytes
}

fn main() {
    let s = Hole { a: 1, b: 2 };
    println!("Alignment of Hole: {} bytes", align_of::<Hole>());
    println!("Physical Size:      {} bytes", size_of::<Hole>());

    println!("Address of a: {:p}", &s.a);
    println!("Address of b: {:p}", &s.b);
}
```

_[Action: Run the code]_

Run the code using `cargo run --bin ex_4`. Observe the 7-byte gap.

Look at those Hex addresses. `a` is at `...00`, but `b` starts at `...08`. There are 7
bytes of 'dead air' in between!

[The Physics: Natural Alignment] A primitive of size $N$ must live at an address where
`Address % N == 0`. Because `b` is 8 bytes, the CPU refuses to load it unless it's on an
8-byte boundary. The compiler silently injected Padding bytes to satisfy the silicon.

_[The Consequence: Cache Line Density]_

Why do we care? Because RAM moves in 64-byte Cache Lines.

- If your struct is 24 bytes (with padding), you fit 2.6 items in one 64-byte fetch.
- If you align it to 16 bytes, you fit 4 items. Padding is literal Cache Pollution. You
  are paying for electricity and bandwidth to move 'nothing.' By shrinking your structs,
  you increase your effective RAM bandwidth.

---

## Scene 5: The Prefetcher (AoS vs SoA)

_[Action: Live code the following]_

Create a new file `src/bin/ex_5.rs`.

Finally, let's look at the The Hardware Prefetcher. It watches your pattern and 'guesses'
what you want next.

```rust
use std::time::Instant;

struct Entity {
    pos_x: f32, // 4 bytes
    pos_y: f32, // 4 bytes
    _pollute: [u8; 56], // Imagine 56 bytes of health, name, etc.
}

struct Entities {
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,
}

fn main() {
    const COUNT: usize = 10_000_000;

    // AoS: Array of Structures
    let mut aos = Vec::with_capacity(COUNT);
    for _ in 0..COUNT {
        aos.push(Entity { pos_x: 0.0, pos_y: 0.0, _pollute: [0; 56] });
    }

    // SoA: Structure of Arrays
    let mut soa = Entities {
        pos_x: vec![0.0; COUNT],
        pos_y: vec![0.0; COUNT],
    };

    // Benchmark AoS
    let start = Instant::now();
    for entity in aos.iter_mut() {
        entity.pos_x += 1.0;
    }
    println!("AoS update: {:?}", start.elapsed());

    // Benchmark SoA
    let start = Instant::now();
    for x in soa.pos_x.iter_mut() {
        *x += 1.0;
    }
    println!("SoA update: {:?}", start.elapsed());
}
```

_[Action: Run the code]_

Run the code using `cargo run --bin ex_5`. You will see two durations printed. Compare
them: the SoA (Structure of Arrays) update is significantly faster (often 2x to 5x) than
the AoS (Array of Structures) version.

This is because in the AoS version, when you iterate over the data to update `pos_x`, the
CPU pulls a 64-byte line to get that single float, but it's forced to bring along 56 bytes
of 'pollution' (the other fields in the struct). You are wasting 93% of your memory
bandwidth.

_[The Mechanics: Spatial Locality]_

In the SoA version, one 64-byte bucket contains sixteen contiguous `f32` positions. Every
bit is 'hot' data. The prefetcher sees this perfect `+4, +4, +4` pattern and 'streams' the
data into L1 cache before the CPU even asks. This is Mechanical Sympathy: designing your
data to flow through the hardware's existing pipes.

---

## Conclusion

_[Visual: Back to face/camera]_

Rust gives us high-level ergonomics, but it never hides the machine. When you understand
the stack, alignment, and cache lines, you stop just writing code and start engineering
silicon.

Thanks for watching.
