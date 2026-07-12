// Copyright (c) 2026 Nazmul Idris. Licensed under Apache License, Version 2.0.

#![allow(dead_code, unused_imports, clippy::wildcard_imports)]

use r3bl_tui::{ColIndex, ColWidth, Pos, RowHeight, RowIndex, Size, height, width};
use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

/// # Vec<Vec<T>> (Scattered Heap Memory):
///
/// [Ptr] -> [Ptr, Ptr, Ptr]
///           |    |    |
///           v    v    v
///        [Row1] [Row2] [Row3]  <-- Cache Misses!
///
/// # Box<[T]> (Contiguous Memory):
///
/// [Ptr] -> [Row1 | Row2 | Row3] <-- Perfect L1/L2 Cache Hits!
///
/// # 2D to 1D Mapping
///
/// The grid is stored row-by-row in a flat 1D slice.
///
/// To find the element at `(row, col)`, we skip `row` full rows of size `width`,
/// and then step forward by `col`.
///
/// 2D Grid:
///           col 0   col 1   col 2
///        ┌───────┬───────┬───────┐
///  row 0 │ idx 0 │ idx 1 │ idx 2 │
///        ├───────┼───────┼───────┤
///  row 1 │ idx 3 │ idx 4 │ idx 5 │
///        ├───────┼───────┼───────┤
///  row 2 │ idx 6 │ idx 7 │ idx 8 │
///        └───────┴───────┴───────┘
///
/// 1D Grid (equivalent):
///
///   row 0                   row 1                   row 1
///   col 0   col 1   col 2   col 0   col 1   col 2   col 0   col 1   col 2
/// ┌───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────┐
/// │ idx 0 │ idx 1 │ idx 2 │ idx 3 │ idx 4 │ idx 5 │ idx 6 │ idx 7 │ idx 8 │
/// └───────┴───────┴───────┴───────┴───────┴───────┴───────┴───────┴───────┘
///
/// Example: `(row 1, col 2)`
/// - `row_offset = row_index * width      = 1 * 3 = 3`
/// - `1d_index   = row_offset + col_index = 3 + 2 = 5`
///
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
#[derive(Debug, Clone, PartialEq)]
pub struct Flat2DArray<T: Copy + PartialEq> {
    pub data: Box<[T]>,
    pub rows: RowHeight,
    pub cols: ColWidth,
}

impl<T: Copy + PartialEq + Debug> Flat2DArray<T> {
    pub fn new(arg_size: impl Into<Size>, default: T) -> Self {
        let size = arg_size.into();
        let num_cells = size.row_height.as_usize() * size.col_width.as_usize();
        let vec_1d = vec![default; num_cells];
        Self {
            data: vec_1d.into_boxed_slice(),
            rows: size.row_height,
            cols: size.col_width,
        }
    }

    pub fn clear(&mut self, it: T) {
        let cols = self.cols.as_usize();
        let rows = self.rows.as_usize();
        for row in 0..rows {
            for col in 0..cols {
                let index = row * cols + col;
                self.data[index] = it;
            }
        }
    }

    pub fn simd_clear(&mut self, it: T) {
        self.data.fill(it);
    }

    pub fn get_mem_size(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();
        total += self.data.len() * std::mem::size_of::<T>();
        total
    }

    pub fn print_screen(&self) -> String {
        let mut buffer = String::new();

        let cols = self.cols.as_usize();
        for (index, item) in self.data.iter().enumerate() {
            // 2d -> 1d mapping (SLOW!)
            let row = index / cols;
            let col = index % cols;
            buffer.push_str(&format!("({row}, {col}): {item:?} | "));
        }

        buffer
    }

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

    pub fn diff(&self, other: &Self) -> Vec<Pos> {
        let mut changes = Vec::new();

        let cols = self.cols.as_usize();
        let rows = self.rows.as_usize();

        for row in 0..rows {
            for col in 0..cols {
                // 2d -> 1d mapping.
                let index = row * cols + col;
                if self.data[index] != other.data[index] {
                    let row: RowIndex = row.into();
                    let col: ColIndex = col.into();
                    changes.push(row + col);
                }
            }
        }

        changes
    }

