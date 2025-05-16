# memory_locality_latency

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Rest In Peace, Moore's Law](#rest-in-peace-moores-law)
- [Memory latency and cache lines](#memory-latency-and-cache-lines)
  - [Order of magnitude latency differences](#order-of-magnitude-latency-differences)
  - [Cache line](#cache-line)
- [Stack vs heap](#stack-vs-heap)
  - [Stack memory](#stack-memory)
  - [Heap memory](#heap-memory)
  - [Why stack access is often faster](#why-stack-access-is-often-faster)
  - [Practical implications in Rust and Linux](#practical-implications-in-rust-and-linux)
  - [Stack size in Ubuntu 25.04](#stack-size-in-ubuntu-2504)
  - [Allocation and drop](#allocation-and-drop)
  - [Heap memory example, String and string slice](#heap-memory-example-string-and-string-slice)
- [Memory alignment](#memory-alignment)
- [Global allocators](#global-allocators)
- [Using arrays for stack or heap allocation](#using-arrays-for-stack-or-heap-allocation)
- [Using smallvec and smallstr crates](#using-smallvec-and-smallstr-crates)
  - [smallvec](#smallvec)
  - [smallstr](#smallstr)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Rest In Peace, Moore's Law

Moore’s Law—the observation that the number of transistors on a chip doubles roughly every
two years—has slowed significantly in recent years. While there’s no single official “end”
date, most experts agree that the exponential pace described by Moore’s Law effectively
ended around 2015–2020. Modern process nodes (e.g., 7nm, 5nm, 3nm) are much harder and
more expensive to shrink further, and improvements now take longer than two years.

Implications:

- This means CPU bound code that runs slowly will run slowly on future CPUs.
- The traditional approach of Big-O analysis and algorithmic improvements is not accurate,
  since it only accounts for "op-count" and totally ignores the cost of memory access.
- Modern CPUs have many cores, but each core isn't getting any faster. So using
  parallelism is another way to get more performance.

Here are some great resources to understand the implication of the end of Moore's Law on
software performance.

- [New CPUs don't speed up old code](https://youtu.be/m7PVZixO35c)
- [Data oriented design (YouTube)](https://youtu.be/WwkuAqObplU)
  - "Flat" data structures are better for memory locality.
  - Pointer jumps are expensive and can slow down access (e.g., a Unicode string that
    doesn't own its data and fetches from another slice is slower due to poor locality).
- [Memory Latency vs CPU operation (YouTube)](https://youtu.be/Dhn-JgZaBWo)
- [Memory Allocation Tips (YouTube)](https://youtu.be/pJ-FRRB5E84&t=1831)
  - For `Vec` and `String`, use `.with_capacity()` to pre-allocate memory and reduce
    reallocations.
  - Consider using these crates for performance improvements:
    - [`smallvec`](https://docs.rs/smallvec/latest/smallvec/struct.SmallVec.html) (part of
      Servo): Store small numbers of elements on the stack.
    - [`smallstr`](https://docs.rs/smallstr/0.3.0/smallstr/) (based on smallvec): Store
      small strings on the stack.

## Memory latency and cache lines

The CPU's cache hierarchy (L1, L2, and often L3) acts as a crucial intermediary, bridging
the massive performance gap between the incredibly fast CPU registers and the much slower
main memory and storage. Cache lines are the fundamental building blocks that enable this
efficient data movement and significantly impact overall system performance.

### Order of magnitude latency differences

The relative latencies look something like this (very approximate) for a machine with
Intel 14th Gen, DDR5-5200, PCIe 4 SSD:

| Memory Type                  | Relative Latency (vs. Register) |
| ---------------------------- | ------------------------------- |
| Register                     | 1x                              |
| L1 Cache                     | 10x - 50x                       |
| L2 Cache                     | 50x - 200x                      |
| RAM                          | 500x - 1000x                    |
| GPU HBM/GDDR6                | 1,000x - 2,000x                 |
| SSD                          | 10,000x - 100,000x              |
| Local Network (LAN)          | 100,000x - 1,000,000x           |
| Internet (Same Region)       | 1,000,000x - 10,000,000x        |
| Internet (Cross-Continental) | 10,000,000x - 100,000,000x      |

![Memory Latency](memory_latency.svg)

It's important to understand that these are _relative_ order-of-magnitude estimates and
can vary based on specific workloads, system configurations, and the exact characteristics
of the components. However, they provide a good sense of the performance hierarchy:

1.  **Fetching Data from a Register:** This is the fastest operation, happening within the
    CPU core itself.

    - **Latency:** Sub-nanosecond (on the order of picoseconds). Let's consider this as
      **~1 unit of time**.

2.  **Fetching Data from L1 Cache:** The L1 cache is the smallest and closest cache to the
    CPU cores.

    - **Latency:** Around 1-5 nanoseconds. This is still incredibly fast, but noticeably
      slower than register access.
    - **Relative Order of Magnitude:** **~10x - 50x** slower than register access.

3.  **Fetching Data from L2 Cache:** The L2 cache is larger and slightly further away than
    L1.

    - **Latency:** Around 5-20 nanoseconds.
    - **Relative Order of Magnitude:** **~50x - 200x** slower than register access.

4.  **Fetching Data from Main Memory (DDR5-5200):** Accessing RAM is significantly slower
    than cache access. DDR5-5200 specifies the data transfer rate, but the actual latency
    to fetch data involves factors like CAS latency and command cycles.

    - **Latency:** Around 50-100 nanoseconds (or even higher depending on the specific
      timings and system load).
    - **Relative Order of Magnitude:** **~500x - 1000x** slower than register access.

5.  **Fetching Data from SSD (PCIe 4):** Accessing an SSD is orders of magnitude slower
    than RAM, although much faster than traditional hard drives. PCIe 4 offers high
    bandwidth, but the latency for a random access is still considerable in CPU time
    scales.
    - **Latency:** Tens to hundreds of _microseconds_ (thousands to hundreds of thousands
      of nanoseconds). Let's say around 10-100 microseconds for a typical random read.
    - **Relative Order of Magnitude:** **~10,000x - 100,000x** slower than register
      access.

> | Unit Name   | Symbol | Value in Seconds                  |
> | ----------- | ------ | --------------------------------- |
> | second      | s      | 1                                 |
> | millisecond | ms     | 1/1,000 = 10⁻³ s                  |
> | microsecond | μs     | 1/1,000,000 = 10⁻⁶ s              |
> | nanosecond  | ns     | 1/1,000,000,000 = 10⁻⁹ s          |
> | picosecond  | ps     | 1/1,000,000,000,000 = 10⁻¹² s     |
> | femtosecond | fs     | 1/1,000,000,000,000,000 = 10⁻¹⁵ s |

### Cache line

A **cache line** is the fundamental unit of data transfer between the CPU's cache
hierarchy and the main memory. On the 14th gen Intel CPUs it is 64 bytes, its primary
function is to enhance performance by fetching and storing data in larger blocks, thereby
reducing the frequency of slower main memory accesses. It is 128 bytes or twice as large
on an Apple M4 chip.

All CPU accesses to RAM are done in cache line units, even if the CPU only needs a single
byte or word (e.g., `usize`, or 64 bits / 8 bytes on 14th gen Intel CPU). The cache line
is the fundamental unit of data transfer between RAM and the CPU cache.

**How it works:** When the CPU needs to read data, it first checks its caches (L1, L2,
L3):

- If the data is not present (**cache miss**), the CPU fetches the data from RAM.
- However, it does not fetch just the specific byte or word requested—it fetches an entire
  **cache line** (e.g., 64 bytes on Intel CPUs).
- This cache line is then stored in the cache, and the requested data is delivered to the
  CPU. Similarly, when data is written, the corresponding cache line is updated and
  eventually written back to main memory.

**Impact on Memory Latency:** Cache lines significantly influence memory latency:

- **Spatial Locality:** By fetching a block of contiguous data, cache lines exploit the
  tendency of programs to access nearby memory locations, minimizing subsequent memory
  accesses.
- **Cache Miss Penalty:** While a cache miss incurs a substantial latency penalty to fetch
  the entire line, this is often offset by the fact that a larger chunk of potentially
  needed data is brought into the cache at once.
- **Bandwidth Utilization:** Transferring data in larger cache line units optimizes the
  use of the available memory bandwidth compared to numerous small transfers.

**Example Benefit:** Accessing elements of an array sequentially demonstrates the
advantage. With a 64-byte cache line and 4-byte integers, fetching one integer brings 15
neighboring integers into the cache, likely satisfying future access requests without
needing to go back to main memory.

More information on cache placement policies:

- [Wikipedia: CPU Cache placement policies](https://en.wikipedia.org/wiki/Cache_placement_policies#Example_3)

## Stack vs heap

Both stack and heap are in main memory (RAM). Their differences are logical rather than
physical.

### Stack memory

- **Cache friendliness**: Stack memory tends to be more cache-friendly.
  - Access patterns are predictable and localized.
  - Recent stack frames likely remain in CPU cache.
- **Locality**: Excellent spatial and temporal locality.
  - Data accessed together is stored together.
  - Recently accessed data is likely to be accessed again soon.
- **Allocation cost**: Essentially free (just incrementing/decrementing a stack pointer).

### Heap memory

- **Cache behavior**: Often less cache-friendly.
  - Allocations can be scattered throughout memory.
  - More likely to cause cache misses.
- **Locality**: Usually poorer spatial locality.
  - Related objects may be far apart in memory.
  - More random access patterns.
- **Allocation cost**: Relatively expensive.
  - Requires searching for free blocks.
  - May involve complex bookkeeping.

### Why stack access is often faster

1. **Predictable access pattern**: The CPU can prefetch stack data more effectively.
2. **Cache utilization**: Better use of cache lines due to contiguous memory access.
3. **Allocation overhead**: No complex memory management routines.

### Practical implications in Rust and Linux

- Small, fixed-size values benefit from stack allocation.
- Larger or dynamically-sized values must use heap allocation.
- Cache line considerations might apply more predictably to stack memory.

Stack size is important when considering memory access patterns. Stack memory benefits
from:

- Better cache locality (growing/shrinking in a linear fashion)
- More predictable access patterns
- Automatic management (no allocation overhead)

The fixed size nature of stacks is why recursion can cause stack overflow errors, while
heap allocations (which have their own performance trade-offs) can grow dynamically until
system memory is exhausted.

This is why data-oriented design principles often recommend organizing data for better
cache utilization, regardless of whether it's on stack or heap.

### Stack size in Ubuntu 25.04

In Ubuntu 25.04 with the latest Linux kernel, the default stack size for:

- **User threads**: 8 MB (8,388,608 bytes)
- **Kernel threads**: ~16 KB (kernel space stack)

This is configurable through several mechanisms:

1. Check current stack size in a terminal with:

   ```shell
   # Displays the current stack size limit in KB.
   ulimit -s
   ```

2. Modify stack size temporarily:

   ```shell
   # Set stack size to 8192 KB.
   ulimit -s 8192
   ```

3. For permanent changes, edit `/etc/security/limits.conf`:
   ```ini
   # <domain> <type> <item> <value>
   # - domain: * means all users.
   # - type: soft and hard are the limit types.
   # - item: stack is the resource.
   # - value: The value is in kilobytes (KB).
   * soft stack 8192
   * hard stack 16384
   ```

### Allocation and drop

The cost of dropping (deallocating) memory on the heap using Rust’s default allocator
(`std::alloc::System`, which typically wraps the underlying OS `malloc` / `free` provided
by `glibc` GNU C Library) is generally much lower than the cost of allocating it, but it
is not free. Here's an example of the costs involved in allocating and dropping 500KB of
memory on the heap and stack:

| Operation    | Stack (500 KB) | Heap (500 KB) | Relative Difference |
| ------------ | -------------- | ------------- | ------------------- |
| Allocation   | ~10–100 ns     | ~1–10 μs      | 10x–100x slower     |
| Deallocation | ~10–100 ns     | ~1–10 μs      | 10x–100x slower     |

> Note: 1μs (micro second) = 1,000ns (nano second)

**Heap:**

- **Allocating**: Can be expensive, especially for large or many small allocations, due to
  searching for free blocks, updating allocator metadata, and possible fragmentation.
- **Dropping/Deallocating**: Usually faster, as it typically just marks the memory as free
  and updates allocator metadata. However, the actual cost depends on the allocator’s
  implementation and fragmentation state.

**Stack:**

- **Allocating**: Very cheap (just moves the stack pointer). However, note that filling it
  with valid data can be expensive if the data is large.
- **Dropping/Deallocating**: Also very cheap (just moves the stack pointer back).

### Heap memory example, String and string slice

First add the following dependencies to your project:

```shell
cargo add r3bl_tui
```

Then you can run the following code:

```rust
#[cfg(test)]
mod string_and_vec_tests {
    use r3bl_tui::{fg_light_yellow_green, fg_lizard_green};

    #[serial_test::serial]
    #[test]
    /// Demonstrates the memory layout of String, which contains [ptr, len, capacity].
    fn mem_layout_string() {
        fg_lizard_green("\n=== String Memory Layout Example ===").println();

        // Create a String.
        // ASCII values for digits:
        // '0': 48 (0x30), '1': 49 (0x31), '2': 50 (0x32), '3': 51 (0x33), '4': 52 (0x34)
        // '5': 53 (0x35), '6': 54 (0x36), '7': 55 (0x37), '8': 56 (0x38), '9': 57 (0x39)
        let s = String::from("0123456789");

        // We can get these values safely.
        fg_light_yellow_green("\nSafely accessing String metadata:").println();
        println!("  ptr: {:p}", s.as_ptr());
        println!("  len: {}", s.len());
        println!("  cap: {}", s.capacity());

        // Unsafely transmute String to Vec of bytes.
        // This will show the Vec representation which includes the UTF-8 bytes
        // (identical to ASCII values for these digits).
        fg_light_yellow_green("\nUnsafely accessing String as Vec<u8> (hex dump):").println();
        println!("{:x?}", unsafe {
            std::mem::transmute::<String, Vec<u8>>(s)
        });

        // Note that transmuting a String to the following does not work:
        // let (ptr, len, cap): (*mut usize, usize, usize) = unsafe { std::mem::transmute(s) };
        // - `(*const u8, usize, usize)`
        // - `(*mut u8, usize, usize)`
        {
            fg_light_yellow_green("\nAccessing String with into_raw_parts():").println();
            let s = String::from("0123456789");
            let (ptr, len, cap) = s.into_raw_parts();
            println!("  ptr: {:p}", ptr);
            println!("  len: {}", len);
            println!("  cap: {}", cap);
        }
    }

    #[serial_test::serial]
    #[test]
    /// Demonstrates the memory layout of &str, which contains [ptr, len].
    fn mem_layout_str_slice() {
        fg_lizard_green("\n=== &str Memory Layout Example 1 ===").println();

        // Create a string slice
        let s = "Hello, world!";

        // &str is represented as [ptr, len].
        unsafe {
            // Transmute &str to raw parts.
            let raw_parts: (*const u8, usize) = std::mem::transmute(s);

            fg_light_yellow_green("\n&str memory layout:").println();
            println!("  ptr: {:p}", raw_parts.0);
            println!("  len: {}", raw_parts.1);

            // We can also get these values safely
            fg_light_yellow_green("\nSafely accessing &str metadata:").println();
            println!("  ptr: {:p}", s.as_ptr());
            println!("  len: {}", s.len());
        }
    }

    #[serial_test::serial]
    #[test]
    fn mem_layout_str_slice_2() {
        fg_lizard_green("\n=== &str Memory Layout Example 2 ===").println();

        // Demonstrate that &str is just a view into some data.
        let owned = String::from("Hello, world!");
        let slice = &owned[0..5]; // "Hello".

        // Safe approach to get the pointer and length for slice.
        let slice_ptr = slice.as_ptr();
        let slice_len = slice.len();

        // Safe approach to get the pointer and length for owned.
        let owned_ptr = owned.as_ptr();
        let owned_len = owned.len();
        let owned_capacity = owned.capacity();

        fg_light_yellow_green("\nComparing owned String and &str slice (safely):").println();
        println!("  String ptr: {:p}", owned_ptr);
        println!("  &str ptr:   {:p}", slice_ptr);
        println!(
            "  String points to same memory as slice: {}",
            slice_ptr == owned_ptr
        );
        println!("  String len: {}, slice len: {}", owned_len, slice_len);
        println!("  String cap: {}", owned_capacity);
    }
}
```

## Memory alignment

Memory alignment refers to arranging data in memory at addresses that are multiples of the
data type’s alignment requirement.

The alignment of a value specifies what addresses are valid to store the value at.

A value of alignment n must only be stored at an address that is a multiple of `n`. For
example, a value with an alignment of `2` must be stored at an even address, while a value
with an alignment of `1` can be stored at any address.

- Alignment is measured in bytes, and must be at least `1`, and always a power of `2`.
- The alignment of a value can be checked with the
  [`align_of_val`](https://doc.rust-lang.org/core/mem/fn.align_of_val.html) function.

Rust’s type system and compiler automatically handle memory alignment for safety and
performance, but understanding alignment is important when working with FFI, low-level
code, or optimizing data structures.

On a 14th gen Intel CPU (which is a 64-bit x86_64 architecture), the default alignment for
primitive types in Rust is:

- 8 bytes for types whose size is 8 bytes (e.g., `u64`, `f64`, `usize`, pointers)
- 4 bytes for types whose size is 4 bytes (e.g., `u32`, `i32`, `f32`)
- 2 bytes for types whose size is 2 bytes (e.g., `u16`, `i16`)
- 1 byte for types whose size is 1 byte (e.g., `u8`, `i8`)
- The alignment of a type is usually equal to its size, but only up to the CPU’s word size
  (which is 8 bytes on 64-bit Intel CPUs). So, the maximum default alignment for most
  types is 8 bytes. Custom types (structs, arrays) may have larger alignment if specified
  with `repr(align(N))`.

Here's an example of how alignment can affect the layout of a struct.

First add the following dependencies to your project:

```shell
cargo add r3bl_tui
```

Then you can run the following code:

```rust
use std::mem::{size_of, align_of};
use r3bl_tui::{fg_light_yellow_green, fg_lizard_green};

#[repr(C)]
struct Demo {
    a: u8,  // 1 byte, alignment 1
    b: u32, // 4 bytes, alignment 4
    c: u16, // 2 bytes, alignment 2
}

fn main() {
    let size = size_of::<Demo>();
    let align = align_of::<Demo>();

    fg_lizard_green(format!("\nSize of Demo: {size}")).println();
    fg_light_yellow_green(format!("Alignment of Demo: {align}")).println();
}
```

The default alignment of 4 bytes for many types (like `u32` or `i32`) is based on their
size and the requirements of most modern CPUs, especially 32-bit architectures. The
alignment ensures that memory accesses are efficient and compatible with the CPU’s
expectations.

- On a 32-bit CPU, the natural word size is 4 bytes, so types like `u32` and pointers are
  aligned to 4 bytes.
- On a 64-bit CPU, the natural word size is 8 bytes, so types like `u64` and pointers are
  aligned to 8 bytes. However, smaller types (`u32`, `i32`, etc.) still have 4-byte
  alignment, unless you use a type that requires more.

Here's an example that shows the alignment of different types.

First add the following dependencies to your project:

```shell
cargo add r3bl_tui
```

Then you can run the following code:

```rust
use std::mem::{size_of, align_of};
use r3bl_tui::{fg_light_yellow_green, fg_lizard_green};

fn pretty_print<T: std::fmt::Debug>() {
    let type_name = std::any::type_name::<T>();
    let size = size_of::<T>();
    let align = align_of::<T>();

    fg_lizard_green(format!("\n{type_name}")).println();
    fg_light_yellow_green(format!("  size = {size}\n  alignment = {align}")).println();
}

fn main() {
    pretty_print::<u8>();
    pretty_print::<u16>();
    pretty_print::<u32>();
    pretty_print::<u64>();
    pretty_print::<usize>();
    pretty_print::<f64>();
}
```

Resources:

- [Rust Reference: Type Layout](https://doc.rust-lang.org/reference/type-layout.html)
- [Forum discussion](https://users.rust-lang.org/t/type-alignment-understanding-memory-layout/126503/56)

## Global allocators

`jemalloc` is a replacement for the default global allocator. It's optimized for
multi-threaded use cases where lots of small objects are created and destroyed. The
default allocator is the system allocator that's optimized for single threaded use cases.

- <https://www.svix.com/blog/heap-fragmentation-in-rust-applications/>
- <https://news.ycombinator.com/item?id=35473271>
- <https://crates.io/crates/jemallocator>
- <https://engineering.fb.com/2011/01/03/core-infra/scalable-memory-allocation-using-jemalloc/>

Here's an example of how to use `jemalloc` as the global allocator in a Rust project.

First add the following dependencies to your project:

```shell
cargo add tikv-jemallocator r3bl_tui
```

Then you can use it in your code:

```rust
use r3bl_tui::set_jemalloc_in_main;

fn main() {
    set_jemalloc_in_main!();
    println!("jemalloc allocator is set.");
}
```

## Using arrays for stack or heap allocation

A ring buffer is a data structure that uses a fixed-size array to store elements in a
circular manner. It is often used in scenarios where a fixed-size buffer is needed, such
as in embedded systems or real-time applications. The ring buffer can be implemented using
either stack or heap allocation, depending on the requirements of the application.

Regardless of allocating this on the stack or the heap, we are working with a fixed-size
array, which can't be resized. So instead of using a `Vec`, we can use a fixed-size array.

Here are some tips on how to work with these types of data structures in Rust:

1. Here's the pattern we can use for declaring how the data will be stored in the ring
   buffer struct: `internal_storage: [Option<T>; N]`. The type is `Option<T>` because any
   slot in the ring buffer can be empty or contain a value.
2. In order to construct this struct, we can use the pattern
   `internal_storage: [(); N].map(|_| None)`, which works for any `T`. Since
   `internal_storage: [None; N]` does not work unless you are willing to constrain
   `T: Copy` which can be limiting.
3. The struct will have to use this generic header:
   `pub struct RingBuffer<T, const N: usize>`. This allows us to create a ring buffer of
   any type `T` with a fixed size `N`.
4. The impl block of this struct will have to use the same generic header:
   `impl<T, const N: usize> RingBuffer<T, N>`. This allows us to implement methods for the
   ring buffer that can work with any type `T` and any size `N`.

```rust
//! - Show stack alloc ring buffer using array allocated on stack.
//! - And pre-allocate using the pattern `internal_storage: [Option<T>; N]`.
//! - Show this constructor magic: `internal_storage: [(); N].map(|_| None)`.
//! - Show this generic header: `pub struct RingBuffer<T, const N: usize>`.
//! - Show the impl block with the same generic header: `impl<T, const N: usize>`.

pub struct RingBuffer<T, const N: usize> {
    internal_storage: [Option<T>; N],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        RingBuffer {
            internal_storage: [(); N].map(|_| None),
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn cap(&self) -> usize {
        N
    }

    pub fn add(&mut self, item: T) {
        if self.count == N {
            // Buffer is full, overwrite the oldest item.
            self.tail = (self.tail + 1) % N;
        } else {
            self.count += 1;
        }
        self.internal_storage[self.head] = Some(item);
        self.head = (self.head + 1) % N;
    }

    pub fn remove(&mut self) -> Option<T> {
        if self.count == 0 {
            return None; // Buffer is empty.
        }
        let item = self.internal_storage[self.tail].take();
        self.tail = (self.tail + 1) % N;
        self.count -= 1;
        item
    }
}

#[cfg(test)]
mod ring_buffer_inline_tests {
    use super::*;

    #[test]
    pub fn test_queue_api() {
        let mut rb = RingBuffer::<u8, 4>::new();

        // Partially fill the ring buffer.
        {
            rb.add(1);
            rb.add(2);
            rb.add(3);
            assert_eq!(rb.len(), 3);
            assert_eq!(rb.cap(), 4);

            let a = rb.remove();
            let b = rb.remove();
            let c = rb.remove();

            assert_eq!(a, Some(1));
            assert_eq!(b, Some(2));
            assert_eq!(c, Some(3));
        }

        // Fill the ring buffer to capacity.
        {
            for i in 0..4 {
                rb.add(i);
            }
            assert_eq!(rb.remove(), Some(0));
            assert_eq!(rb.remove(), Some(1));
            assert_eq!(rb.remove(), Some(2));
            assert_eq!(rb.remove(), Some(3));
            assert_eq!(rb.remove(), None);
        }

        // Overfill the ring buffer.
        {
            rb.add(1);
            rb.add(2);
            rb.add(3);
            rb.add(4);
            rb.add(5);

            assert_eq!(rb.len(), 4);
            assert_eq!(rb.cap(), 4);

            assert_eq!(rb.remove(), Some(2));
            assert_eq!(rb.remove(), Some(3));
            assert_eq!(rb.remove(), Some(4));
            assert_eq!(rb.remove(), Some(5))
            assert_eq!(rb.remove(), None);
        }
    }
}
```

## Using smallvec and smallstr crates

### smallvec

`smallvec` is a crate that is part of the Servo project (which is now in the Linux
Foundation) that provides a vector type that can store a small number of elements on the
stack. If the capacity of the vector exceeds the stack size, it will automatically
allocate on the heap. You can check this using the `spilled()` method.

This is useful if you want to allocate a small number of `Sized` items on the stack.
However, if you have a large number of items, then `Vec` is a better choice. Also the size
of the stack is typically limited to 8MB on most systems, so be careful when using
`smallvec` with large types or lots of items of a type, to avoid stack overflow.

The `r3bl_tui` crate provides a `InlineVec` type that is a wrapper around
`smallvec::SmallVec` and a `inline_vec!` macro that can be used to create an `InlineVec`
with items that are provided inline to the macro.

To run the example below, first add the following dependencies to your project:

```shell
cargo add smallvec r3bl_tui
```

Then you can run the following code:

```rust
//! This module demonstrates the use of `smallvec` crate. And easier to
//! use version of them: `InlineVec`.
//!
//! - Show how to use smallvec -> InlineVec
//! - Show how to use smallstr -> InlineString
//! - Use the join_ macros from r3bl_tui

#[cfg(test)]
mod inline_vec_ex_tests {
    use r3bl_tui::{Index, InlineVec, Length, fg_lizard_green, inline_vec, len};

    #[serial_test::serial]
    #[test]
    fn test_new_inline_vec() {
        // Using with default capacity. Use `[]` accessor.
        {
            let mut inline_vec = InlineVec::new();
            let length: Length = len(5); // 5
            let max_index: Index = length.convert_to_index(); // 4
            for i in 0..=max_index.as_usize() {
                inline_vec.push(i); // 0, 1, 2, 3, 4
            }
            assert_eq!(inline_vec[Index::from(0).as_usize()], 0);
            assert_eq!(inline_vec[max_index.as_usize()], 4);
            // assert_eq!(inline_vec[max_index.as_usize() + 1], 0); // OOB error!
            assert_eq!(inline_vec.get(max_index.as_usize() + 1), None);
            fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();
            assert_eq!(inline_vec.capacity(), 8);
            assert_eq!(inline_vec.len(), 5);
        }

        // Using with macro. Use `get()` accessor.
        {
            let length: Length = len(5); // 5
            let max_index: Index = length.convert_to_index(); // 4
            let inline_vec = inline_vec!(0, 1, 2, 3, 4);
            assert_eq!(inline_vec.get(Index::from(0).as_usize()), Some(&0));
            assert_eq!(inline_vec.get(max_index.as_usize()), Some(&4));
            assert_eq!(inline_vec.get(max_index.as_usize() + 1), None);
            fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();
            assert_eq!(inline_vec.capacity(), 8);
            assert_eq!(inline_vec.len(), 5);
        }

        // Using with capacity (even though it is pre-allocated). Use `get()` accessor.
        {
            let mut inline_vec = InlineVec::with_capacity(5);
            let length: Length = len(5); // 5
            let max_index: Index = length.convert_to_index(); // 4
            for i in 0..=max_index.as_usize() {
                inline_vec.push(i); // 0, 1, 2, 3, 4
            }
            assert_eq!(inline_vec.get(Index::from(0).as_usize()), Some(&0));
            assert_eq!(inline_vec.get(max_index.as_usize()), Some(&4));
            assert_eq!(inline_vec.get(max_index.as_usize() + 1), None);
            fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();
            assert_eq!(inline_vec.capacity(), 8);
            assert_eq!(inline_vec.len(), 5);
        }
    }

    #[serial_test::serial]
    #[test]
    fn test_mut_inline_vec() {
        let mut inline_vec = InlineVec::new();

        let length: Length = len(5); // 5
        let max_index: Index = length.convert_to_index(); // 4
        for i in 0..=max_index.as_usize() {
            inline_vec.push(i); // 0, 1, 2, 3, 4
        }

        inline_vec[max_index.as_usize()] = 100;

        assert_eq!(inline_vec[0], 0);
        assert_eq!(inline_vec[max_index.as_usize()], 100);

        fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();

        // Remove the first element, and shift the rest.
        inline_vec.remove(0);
        assert_eq!(inline_vec.len(), 4);
        assert_eq!(inline_vec.capacity(), 8);
        assert_eq!(inline_vec[0], 1);
        assert_eq!(inline_vec[3], 100);
        fg_lizard_green(format!("InlineVec: {:?}", inline_vec)).println();
    }

    #[serial_test::serial]
    #[test]
    #[should_panic]
    fn test_inline_vec_oob() {
        let mut inline_vec = InlineVec::new();

        assert_eq!(inline_vec.capacity(), 8);
        assert_eq!(inline_vec.len(), 0);

        let length: Length = len(5); // 5
        let max_index: Index = length.convert_to_index(); // 4
        for i in 0..=max_index.as_usize() {
            inline_vec.push(i); // 0, 1, 2, 3, 4
        }

        assert_eq!(inline_vec.capacity(), 8);
        assert_eq!(inline_vec.len(), 5);

        // This should panic because we are trying to access an index that is out of
        // bounds.
        inline_vec[max_index.as_usize() + 1] = 100;
    }
}

#[cfg(test)]
mod smallvec_ex_tests {
    use smallvec::{SmallVec, smallvec};

    // Type alias to reduce typing.
    type MySmallVec = SmallVec<[u8; 4]>;

    #[serial_test::serial]
    #[test]
    fn test_new_smallvec() {
        // With new.
        {
            let mut acc = MySmallVec::new();
            for i in 0..=2 {
                acc.push(i); // 0, 1, 2
            }
            assert_eq!(acc.len(), 3);
            assert_eq!(acc.capacity(), 4);
            assert_eq!(acc.get(0), Some(&0));
            assert_eq!(acc.get(1), Some(&1));
            assert_eq!(acc.get(2), Some(&2));
            assert_eq!(acc.get(3), None);
        }

        // With macro.
        {
            let acc: MySmallVec = smallvec![0, 1, 2];
            assert_eq!(acc.len(), 3);
            assert_eq!(acc.capacity(), 4);
            assert_eq!(acc[0], 0);
            assert_eq!(acc[1], 1);
            assert_eq!(acc[2], 2);
        }
    }

    #[serial_test::serial]
    #[test]
    fn test_mut_smallvec() {
        let mut acc = MySmallVec::new();
        for i in 0..=2 {
            acc.push(i); // 0, 1, 2
        }
        assert_eq!(acc.len(), 3);
        assert_eq!(acc.capacity(), 4);

        acc[2] = 100;

        assert_eq!(acc[0], 0);
        assert_eq!(acc[1], 1);
        assert_eq!(acc[2], 100);

        // Remove the first element, and shift the rest.
        acc.remove(0);
        assert_eq!(acc.len(), 2);
        assert_eq!(acc.capacity(), 4);
        assert_eq!(acc[0], 1);
        assert_eq!(acc[1], 100);
    }
}
```

### smallstr

The `smallstr` crate is similar to `smallvec`, and it is build on top of `smallvec`. It
provides a string type that can store a small number of characters on the stack. If the
capacity of the string exceeds the stack size, it will automatically allocate on the heap.
This is useful for storing small strings on the stack, but if you have a large string,
then `String` is a better choice.

The `r3bl_tui` crate provides a `InlineString` type that is a wrapper around
`smallstr::SmallStr` and a `inline_string!` macro that can be used to create an
`InlineString` with items that are provided inline to the macro (use it like you would
`println!` since it uses `FmtArgs` under the hood).

To run the example below, first add the following dependencies to your project:

```shell
cargo add smallstr r3bl_tui
```

Then you can run the following code:

```rust
//! This module demonstrates the use of `smallstr` crate. And easier to
//! use version of them: `InlineString`.
//!
//! Show how to use smallstr -> InlineString

#[cfg(test)]
mod inline_string_ex_tests {
    use r3bl_tui::{InlineString, fg_lizard_green, fg_soft_pink, inline_string};
    use smallstr::SmallString;

    #[serial_test::serial]
    #[test]
    fn test_new_inline_string() {
        // Constructor.
        {
            let mut acc = InlineString::new();
            use std::fmt::Write as _;
            _ = write!(acc, "Hello, world!").unwrap();
            assert_eq!(acc, "Hello, world!");
        }

        // Macro.
        {
            let mut acc = inline_string!("Hello,");
            use std::fmt::Write as _;
            _ = write!(acc, " world!").unwrap();
            assert_eq!(acc, "Hello, world!");
        }
    }

    /// Demonstrates the use of `inline_string!` macro to create an
    /// `InlineString` and then format it using the `Display` trait.
    /// Without allocating a new [String] (on the heap).
    #[serial_test::serial]
    #[test]
    fn test_new_inline_string_display_impl() {
        struct DemoStruct {
            id: u8,
            name: InlineString,
        }

        impl std::fmt::Display for DemoStruct {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "id: {}, name: {}", self.id, self.name)
            }
        }

        let demo = DemoStruct {
            id: 1,
            name: inline_string!("Hello, world!"),
        };
        let to_inline_string = inline_string!("{}", demo);
        assert_eq!(to_inline_string, "id: 1, name: Hello, world!");
        fg_lizard_green(to_inline_string).println();
    }

    #[serial_test::serial]
    #[test]
    fn test_new_smallstr() {
        let mut acc: SmallString<[u8; 8]> = SmallString::new();
        assert_eq!(acc.capacity(), 8);
        assert_eq!(acc.len(), 0);
        fg_lizard_green(format!("is spilled: {}", acc.spilled())).println();

        use std::fmt::Write as _;
        _ = write!(acc, "Hello, world!").unwrap();
        assert_eq!(acc, "Hello, world!");

        assert_eq!(acc.len(), 13);
        assert_eq!(acc.spilled(), true);
        fg_soft_pink(format!("is spilled: {}", acc.spilled())).println();
    }
}
```
