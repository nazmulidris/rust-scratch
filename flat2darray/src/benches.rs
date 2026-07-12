// Copyright (c) 2026 Nazmul Idris. Licensed under Apache License, Version 2.0.

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

    const WIDTH: usize = 30;
    const HEIGHT: usize = 30;

    pub fn size() -> Size {
        width(WIDTH) + height(HEIGHT)
    }
}

mod spills_l1_cache {
    use super::*;

    const WIDTH: usize = 200;
    const HEIGHT: usize = 100;

    pub fn size() -> Size {
        width(WIDTH) + height(HEIGHT)
    }
}

// GROUP 1: Clear Screen Benchmarks
//
// Fits in L1 Cache (30x30) Benchmarks:
// - 1. Flat1DArray clear (simd)
// - 2. Flat1DArray clear (scalar)
// - 3. Vec2DArray clear  (scalar)
//
// Spills L1 Cache (200x100) Benchmarks:
// - 1. Flat1DArray clear (simd)
// - 2. Vec2DArray clear  (scalar)
// - 3. Flat1DArray clear (scalar)

#[bench]
fn fits_l1_g1_clear_screen_vec2darray_scalar(b: &mut Bencher) {
    let size = fits_l1_cache::size();
    let mut grid = Vec2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Vec2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.clear(black_box(PixelChar::Void)));
}

#[bench]
fn fits_l1_g1_clear_screen_flat1darray_scalar(b: &mut Bencher) {
    let size = fits_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.clear(black_box(PixelChar::Void)));
}

#[bench]
fn fits_l1_g1_clear_screen_flat1darray_simd(b: &mut Bencher) {
    let size = fits_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.simd_clear(black_box(PixelChar::Void)));
}

#[bench]
fn spills_l1_g1_clear_screen_vec2darray_scalar(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Vec2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Vec2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.clear(black_box(PixelChar::Void)));
}

#[bench]
fn spills_l1_g1_clear_screen_flat1darray_scalar(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.clear(black_box(PixelChar::Void)));
}

#[bench]
fn spills_l1_g1_clear_screen_flat1darray_simd(b: &mut Bencher) {
    let size = spills_l1_cache::size();
    let mut grid = Flat2DArray::<PixelChar>::new(size, PixelChar::default());
    println!("Flat2DArray -> grid size: {:?}", grid.get_mem_size());
    b.iter(|| grid.simd_clear(black_box(PixelChar::Void)));
}

// GROUP 2: Scroll Up Benchmarks
//
// Spills L1 Cache (200x100) Benchmarks:
// - 1. Vec2DArray scroll_up  (scalar)
// - 2. Flat1DArray scroll_up (simd)

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

// GROUP 3: Print Screen Benchmarks
//
// Spills L1 Cache (200x100) Benchmarks:
// - 1. Flat1DArray print_screen (simd)
// - 2. Vec2DArray print_screen  (scalar)

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

// GROUP 4: Diff Benchmarks
//
// Spills L1 Cache (200x100) Benchmarks:
// - 1. Flat1DArray print_screen (simd) -> much tighter variance
// - 2. Vec2DArray print_screen  (scalar)

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

// GROUP 5: Memory Size Benchmarks
//
// Spills L1 Cache (200x100) Benchmarks:
// - 1. Flat1DArray print_screen (simd) -> much tighter variance
// - 2. Vec2DArray print_screen  (scalar)

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

// GROUP 6: Traversal By Row and Col Benchmarks (similar to Group 3)
//
// Spills L1 Cache (200x100) Benchmarks:
// - 1. Flat1DArray print_screen (simd) -> much tighter variance
// - 2. Vec2DArray print_screen  (scalar)
// - 3. Flat1DArray print_screen (scalar)

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
