// Copyright (c) 2026 Nazmul Idris. Licensed under Apache License, Version 2.0.

// We need this for cargo bench to work.
#![cfg_attr(test, feature(test))]

#[cfg(test)]
mod benches;

// Attach files.
pub mod flat_2d_array;
pub mod vec_2d_array;
