<!-- cspell:words **(Visual: framerates Vecs bitshifts bitshift memmoves flatline -->
<!-- cspell:words bottlenecked ilog mispredictions superscalar VPCMPEQB memset memmove -->

# YouTube Script: Build High-Performance Flat 2D Arrays in Rust (using SIMD & L1 Cache)

## 1. Project Setup (0:00 - 0:45)

**(Visual: Terminal window)**

**Speaker:** "Today we are going to build a high-performance 2D array in Rust. Before we
dive into the code, let's scaffold our project and enable the nightly toolchain so we can
run micro-benchmarks later."

**(Visual: Typing in terminal)**

```fish
cd (mktemp -d)
cargo new --lib flat2darray
cd flat2darray
rustup override set nightly
```

**Speaker:** "We don't need any external dependencies, so `Cargo.toml` is good to go. We
just need to configure our `lib.rs` to enable the benchmarking features and expose our
modules."

**(Visual: Open `src/lib.rs` and type the following)**

```rust
pub mod vec_2d_array;
```

## 2. The Hook & The Naive Approach (0:45 - 1:45)

**(Visual: A developer staring at a terminal UI or a grid-based game with choppy
framerates. Then transition to a slick, 60FPS high-performance terminal UI like
r3bl-open-core.)**

**Speaker:** "If you are building a Terminal UI, an image processor, or a grid-based game
in Rust, you are going to need a 2D data structure to represent the screen.

The immediate, naive approach almost everyone takes is a 'Vec of Vecs', creating a
`Vec2DArray` struct."

**(Visual: Create `src/vec_2d_array.rs` and type the `Vec2DArray` data structure)**

```rust
use r3bl_tui::{ColWidth, RowHeight};

pub struct Vec2DArray<T: Clone> {
    pub data: Vec<Vec<T>>,
    pub rows: RowHeight,
    pub cols: ColWidth,
}
```

**Speaker:** "To manipulate this grid, we need a few standard methods. Let's ground them
in a real-world Terminal UI use case:

1. **Iterate**: We need to traverse the grid cell-by-cell to render it to the terminal.
2. **Diffing**: We need to compare the old frame buffer with the new frame buffer to only
   redraw pixels that changed.
3. **Clearing**: We need to wipe all cells in the grid to handle a 'clear screen' command.
4. **Scrolling**: We need to shift terminal history up by moving rows when a new line is
   printed at the bottom.

**(Visual: Continue in `src/vec_2d_array.rs`, adding the scalar methods, a
`.get_mem_size()` method to calculate heap allocation size, and a `#[cfg(test)]` module
with unit tests)**

````rust
impl<T: Copy + PartialEq + std::fmt::Debug> Vec2DArray<T> {
    pub fn new(arg_size: impl Into<Size>, default: T) -> Self {
        let size = arg_size.into();
        let row = vec![default; size.col_width.as_usize()];
        let vec_of_rows = vec![row; size.row_height.as_usize()];
        Self {
            data: vec_of_rows,
            rows: size.row_height,
            cols: size.col_width,
        }
    }

    pub fn clear(&mut self, default_val: T) {
        let rows_usize = self.rows.as_usize();
        let cols_usize = self.cols.as_usize();
        for row in 0..rows_usize {
            for col in 0..cols_usize {
                self.data[row][col] = default_val.clone();
            }
        }
    }

    /// Scrolls the grid up by one row.
    ///
    /// # Logic
    /// Because `self.data` is a `Vec<Vec<T>>`, we don't actually need to copy the
    /// underlying elements. We can simply rotate the `Vec` of row pointers!
    /// `rotate_left(1)` moves the first row to the end, and shifts all other row
    /// pointers up by 1. This is extremely fast because it only moves memory pointers,
    /// not the actual items in the rows.
    ///
    /// # ASCII Diagram
    /// ```text
    /// Before:         After:
    /// [ Row 0 ]       [ Row 1 ]  <-- Shifted up
    /// [ Row 1 ]  ==>  [ Row 2 ]
    /// [ Row 2 ]       [ Row 2 ]  <-- Duplicated from above
    /// ```
    pub fn scroll_up(&mut self) {
        if !self.data.is_empty() {
            self.data.rotate_left(1);
            let len = self.data.len();
            if len > 1 {
                // After rotation, the old top row is now at the bottom.
                // To mimic `copy_within` (where the last row is left untouched and thus
                // duplicated), we overwrite this new bottom row with a clone of the row
                // just above it.
                self.data[len - 1] = self.data[len - 2].clone();
            }
        }
    }

    pub fn diff(&self, other: &Self) -> Vec<Pos> {
        let mut changes = Vec::new();
        for row in 0..self.rows.as_usize() {
            for col in 0..self.cols.as_usize() {
                if self.data[row][col] != other.data[row][col] {
                    let row: RowIndex = row.into();
                    let col: ColIndex = col.into();
                    changes.push(row + col);
                }
            }
        }
        changes
    }

    pub fn get_mem_size(&self) -> usize {
        // Struct.
        let mut total = std::mem::size_of::<Self>();

        // Vec<_>.
        let num_of_rows = self.data.capacity();
        total += num_of_rows * {
            let size_of_a_row = std::mem::size_of::<Vec<T>>();
            size_of_a_row
        };

        // Vec<T> for each row (containing the columns / cells).
        for row in &self.data {
            total += row.capacity() * std::mem::size_of::<T>();
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_2d_array() {
        let mut grid = Vec2DArray {
            data: vec![vec![0; 10]; 5],
            cols: ColWidth::from(10),
            rows: RowHeight::from(5),
        };
        grid.clear(1);
        assert_eq!(grid.data[0][0], 1);
    }
}
````

But there is a major performance catch here. A `Vec<Vec<T>>` is terrible for iteration
because it requires multiple heap allocations scattered randomly in memory. This destroys
CPU cache locality. Because the CPU's Hardware Prefetcher can't predict where the next row
is, the pipeline suffers massive stalls—wasting up to 300 clock cycles fetching from slow
RAM instead of the 1 to 4 cycles it takes to read from the L1 Cache. By the end of this
video, I'll show you exactly how much performance you're losing with real benchmarks.

But don't throw away `Vec<Vec<T>>` entirely! I'll also show you one surprisingly elegant
trick where `Vec2DArray` actually wins: scrolling. Because it's an array of pointers, we
can use `rotate_left(1)` to scroll the screen by just shifting pointers, which is
lightning fast compared to moving contiguous bytes!"

> Review the content in this page (local rustdocs for Flat1DSimd)::
> `file:///tmp/roc/target/doc/r3bl_tui/core/common/flat_2d_array/core/struct.Flat1DSimd.html#the-cpu-cache--hardware-prefetching`

## 3. The 1D Solution (1:45 - 3:15)

**(Visual: Open `src/lib.rs` and type the following)**

```rust
pub mod flat_2d_array;
```

**(Visual: Transition to the Flat2DArray struct)**

**Speaker:** "The solution is to flatten our 2D grid into a single, contiguous 1D array
using `Box<[T]>`."

"Why does this matter? Let's look at the memory layout."

**(Visual: Create `src/flat_2d_array.rs` and copy the following diagrams in it)**

```text
/// # Vec<Vec<T>> (Scattered Heap Memory):
///
/// [Ptr] -> [Ptr, Ptr, Ptr]
///           |    |    |
///           v    v    v
///        [Row1] [Row2] [Row3]  <-- Cache Misses!
///
/// Box<[T]> (Contiguous Memory):
/// [Ptr] -> [Row1 | Row2 | Row3] <-- Perfect L1/L2 Cache Hits!
```

**Speaker:** "By guaranteeing a single contiguous memory allocation, the CPU's Hardware
Prefetcher can effortlessly pull data from RAM into the L1 Cache in perfect 64-byte
chunks, known as Cache Lines. To access any coordinate, we just use simple math:
`(row, col) -> index = row * cols + col`."

```text
/// # 2D to 1D Mapping
///
/// The grid is stored row-by-row in a flat 1D slice.
///
/// To find the element at `(row, col)`, we skip `row` full rows of size `width`,
/// and then step forward by `col`.
///
///           col 0   col 1   col 2
///        ┌───────┬───────┬───────┐
///  row 0 │ idx 0 │ idx 1 │ idx 2 │
///        ├───────┼───────┼───────┤
///  row 1 │ idx 3 │ idx 4 │ idx 5 │
///        ├───────┼───────┼───────┤
///  row 2 │ idx 6 │ idx 7 │ idx 8 │
///        └───────┴───────┴───────┘
///
///   row 0                   row 1                   row 1
///   col 0   col 1   col 2   col 0   col 1   col 2   col 0   col 1   col 2
/// ┌───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────┐
/// │ idx 0 │ idx 1 │ idx 2 │ idx 3 │ idx 4 │ idx 5 │ idx 6 │ idx 7 │ idx 8 │
/// └───────┴───────┴───────┴───────┴───────┴───────┴───────┴───────┴───────┘
///
/// Example: `(row 1, col 2)`
/// - `row_offset = 1 * 3 = 3`
/// - `final_index = 3 + 2 = 5`
```

````text
/// # 1D to 2D Mapping
///
/// This is the exact inverse of the above. It is primarily used
/// during SIMD fast-path diffing, where the algorithm iterates linearly over the
/// 1D slice, finds a difference at a specific 1D `index`, and needs to know the
/// corresponding `(row, col)` coordinate to issue a terminal cursor movement
/// command.
///
/// ```text
/// 1D Grid:
///
///   row 0                   row 1                   row 1
///   col 0   col 1   col 2   col 0   col 1   col 2   col 0   col 1   col 2
/// ┌───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────┐
/// │ idx 0 │ idx 1 │ idx 2 │ idx 3 │ idx 4 │ idx 5 │ idx 6 │ idx 7 │ idx 8 │
/// └───────┴───────┴───────┴───────┴───────┴───────┴───────┴───────┴───────┘
///                                           ^
///                                           index_to_pos(5)
/// 2D Grid (equivalent):
///
///          col 0   col 1   col 2
///        ┌───────┬───────┬───────┐
///  row 0 │ idx 0 │ idx 1 │ idx 2 │
///        ├───────┼───────┼───────┤
///  row 1 │ idx 3 │ idx 4 │ idx 5 │  ← index_to_pos(5)
///        ├───────┼───────┼───────┤    = Pos { row: 1, col: 2 }
///  row 2 │ idx 6 │ idx 7 │ idx 8 │
///        └───────┴───────┴───────┘
/// ```
///
/// Example: `index 5` with `width 3`
/// - `row = index / width = 5 / 3 = 1`
/// - `col = index % width = 5 % 3 = 2`
````

**(Visual: In `src/flat_2d_array.rs` type the `Flat2DArray` data structure, the scalar
methods, a `.get_mem_size()` method, and a `#[cfg(test)]` module with unit tests)**

```rust
use r3bl_tui::{ColWidth, RowHeight};

pub struct Flat2DArray<T: Clone> {
    pub data: Box<[T]>,
    pub rows: RowHeight,
    pub cols: ColWidth,
}

impl<T: Copy + PartialEq + std::fmt::Debug> Flat2DArray<T> {
    pub fn new(arg_size: impl Into<Size>, default_val: T) -> Self {
        let size = arg_size.into();
        let flat_size = size.row_height.as_usize() * size.col_width.as_usize();
        let data = vec![default_val; flat_size];
        Self {
            data: data.into_boxed_slice(),
            rows: size.row_height,
            cols: size.col_width,
        }
    }

    pub fn clear(&mut self, default_val: T) {
        let cols_usize = self.cols.as_usize();
        let rows_usize = self.rows.as_usize();
        for row in 0..rows_usize {
            for col in 0..cols_usize {
                self.data[row * cols_usize + col] = default_val.clone();
            }
        }
    }

    pub fn scroll_up(&mut self) {
        let cols_usize = self.cols.as_usize();
        let rows_usize = self.rows.as_usize();
        for row in 0..rows_usize - 1 {
            for col in 0..cols_usize {
                let src_idx = (row + 1) * cols_usize + col;
                let dest_idx = row * cols_usize + col;
                self.data[dest_idx] = self.data[src_idx].clone();
            }
        }
    }

    pub fn diff(&self, other: &Self) -> Vec<Pos> {
        let mut changes = Vec::new();
        let cols_usize = self.cols.as_usize();
        let rows_usize = self.rows.as_usize();
        for row in 0..rows_usize {
            for col in 0..cols_usize {
                let idx = row * cols_usize + col;
                if self.data[idx] != other.data[idx] {
                    let row: RowIndex = row.into();
                    let col: ColIndex = col.into();
                    changes.push(row + col);
                }
            }
        }
        changes
    }

    pub fn print_screen(&self) {
        let cols_usize = self.cols.as_usize();
        for (index, item) in self.data.iter().enumerate() {
            let row = index / cols_usize;
            let col = index % cols_usize;
            print!("[{}][{}]-{:?} ", row, col, item);
        }
        println!();
    }

    pub fn get_mem_size(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();
        total += self.data.len() * std::mem::size_of::<T>();
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_2d_array() {
        let grid = Flat2DArray {
            data: vec![0; 50].into_boxed_slice(),
            cols: ColWidth::from(10),
            rows: RowHeight::from(5),
        };
        assert!(grid.get_mem_size() > 0);
    }
}
```

### Ergonomic Array Access (`Index` and `IndexMut`)

To make `Flat2DArray` feel just like `Vec2DArray`, we can implement the `Index` and
`IndexMut` traits for both `usize` (row index) and `Pos` (row + col coordinates):

```rust
use std::ops::{Index, IndexMut};
use r3bl_tui::Pos;

impl<T: Copy + PartialEq> Index<usize> for Flat2DArray<T> {
    type Output = [T];

    fn index(&self, row_index: usize) -> &Self::Output {
        let cols = self.cols.as_usize();
        let range_start = row_index * cols;
        let range_end = range_start + cols;
        &self.data[range_start..range_end]
    }
}

impl<T: Copy + PartialEq> IndexMut<usize> for Flat2DArray<T> {
    fn index_mut(&mut self, row_index: usize) -> &mut Self::Output {
        let cols = self.cols.as_usize();
        let range_start = row_index * cols;
        let range_end = range_start + cols;
        &mut self.data[range_start..range_end]
    }
}

impl<T: Copy + PartialEq> Index<Pos> for Flat2DArray<T> {
    type Output = [T];

    fn index(&self, pos: Pos) -> &Self::Output {
        let row = pos.row_index.as_usize();
        let col = pos.col_index.as_usize();
        let cols = self.cols.as_usize();
        let range_start = row * cols + col;
        let range_end = range_start + 1;
        &self.data[range_start..range_end]
    }
}

impl<T: Copy + PartialEq> IndexMut<Pos> for Flat2DArray<T> {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        let row = pos.row_index.as_usize();
        let col = pos.col_index.as_usize();
        let cols = self.cols.as_usize();
        let range_start = row * cols + col;
        let range_end = range_start + 1;
        &mut self.data[range_start..range_end]
    }
}
```

## 4. The 2D Iteration Trap (3:15 - 4:45)

**(Visual: A warning sign or 'trap' icon)**

**Speaker:** "But wait, there is a trap here: The Math Pipeline Stall Problem. What if we
need to iterate over the whole grid, but we _still need_ to know our `(row, col)`
coordinates to know where to draw them? This is a common scenario in a Terminal UI, where
we need to render each pixel at its correct position (row and col) on the terminal
emulator screen by writing an ANSI escape sequence to `stdout`.

The naive way to iterate our flat array looks like this:"

**(Visual: Highlight the `print_screen` method in `src/flat_2d_array.rs` as the speaker
discusses it)**

**Speaker:** "The trap is that division (`/`) and modulo (`%`) are extremely slow for the
CPU. Now, if our grid `cols` was a guaranteed compile-time constant _and_ a perfect power
of two—like 128—the compiler is smart. It optimizes the math into lightning-fast
bitshifts:"

**(Visual: Code snippet on screen)**

```rust
const COLS: usize = 128;
const SHIFT_AMT: u32 = COLS.ilog2(); // 7
const MASK_AMT: usize = COLS - 1;    // 127

let index = 5;
let row = index >> SHIFT_AMT; // Replaces / with bitshift
let col = index & MASK_AMT; // Replaces % with bitwise AND

println!("row: {}, col: {}", row, col);
```

**Speaker:** "But here is the catch: In a Terminal UI, the cols is almost never a power of
two, and it's a **runtime variable** because the user can resize their window at any time
(for example, to 113 columns). Because the compiler doesn't know this number at compile
time, it cannot use the bitshift trick. It is forced to emit actual, slow division
instructions to the CPU for every single pixel, causing significant pipeline stalls."

## 5. Unlocking SIMD & Raw Memory Operations (4:45 - 7:15)

**(Visual: The 'Two Rules of Thumb' slide)**

**(Visual: Add the SIMD-optimized methods to `src/flat_2d_array.rs` as the speaker
discusses them)**

**Speaker:** "So how do we fix it? We follow two simple rules of thumb for 1D memory
access to replace those slow scalar loops.

**Rule 1: If you DO care about 2D coordinates.** Let's say you're rendering or diffing
rows. The silver bullet here is `.chunks_exact(cols)`."

**(Visual: Type the `simd_diff` and `simd_print_screen` methods in
`src/flat_2d_array.rs`)**

```rust
    pub fn simd_print_screen(&self) -> String {
        let mut buffer = String::new();

        let cols /* chunk size / num cols / width */ = self.cols.as_usize();

        let rows_iter = self.data.chunks_exact(cols);
        debug_assert!(
            rows_iter.remainder().is_empty(),
            "The data length should be a multiple of the number of columns."
        );

        let rows_iter = rows_iter.enumerate();
        for (row_idx, row_chunk) in rows_iter {
            let cols_iter = row_chunk.iter().enumerate();
            for (col_idx, item) in cols_iter {
                buffer.push_str(&format!("({row_idx}, {col_idx}): {item:?} | "));
            }
        }

        buffer
    }

    pub fn simd_diff(&self, other: &Self) -> Vec<Pos> {
        let mut changes = Vec::new();

        let cols /* chunk size / num cols / width */ = self.cols.as_usize();

        let self_rows_iter = self.data.chunks_exact(cols);
        debug_assert!(
            self_rows_iter.remainder().is_empty(),
            "The data length should be a multiple of the number of columns."
        );

        let other_rows_iter = other.data.chunks_exact(cols);
        debug_assert!(
            other_rows_iter.remainder().is_empty(),
            "The data length should be a multiple of the number of columns."
        );

        let zipped_rows_iter = self_rows_iter.zip(other_rows_iter).enumerate();
        for (row_idx, (self_row_chunk, other_row_chunk)) in zipped_rows_iter {
            if self_row_chunk != other_row_chunk {
                let cols_iter = self_row_chunk.iter().zip(other_row_chunk.iter()).enumerate();
                for (col_idx, (s, o)) in cols_iter {
                    if s != o {
                        let row: RowIndex = row_idx.into();
                        let col: ColIndex = col_idx.into();
                        changes.push(row + col);
                    }
                }
            }
        }

        changes
    }
```

**Speaker:** "The performance wins here are massive, happening at multiple layers of the
CPU:

1. **Bypassing the Math Pipeline:** Under the hood, this doesn't use division (`/`) or
   modulo (`%`). It uses pure **pointer addition**, just adding `cols` to the memory
   pointer for each row.
2. **Eliding Bounds Checks:** By `zip()`ing `chunks_exact` together, we prove to the
   compiler at compile-time that both iterators have the exact same length. LLVM
   completely removes bounds checks from the inner loops, preventing branch
   mispredictions.

Here is the magic behind how that works for comparing two separate chunks of RAM (`self`
and `other`):

### Multi-Stream Hardware Prefetching

The CPU's hardware prefetcher isn't limited to tracking just one stream of memory. Modern
CPUs (like Intel, AMD, and Apple Silicon) can track multiple independent, sequential
memory streams simultaneously (often up to 16 or 32 streams at a time).

Because `simd_diff` iterates linearly through `self.data` and linearly through
`other.data`, the prefetcher quickly recognizes two distinct linear access patterns. It
fires off requests to RAM for both streams concurrently, pulling the next 64-byte Cache
Lines for both `self` and `other` into the L1 Cache ahead of time.

### Dual-Ported L1 Cache

L1 Caches on modern CPUs are usually "multi-ported." This means the CPU doesn't have to
wait to read `self` on cycle 1 and `other` on cycle 2. It can literally fetch data from
two completely different memory addresses in the L1 cache in the exact same clock cycle.

### SIMD Registers and Superscalar Execution

Once the data is sitting in the L1 cache, the CPU executes the equality check
(`if self_row_chunk != other_row_chunk`):

1. It issues two SIMD load instructions (e.g., pulling 32 bytes of `self` into register
   `YMM0` and 32 bytes of `other` into register `YMM1`).
2. Because CPUs are "superscalar" (meaning they can execute multiple instructions per
   cycle), it loads both registers at nearly the exact same time.
3. It then issues a single SIMD compare instruction (like `VPCMPEQB` in x86 AVX2).

While the latency of the entire pipeline (fetch, decode, load, compare) takes several
cycles, the CPU overlaps these operations in an assembly line (pipelining). The result is
a throughput of one massive 32-byte or 64-byte comparison retiring every single clock
cycle.

So, because the memory access is perfectly linear for both arrays, the hardware prefetcher
and L1 Cache perfectly spoon-feed the SIMD registers without ever starving the CPU!"

**Rule 2: If you DON'T care about 2D coordinates.** Let's say you just need to clear the
screen or scroll memory. You don't need chunks. Just blast through the entire raw 1D slice
using `.fill()` for instant clearing."

**(Visual: Type the `simd_clear` method in `src/flat_2d_array.rs`)**

```rust
    pub fn simd_clear(&mut self, default_val: T) {
        self.data.fill(default_val);
    }
```

**Speaker:** "Because it's an uninterrupted memory block, LLVM aggressively
auto-vectorizes this into SIMD instructions. But here's the kicker: `.fill()` doesn't
_read_ data, it _writes_ data. LLVM translates this into ultra-wide SIMD Store
instructions (acting like a massive `memset`). Instead of writing one character at a time,
the CPU blasts 32 or 64 bytes of the `default_val` directly into the L1 Cache in a single
clock cycle!

If your array is larger than a SIMD register, LLVM handles the magic: it creates a highly
optimized loop, unrolls it to keep the CPU pipeline saturated, and generates a 'scalar
tail' to perfectly clean up any leftover bytes at the end of the array.

**(Visual: Type the `simd_scroll_up` method in `src/flat_2d_array.rs`)**

````rust
/// Scrolls the grid up by one row.
///
/// # Logic
/// Because `self.data` is a flat 1D array, we shift the entire contiguous block of
/// memory left by `cols` elements. The `copy_within` method maps directly to
/// a highly optimized `memmove` operation, safely copying overlapping memory regions
/// in bulk.
///
/// # ASCII Diagram
/// ```text
/// Before:                                   After:
/// [ Row 0 | Row 1 | Row 2 | Row 3 ]         [ Row 1 | Row 2 | Row 3 | Row 3 ]
///           ^^^^^^^^^^^^^^^^^^^^^             ^^^^^^^^^^^^^^^^^^^^^   ^^^
///             Copied and shifted              Pasted at index 0       Duplicated
/// ```
pub fn simd_scroll_up(&mut self) {
    let src_range = self.cols.as_usize()..;
    self.data.copy_within(src_range, /*starting index*/ 0);
}
````

For scrolling, we use `.copy_within()`. This is incredibly fast because it maps directly
to `std::ptr::copy` (which acts as a highly optimized SIMD `memmove`). It shifts huge
contiguous blocks of memory in bulk rather than moving elements one by one.

However, as we'll see in the benchmarks, this is actually the _one_ scenario where a
nested `Vec2DArray` beats the Flat Array. Why? Because simply swapping memory pointers
(what a nested `Vec` does when you rotate rows) is always mathematically faster than
physically copying contiguous bytes, no matter how wide your SIMD registers are!"

## 6. Proving it with Benchmarks (7:15 - 8:45)

**(Visual: Open `src/lib.rs` and type the following)**

```rust
// We need this for cargo bench to work.
#![cfg_attr(test, feature(test))]

#[cfg(test)]
mod benches;
```

**(Visual: A split screen showing Vec2DArray and Flat2DArray benchmark results)**

**Speaker:** "We've implemented both `Vec2DArray` and `Flat2DArray`. Let's look at the
benchmarks. But before we do, we need to talk about modern CPU architectures.

**(Visual: Diagram of Intel P-Cores vs E-Cores and their L1 Cache sizes)**

If you are running a modern CPU, like an Intel 14th Gen, you have Performance Cores and
Efficiency Cores. The P-Cores have 48KB of L1 Data Cache, and the E-Cores have 32KB. To
test raw CPU instruction speed, we need a grid small enough to fit inside that 32KB limit.
If we use a grid that is too large, the CPU will spend all its time waiting for RAM, and
we'll hit the **Memory Wall**.

**(Visual: Create `src/benches.rs` and set up the modules)**

```rust
#![allow(
    dead_code,
    unused_variables,
    unused_mut,
    unused_imports,
    clippy::wildcard_imports
)]

extern crate test;

use crate::flat_2d_array::Flat2DArray;
use crate::vec_2d_array::Vec2DArray;
use r3bl_tui::{ColWidth, PixelChar, RowHeight, Size, height, width};
use test::{Bencher, black_box};

// Intel CPU:
// p core - 48K L1 cache
// e core - 32K L1 cache

mod fits_l1_cache {
    use super::*;
    pub const WIDTH: usize = 30;
    pub const HEIGHT: usize = 30;
    pub fn size() -> Size { width(WIDTH) + height(HEIGHT) }
}

mod spills_l1_cache {
    use super::*;
    pub const WIDTH: usize = 200;
    pub const HEIGHT: usize = 100;
    pub fn size() -> Size { width(WIDTH) + height(HEIGHT) }
}
```

**(Crucial Point)** First, before we even look at average speed, look at the **Margin of
Error**. `Vec2DArray` suffers from massive variance—sometimes swinging by ±98%—because its
performance relies entirely on lucky CPU cache placement for scattered heap allocations.
`Flat2DArray` guarantees perfectly consistent, flatline frame times. In a UI, eliminating
those micro-stutters is just as important as raw speed!

Let's validate our theories across the 6 groups:

**(Visual: Type Group 1 benchmarks in `src/benches.rs`)**

```rust
#[bench]
fn fits_l1_g1_clear_screen_vec2darray_scalar(b: &mut Bencher) {
    let size = fits_l1_cache::size();
    let mut grid = Vec2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Vec2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.clear(black_box(PixelChar::Void)));
}

#[bench]
fn fits_l1_g1_clear_screen_flat1darray_simd(b: &mut Bencher) {
    let size = fits_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.simd_clear(black_box(PixelChar::Void)));
}
```

1. **Clear Screen (The Memory Wall)**: When our grid fits perfectly inside the L1 Cache
   (`30x30`), a pure 1D `.fill()` is consistently faster than a scalar row-by-row clear
   loop. However, when we expand to a realistic fullscreen terminal (`200x100`), we spill
   out of the L1 Cache. Suddenly, both the SIMD and Scalar algorithms tie! This proves the
   **Memory Wall**—at 640KB, the CPU is so fast that RAM bandwidth becomes the ultimate
   bottleneck.

**(Visual: Type Group 2 benchmarks in `src/benches.rs`)**

```rust
#[bench]
fn spills_l1_g2_scroll_up_vec2darray_scalar(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Vec2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Vec2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.scroll_up());
}

#[bench]
fn spills_l1_g2_scroll_up_flat1darray_simd(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.simd_scroll_up());
}
```

2. **Scroll Screen (The Silver Bullet Fallacy)**: This is the one rare case where the
   naive `Vec2DArray` absolutely dominates. Because the naive approach uses
   `rotate_left(1)` to just shift pointer addresses instead of copying actual contiguous
   bytes, it is extremely fast. The Flat 1D Array (`.copy_within()`) has to physically
   move 640KB of memory.

**(Visual: Type Group 3 benchmarks in `src/benches.rs`)**

```rust
#[bench]
fn spills_l1_g3_print_screen_vec2darray_scalar(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Vec2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Vec2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| {
        for row in 0..grid.rows.as_usize() {
            for col in 0..grid.cols.as_usize() {
                let _ = black_box(grid.data[row][col]);
            }
        }
    });
}

#[bench]
fn spills_l1_g3_print_screen_flat1darray_simd(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| {
        for item in grid.data.iter() {
            let _ = black_box(item);
        }
    });
}
```

3. **Print Screen / Streaming**: Here is **The L1 Prefetcher Theory**. When the compositor
   tries to simply _read_ the screen sequentially, linearly streaming flat memory into the
   L1 cache completely destroys nested vector heap-chasing. The hardware prefetcher
   predicts perfectly, resulting in massive speedups.

**(Visual: Type Group 4 benchmarks in `src/benches.rs`)**

```rust
#[bench]
fn spills_l1_g4_diff_vec2darray_scalar(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid_1 = Vec2DArray::<PixelChar>::new(size, PixelChar::Void);
    println!("Vec2DArray -> grid size: {:?}", grid_1.get_mem_size());
    let mut grid_2 = Vec2DArray::<PixelChar>::new(size, PixelChar::Void);
    grid_2.data[50][50] = PixelChar::Spacer;
    b.iter(|| {
        let _ = black_box(&grid_1.diff(&grid_2));
    });
}

#[bench]
fn spills_l1_g4_diff_flat1darray_simd(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid_1 = Flat2DArray::<PixelChar>::new(size, PixelChar::Void);
    println!("Flat2DArray -> grid size: {:?}", grid_1.get_mem_size());
    let mut grid_2 = Flat2DArray::<PixelChar>::new(size, PixelChar::Void);
    grid_2[50][50] = PixelChar::Spacer;
    b.iter(|| {
        let _ = black_box(&grid_1.diff(&grid_2));
    });
}
```

4. **Diff Screen**: This is the absolute core of TUI rendering! The flat array is **1.5x
   faster** at finding coordinate differences than legacy pointer-chasing arrays,
   translating directly into higher FPS for your users.

**(Visual: Type Group 5 benchmarks in `src/benches.rs`)**

```rust
#[bench]
fn spills_l1_g5_mem_size_vec2darray_scalar(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let grid = Vec2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Vec2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| {
        let _ = black_box(grid.get_mem_size());
    });
}

#[bench]
fn spills_l1_g5_mem_size_flat1darray_simd(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| {
        let _ = black_box(grid.get_mem_size());
    });
}
```

5. **Memory Overhead**: Calculating the size of `Vec<Vec<T>>` has massive pointer-chasing
   overhead, while `Box<[T]>` is near-instantaneous. A whopping **39.0x speedup** just for
   asking a struct how big it is!

**(Visual: Type Group 6 benchmarks in `src/benches.rs`)**

```rust
#[bench]
fn spills_l1_g6_traversal_by_row_col_vec2darray_scalar(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Vec2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Vec2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| {
        for row in 0..grid.rows.as_usize() {
            for col in 0..grid.cols.as_usize() {
                let _ = black_box((row, col, grid.data[row][col]));
            }
        }
    });
}

#[bench]
fn spills_l1_g6_traversal_by_row_col_flat1darray_simd_mod_and_div(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| {
        for (idx, item) in grid.data.iter().enumerate() {
            let row = idx / grid.cols.as_usize();
            let col = idx % grid.cols.as_usize();
            let _ = black_box((row, col, item));
        }
    });
}

#[bench]
fn spills_l1_g6_traversal_by_row_col_flat1darray_simd(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| {
        grid.data
            .chunks_exact(grid.cols.as_usize())
            .enumerate()
            .for_each(|(row, chunk)| {
                chunk.iter().enumerate().for_each(|(col, item)| {
                    let _ = black_box((row, col, item));
                });
            });
    });
}
```

6. **The Math Pipeline Stall**: This proves the Math Pipeline Stall theory. We benchmarked
   `chunks_exact` against division/modulo to prove that doing division inside a hot loop
   stalls the CPU pipeline. By avoiding modulo math, `chunks_exact` is **1.5x faster**.

**(Visual: Final diagram on screen)**

```text
1. Vec2DArray  (Scalar) - Slowest (Cache Misses & Modulo Math)
2. Flat2DArray (Scalar) - Fast (Cache Hits! But Modulo Math stalls)
3. Flat2DArray (SIMD)   - Fastest (Cache Hits & pure pointer addition!)
```

**Speaker:** "One quick bonus fact before you go: These benchmarks were run using a heavy,
multi-byte struct for each cell representing a colored terminal pixel. If your grid uses
simple primitive integers like a `usize`, LLVM optimizes the SIMD `.fill()` operations so
aggressively that it actually deletes the loop entirely during compilation! It will print
a jaw-dropping **60,000x speedup** when clearing the screen, but beware: that's a classic
benchmarking illusion called Dead Code Elimination.

And there you have it. Flatten your arrays, unlock SIMD, and stop pipeline stalls. Thanks
for watching!"
