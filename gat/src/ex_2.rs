/*
 *   Copyright (c) 2025 Nazmul Idris
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

trait StreamingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

struct OverlappingSlices<'a> {
    data: &'a mut [usize],
    window_size: usize,
    position: usize,
}

impl<'a> OverlappingSlices<'a> {
    fn new(data: &'a mut [usize], window_size: usize) -> Self {
        Self {
            data,
            window_size,
            position: 0,
        }
    }
}

impl<'a> StreamingIterator for OverlappingSlices<'a> {
    type Item<'b>
        = &'b mut [usize]
    where
        Self: 'b;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>> {
        if self.position + self.window_size <= self.data.len() {
            let slice = &mut self.data[self.position..self.position + self.window_size];
            self.position += 1;
            Some(slice)
        } else {
            None
        }
    }
}

#[test]
fn test_overlapping_slices() {
    let mut data = vec![1, 2, 3, 4, 5];
    let mut iterator = OverlappingSlices::new(&mut data, 3);

    while let Some(slice) = iterator.next() {
        println!("Slice: {:?}", slice);
        // Example mutation to demonstrate mutable access
        slice[0] += 10;
    }

    println!("Modified data: {:?}", data);
}
