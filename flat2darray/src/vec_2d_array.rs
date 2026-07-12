// Copyright (c) 2026 Nazmul Idris. Licensed under Apache License, Version 2.0.

#![allow(dead_code, unused_imports)]

use r3bl_tui::{ColIndex, ColWidth, Pos, RowHeight, RowIndex, Size, height, width};

/// ```text
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
/// ```
#[derive(Debug, Clone)]
pub struct Vec2DArray<T: Clone> {
    pub data: Vec<Vec<T>>,
    pub rows: RowHeight,
    pub cols: ColWidth,
}

impl<T: Clone> Vec2DArray<T> {
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

    pub fn clear(&mut self, it: T) {
        for row in 0..self.rows.as_usize() {
            for col in 0..self.cols.as_usize() {
                self.data[row][col] = it.clone();
            }
        }
    }

    /// # ASCII Diagram
    /// ```text
    /// Before:         After:
    /// [ Row 0 ]       [ Row 1 ]  <-- Shifted up
    /// [ Row 1 ]  ==>  [ Row 2 ]
    /// [ Row 2 ]       [ Row 2 ]  <-- Duplicated from above
    /// ```
    pub fn scroll_up(&mut self) {
        if self.data.is_empty() {
            return;
        }

        self.data.rotate_left(1);

        // Duplicate the last row from the second-to-last row.
        let len = self.data.len();
        if len > 1 {
            let copy_of_2nd_last_row = self.data[len - 2].clone();
            self.data[len - 1] /* last row */ = copy_of_2nd_last_row;
        }
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

impl<T: Clone + PartialEq> Vec2DArray<T> {
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
}

#[cfg(test)]
#[allow(dead_code, unused_imports)]
mod tests {
    use r3bl_tui::{col, row};

    use super::*;

    #[test]
    fn test_new() {
        let w = width(10);
        let h = height(10);
        let _grid = Vec2DArray::new(w + h, 0usize);
    }

    #[test]
    fn test_constructor() {
        let rows = RowHeight::new(3);
        let cols = ColWidth::new(4);
        let default_value = 0;
        let array = Vec2DArray::new(rows + cols, default_value);
        for row in 0..rows.as_usize() {
            for col in 0..cols.as_usize() {
                assert_eq!(array.data[row][col], default_value);
            }
        }
    }

    #[test]
    fn test_clear() {
        let rows = RowHeight::new(2);
        let cols = ColWidth::new(3);
        let default_value = 0;
        let mut array = Vec2DArray::new(rows + cols, default_value);
        let new_value = 5;
        array.clear(new_value);
        for row in 0..rows.as_usize() {
            for col in 0..cols.as_usize() {
                assert_eq!(array.data[row][col], new_value);
            }
        }
    }

    #[test]
    fn test_scroll_up() {
        let rows = RowHeight::new(3);
        let cols = ColWidth::new(2);
        let default_value: usize = 0;

        let mut array = Vec2DArray::new(rows + cols, default_value);

        // Fill the array with distinct values for testing.
        for row in 0..rows.as_usize() {
            for col in 0..cols.as_usize() {
                array.data[row][col] = row * cols.as_usize() + col;
            }
        }

        // Scroll up and check the values.
        array.scroll_up();
        assert_eq!(array.data[0], vec![2, 3]); // Row 1 becomes Row 0
        assert_eq!(array.data[1], vec![4, 5]); // Row 2 becomes Row 1
        assert_eq!(array.data[2], vec![4, 5]); // Last row duplicates the new last row
    }

    #[test]
    fn test_get_mem_size() {
        let rows = RowHeight::new(2);
        let cols = ColWidth::new(3);
        let default_value: usize = 0;
        let array = Vec2DArray::new(rows + cols, default_value);

        let expected_size = std::mem::size_of::<Vec2DArray<usize>>()
            + array.data.capacity() * std::mem::size_of::<Vec<usize>>()
            + array
                .data
                .iter()
                .map(|row| row.capacity() * std::mem::size_of::<usize>())
                .sum::<usize>();

        assert_eq!(array.get_mem_size(), expected_size);
    }

    #[test]
    fn test_diff() {
        let rows = RowHeight::new(2);
        let cols = ColWidth::new(2);
        let default_value: usize = 0;

        let array1 = Vec2DArray::new(rows + cols, default_value);
        let mut array2 = array1.clone();

        // Modify some values in array2.
        array2.data[0][1] = 1;
        array2.data[1][0] = 2;

        let changes = array1.diff(&array2);
        assert_eq!(changes.len(), 2);
        assert!(changes.contains(&(row(0) + col(1))));
        assert!(changes.contains(&(row(1) + col(0))));
    }
}