    pub fn simd_diff(&self, other: &Self) -> Vec<Pos> {
        let mut changes = Vec::new();

        let cols /* chunk size / num cols / width */ = self.cols.as_usize();

        let self_rows_iter = self.data.chunks_exact(cols);
        let other_rows_iter = other.data.chunks_exact(cols);

        debug_assert!(
            self_rows_iter.remainder().is_empty(),
            "The data length should be a multiple of the number of columns."
        );
        debug_assert!(
            other_rows_iter.remainder().is_empty(),
            "The data length should be a multiple of the number of columns."
        );

        let zipped_rows_iter = self_rows_iter.zip(other_rows_iter).enumerate();
        for (row_idx, (self_row_chunk, other_row_chunk)) in zipped_rows_iter {
            if self_row_chunk != other_row_chunk {
                let zipped_cols_iter = self_row_chunk
                    .iter()
                    .zip(other_row_chunk.iter())
                    .enumerate();
                for (col_idx, (self_item, other_item)) in zipped_cols_iter {
                    if self_item != other_item {
                        let row: RowIndex = row_idx.into();
                        let col: ColIndex = col_idx.into();
                        changes.push(row + col);
                    }
                }
            }
        }

        changes
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
        let cols = self.cols.as_usize();
        let rows = self.rows.as_usize();

        if rows <= 1 {
            return;
        }

        // Shift all rows up by one.
        for row in 0..(rows - 1) {
            for col in 0..cols {
                // Row `row` will be replaced by the contents of row `row + 1`.
                let src_index = (row + 1) * cols + col;
                let dest_index = row * cols + col;
                // Move the value from the next row to the current row.
                self.data[dest_index] = self.data[src_index];
            }
        }

        // Duplicate the last row from the second-to-last row.
        for col in 0..cols {
            let src_index = (rows - 2) * cols + col;
            let dest_index = (rows - 1) * cols + col;
            self.data[dest_index] = self.data[src_index];
        }
    }

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
    pub fn simd_scroll_up(&mut self) {
        let src_range = self.cols.as_usize()..;
        self.data.copy_within(src_range, /*starting index*/ 0);
    }
}

mod impl_index_usize {
    use super::*;

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
}

mod impl_index_pos {
    use super::*;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use r3bl_tui::{col, row};

    #[test]
    fn test_new() {
        let w = width(10);
        let h = height(10);
        let s = w + h;
        println!("s: {:?}", s);

        let grid = Flat2DArray::new(s, 0usize);
        let grid: Box<[usize]> = grid.data;
        println!("grid: {:?}", grid);

        let other = vec![
            /*default value*/ 0usize;
            /*size*/100
        ];
        // let mut other = Vec::with_capacity(100);
        // other.resize(100, 0usize);

        let other: Box<[usize]> = other.into_boxed_slice();
        println!("other: {:?}", other);

        assert_eq!(/*100*/ grid.len(), /*100*/ other.len());
        assert_eq!(grid, other);
    }

    #[test]
    fn test_clear() {
        let w = width(10);
        let h = height(10);
        let s = w + h;
        let mut grid = Flat2DArray::new(s, 0usize);
        grid.clear(1usize);
        let other = vec![
            /*default value*/ 1usize;
            /*size*/100
        ]
        .into_boxed_slice();
        assert_eq!(grid.data, other);
    }

    #[test]
    fn test_get_mem_size() {
        let w = width(10);
        let h = height(10);
        let s = w + h;
        let grid = Flat2DArray::new(s, 0usize);
        let mem_size = grid.get_mem_size();
        assert_eq!(
            mem_size,
            std::mem::size_of::<Flat2DArray<usize>>() + 100 * std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn test_print_screen() {
        let w = width(3);
        let h = height(3);
        let s = w + h;
        let grid = Flat2DArray::new(s, 0usize);
        let screen_output = grid.print_screen();
        println!("screen_output: {:?}", screen_output);
        let expected_output = "(0, 0): 0 | (0, 1): 0 | (0, 2): 0 | \
            (1, 0): 0 | (1, 1): 0 | (1, 2): 0 | \
            (2, 0): 0 | (2, 1): 0 | (2, 2): 0 | ";
        assert_eq!(screen_output, expected_output);
    }

    #[test]
    fn test_diff() {
        let w = width(2);
        let h = height(2);
        let s = w + h;
        let array1 = Flat2DArray::new(s, 0usize);

        // Modify some elements in array2 to create differences.
        let mut array2 = array1.clone();
        array2.data[1] /* 1d */ = 1; /* Change (0, 1) */
        array2.data[2] /* 1d */ = 1; /* Change (1, 0) */

        let changes = array1.diff(&array2);
        assert_eq!(changes.len(), 2);
        assert!(changes.contains(&(row(0) + col(1))));
        assert!(changes.contains(&(row(1) + col(0))));
    }

    #[test]
    fn test_scroll_up() {
        let cols = width(2);
        let rows = height(3);
        let s = cols + rows;
        let mut array = Flat2DArray::new(s, 0usize);

        // Fill the array with distinct values for testing.
        for row in 0..rows.as_usize() {
            for col in 0..cols.as_usize() {
                // 2d -> 1d mapping.
                let index = row * cols.as_usize() + col;
                // Fill with distinct value.
                array.data[index] = row * cols.as_usize() + col;
            }
        }

        // Scroll up and check the values.
        array.scroll_up();
        assert_eq!(array.data[0], 2); // Row 1 becomes Row 0
        assert_eq!(array.data[1], 3); // Row 1 becomes Row 0
        assert_eq!(array.data[2], 4); // Row 2 becomes Row 1
        assert_eq!(array.data[3], 5); // Row 2 becomes Row 1
        assert_eq!(array.data[4], 4); // Last row duplicates the new last row
        assert_eq!(array.data[5], 5); // Last row duplicates the new last row
    }

    #[test]
    fn array_access_syntax() {
        use r3bl_tui::{col, row};
        let w = width(10);
        let h = height(10);
        let s = w + h;

        // Index by usize.
        {
            let grid = Flat2DArray::new(s, 0usize);
            let row_slice: &[usize] = &grid[0];
            let cell = row_slice[0];
            assert_eq!(cell, 0);
            assert_eq!(grid[0][0], cell);
        }

        // IndexMut by usize.
        {
            let mut grid_mut = Flat2DArray::new(s, 0usize);
            grid_mut[0][0] = 42;
            assert_eq!(grid_mut[0][0], 42);
        }

        // Index by Pos.
        {
            let grid = Flat2DArray::new(s, 0usize);
            let pos = row(0) + col(0);
            let row_slice = &grid[pos];
            let cell = row_slice[0];
            assert_eq!(cell, 0);
        }

        // IndexMut by Pos.
        {
            let mut grid_mut = Flat2DArray::new(s, 0usize);
            let pos = row(0) + col(0);
            grid_mut[pos][0] = 42;
            assert_eq!(grid_mut[pos][0], 42);
        }
    }

    #[test]
    fn test_simd_print_screen() {
        let w = width(3);
        let h = height(3);
        let s = w + h;
        let grid = Flat2DArray::new(s, 0usize);

        // SIMD/Vectorized version.
        let screen_output = grid.simd_print_screen();
        println!("screen_output: {:?}", screen_output);
        let expected_output_vectorized = "(0, 0): 0 | (0, 1): 0 | (0, 2): 0 | \
            (1, 0): 0 | (1, 1): 0 | (1, 2): 0 | \
            (2, 0): 0 | (2, 1): 0 | (2, 2): 0 | ";
        assert_eq!(screen_output, expected_output_vectorized);

        // Scalar version.
        let expected_output_scalar = grid.print_screen();
        assert_eq!(expected_output_vectorized, expected_output_scalar);
    }

    #[test]
    fn test_simd_diff() {
        let w = width(2);
        let h = height(2);
        let s = w + h;
        let self_array = Flat2DArray::new(s, 0usize);

        // Modify some elements in array2 to create differences.
        let mut other_array = self_array.clone();
        other_array.data[1] /* 1d */ = 1; /* Change (0, 1) */
        other_array.data[2] /* 1d */ = 1; /* Change (1, 0) */

        let simd_changes = self_array.simd_diff(&other_array);
        assert_eq!(simd_changes.len(), 2);
        assert!(simd_changes.contains(&(row(0) + col(1))));
        assert!(simd_changes.contains(&(row(1) + col(0))));

        // Compare to scalar diff to ensure they produce the same results.
        let scalar_changes = self_array.diff(&other_array);
        assert_eq!(simd_changes, scalar_changes);
    }

    #[test]
    fn test_simd_clear() {
        let w = width(10);
        let h = height(10);
        let s = w + h;

        let mut self_grid = Flat2DArray::new(s, 0usize);
        self_grid.simd_clear(1usize);

        let other_grid = vec![
            /*default value*/ 1usize;
            /*size*/100
        ]
        .into_boxed_slice();

        assert_eq!(self_grid.data, other_grid);
    }

    #[test]
    fn test_simd_scroll() {
        let cols = width(2);
        let rows = height(3);
        let s = cols + rows;
        let mut og_array = Flat2DArray::new(s, 0usize);
        for row in 0..rows.as_usize() {
            for col in 0..cols.as_usize() {
                // 2d -> 1d mapping.
                let index = row * cols.as_usize() + col;
                // Fill with distinct value.
                og_array.data[index] = row * cols.as_usize() + col;
            }
        }

        let array_1_simd_scroll = {
            let mut array = og_array.clone();
            // Scroll up and check the values.
            array.simd_scroll_up();
            assert_eq!(array.data[0], 2); // Row 1 becomes Row 0
            assert_eq!(array.data[1], 3); // Row 1 becomes Row 0
            assert_eq!(array.data[2], 4); // Row 2 becomes Row 1
            assert_eq!(array.data[3], 5); // Row 2 becomes Row 1}
            array
        };

        let array_2_scalar_scroll = {
            let mut array = og_array.clone();
            // Scroll up and check the values.
            array.scroll_up();
            assert_eq!(array.data[0], 2); // Row 1 becomes Row 0
            assert_eq!(array.data[1], 3); // Row 1 becomes Row 0
            assert_eq!(array.data[2], 4); // Row 2 becomes Row 1
            assert_eq!(array.data[3], 5); // Row 2 becomes Row 1}
            array
        };

        assert_eq!(array_1_simd_scroll.data, array_2_scalar_scroll.data);
    }
}

#[cfg(test)]
mod corner_cases {
    #[test]
    fn slow_division_corner_case_speedup_using_bit_ops() {
        const COLS: usize = 128; // 2^7
        const SHIFT_AMT: u32 = COLS.ilog2(); // 7
        const MASK_AMT: usize = COLS - 1; // 127

        // 1d index to 2d mapping NOT using division (bitshift) and modulo (bitwise AND).
        let index: usize = 5;
        let row = index >> SHIFT_AMT; // index / COLS
        let col = index & MASK_AMT; // index % COLS

        println!("index: {index}, row: {row}, col: {col}");
    }
}
