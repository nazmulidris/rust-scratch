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

/// A fixed-size ring buffer implementation.
///
/// The `RingBuffer` struct is a generic data structure that allows for efficient
/// insertion and removal of elements in a circular buffer. It maintains a fixed capacity
/// and overwrites the oldest elements when the buffer is full. It behaves like a queue
/// with a fixed size ([Self::add] and [Self::remove]).
///
/// # Type Parameters
///
/// * `T`: The type of elements stored in the ring buffer.
/// * `N`: The fixed capacity of the ring buffer.
///
/// # Fields
///
/// * `internal_storage`: An array of `Option<T>` used to store the elements.
/// * `head`: The index of the next insertion point.
/// * `tail`: The index of the next removal point.
/// * `count`: The current number of elements in the buffer.
///
/// # Modules
///
/// * `constructor`: Contains the implementation of the `new` method and the `Default`
///   trait.
/// * `mutator`: Contains methods for inserting and removing elements.
/// * `size`: Contains methods for querying the size and state of the buffer.
/// * `iterator`: Contains the implementation of an iterator for the ring buffer.
///
/// # Examples
///
/// ```
/// use ringbuffer::RingBuffer;
///
/// let mut ring_buffer: RingBuffer<i32, 3> = RingBuffer::new();
///
/// ring_buffer.add(1);
/// ring_buffer.add(2);
/// ring_buffer.add(3);
///
/// assert_eq!(ring_buffer.len(), 3);
/// assert_eq!(ring_buffer.is_full(), true);
/// assert_eq!(ring_buffer.iter().collect::<Vec<&i32>>(), vec![&1, &2, &3]);
///
/// ring_buffer.remove();
///
/// assert_eq!(ring_buffer.len(), 2);
/// assert_eq!(ring_buffer.is_empty(), false);
/// assert_eq!(ring_buffer.iter().collect::<Vec<&i32>>(), vec![&2, &3]);
/// ```

#[derive(Debug, PartialEq)]
pub struct RingBuffer<T, const N: usize> {
    internal_storage: [Option<T>; N],
    head: usize,
    tail: usize,
    count: usize,
}

mod constructor {
    use super::*;

    impl<T, const N: usize> Default for RingBuffer<T, N> {
        fn default() -> Self {
            Self::new()
        }
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
    }
}

mod mutator {
    use super::*;

    impl<T, const N: usize> RingBuffer<T, N> {
        /// Insert at head (ie, insert the newest item).
        pub fn add(&mut self, value: T) {
            if self.count == N
            // Wrap around. Update both head and tail.
            {
                self.internal_storage[self.head] = Some(value);
                self.head = (self.head + 1) % N;
                self.tail = (self.tail + 1) % N;
            }
            // Normal insert. Don't touch the tail.
            else {
                self.internal_storage[self.head] = Some(value);
                self.head = (self.head + 1) % N;
                self.count += 1;
            }
        }

        /// Remove from tail (ie, remove the oldest item).
        pub fn remove(&mut self) -> Option<T> {
            if self.count == 0 {
                return None;
            }

            let value = self.internal_storage[self.tail].take();
            self.tail = (self.tail + 1) % N;
            self.count -= 1;

            value
        }

        /// Clear all items. This does not affect memory allocation (the capacity remains
        /// the same). The `internal_storage` array is not modified, so there maybe stale
        /// data in there, which does not affect the behavior of the ring buffer
        /// ([Self::iter], [Self::add], [Self::remove], etc.) work as expected.
        pub fn clear(&mut self) {
            self.head = 0;
            self.tail = 0;
            self.count = 0;
        }
    }
}

mod size {
    use super::*;

    impl<T, const N: usize> RingBuffer<T, N> {
        pub fn len(&self) -> usize {
            self.count
        }

        pub fn is_empty(&self) -> bool {
            self.count == 0
        }

        pub fn is_full(&self) -> bool {
            self.count == N
        }
    }
}

mod iterator {
    use super::*;
    pub struct RingBufferIterator<'a, T, const N: usize> {
        ring_buffer: &'a RingBuffer<T, N>,
        iterator_index: usize,
    }

    impl<'a, T, const N: usize> Iterator for RingBufferIterator<'a, T, N> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            if self.iterator_index == self.ring_buffer.count {
                return None;
            }

            let actual_index = (self.ring_buffer.tail + self.iterator_index) % N;
            self.iterator_index += 1;

            self.ring_buffer.internal_storage[actual_index].as_ref()
        }
    }

    impl<T, const N: usize> RingBuffer<T, N> {
        pub fn iter(&self) -> RingBufferIterator<T, N> {
            RingBufferIterator {
                ring_buffer: self,
                iterator_index: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests_ring_buffer {
    use super::*;
    use smallstr::SmallString;

    pub type SmallStringBackingStore = SmallString<[u8; DEFAULT_SMALL_STRING_SIZE]>;
    pub const DEFAULT_SMALL_STRING_SIZE: usize = 32;

    #[test]
    fn test_ring_buffer_iter() {
        let mut ring_buffer: RingBuffer<SmallStringBackingStore, 3> = RingBuffer::new();

        {
            assert_eq!(ring_buffer.len(), 0);
            assert_eq!(ring_buffer.head, 0); // Empty.
            assert_eq!(ring_buffer.tail, 0); // Empty.
            assert_eq!(ring_buffer.count, 0); // Empty.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        // Normal insert.
        ring_buffer.add("Hello".into());

        {
            assert_eq!(ring_buffer.len(), 1);
            assert_eq!(ring_buffer.head, 1); // Head moves.
            assert_eq!(ring_buffer.tail, 0); // Tail does not move.
            assert_eq!(ring_buffer.count, 1); // Count increments
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "Hello");
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["Hello"]);
        }

        // Normal insert.
        ring_buffer.add("World".into());

        {
            assert_eq!(ring_buffer.len(), 2);
            assert_eq!(ring_buffer.head, 2); // Head moves.
            assert_eq!(ring_buffer.tail, 0); // Tail does not move.
            assert_eq!(ring_buffer.count, 2); // Count increments
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "Hello");
            assert_eq!(iter.next().unwrap(), "World");
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["Hello", "World"]);
        }

        // Normal insert.
        ring_buffer.add("Rust".into());

        {
            assert_eq!(ring_buffer.len(), 3);
            assert_eq!(ring_buffer.head, 0); // Head wraps around.
            assert_eq!(ring_buffer.tail, 0); // Tail does not move.
            assert_eq!(ring_buffer.count, 3); // Count increments
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "Hello");
            assert_eq!(iter.next().unwrap(), "World");
            assert_eq!(iter.next().unwrap(), "Rust");
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["Hello", "World", "Rust"]);
        }

        // Normal remove.
        ring_buffer.remove();

        {
            assert_eq!(ring_buffer.len(), 2);
            assert_eq!(ring_buffer.head, 0); // Head does not move.
            assert_eq!(ring_buffer.tail, 1); // Tail moves.
            assert_eq!(ring_buffer.count, 2); // Count decrements.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "World");
            assert_eq!(iter.next().unwrap(), "Rust");
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["World", "Rust"]);
        }

        // Wrap around! This will clobber "Hello". Head and tail move.
        ring_buffer.add("R3BL".into());

        {
            assert_eq!(ring_buffer.len(), 3);
            assert_eq!(ring_buffer.head, 1); // Wrapped around.
            assert_eq!(ring_buffer.tail, 1); // Tail does not move.
            assert_eq!(ring_buffer.count, 3); // Count is same.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "World");
            assert_eq!(iter.next().unwrap(), "Rust");
            assert_eq!(iter.next().unwrap(), "R3BL");
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["World", "Rust", "R3BL"]);
        }

        for it in ring_buffer.iter() {
            assert!(it == "R3BL" || it == "World" || it == "Rust");
        }

        let vec = ring_buffer
            .iter()
            .collect::<Vec<&SmallStringBackingStore>>();
        assert_eq!(vec, vec!["World", "Rust", "R3BL"]);
        assert_eq!(ring_buffer.head, 1); // Wrapped around.
        assert_eq!(ring_buffer.count, 3); // Count is same.

        // Wrapped around remove.
        ring_buffer.remove();

        {
            assert_eq!(ring_buffer.len(), 2);
            assert_eq!(ring_buffer.head, 1); // Head does not move.
            assert_eq!(ring_buffer.tail, 2); // Tail moves.
            assert_eq!(ring_buffer.count, 2); // Count decrements.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "Rust");
            assert_eq!(iter.next().unwrap(), "R3BL");
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["Rust", "R3BL"]);
        }

        // Wrapped around insert.
        ring_buffer.add("Java".into());

        {
            assert_eq!(ring_buffer.len(), 3);
            assert_eq!(ring_buffer.head, 2); // Head moves.
            assert_eq!(ring_buffer.tail, 2); // Tail does not move.
            assert_eq!(ring_buffer.count, 3); // Count is same.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "Rust");
            assert_eq!(iter.next().unwrap(), "R3BL");
            assert_eq!(iter.next().unwrap(), "Java");
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["Rust", "R3BL", "Java"]);
        }

        // Wrapped around remove.
        ring_buffer.remove();

        {
            assert_eq!(ring_buffer.len(), 2);
            assert_eq!(ring_buffer.head, 2); // Head does not move.
            assert_eq!(ring_buffer.tail, 0); // Tail wraps around.
            assert_eq!(ring_buffer.count, 2); // Count decrements.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "R3BL");
            assert_eq!(iter.next().unwrap(), "Java");
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["R3BL", "Java"]);
        }

        // Wrapped around remove.
        ring_buffer.remove();

        {
            assert_eq!(ring_buffer.len(), 1);
            assert_eq!(ring_buffer.head, 2); // Head does not move.
            assert_eq!(ring_buffer.tail, 1); // Tail moves.
            assert_eq!(ring_buffer.count, 1); // Count decrements.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "Java");
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["Java"]);
        }

        // Wrapped around remove.
        ring_buffer.remove();

        {
            assert_eq!(ring_buffer.len(), 0);
            assert_eq!(ring_buffer.head, 2); // Head does not move.
            assert_eq!(ring_buffer.tail, 2); // Tail does not move.
            assert_eq!(ring_buffer.count, 0); // Count is zero.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert!(vec.is_empty());
        }

        // Wrapped around insert.
        ring_buffer.add("C++".into());

        {
            assert_eq!(ring_buffer.len(), 1);
            assert_eq!(ring_buffer.head, 0); // Head wraps around.
            assert_eq!(ring_buffer.tail, 2); // Tail does not move.
            assert_eq!(ring_buffer.count, 1); // Count is one.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next().unwrap(), "C++");
            assert_eq!(iter.next(), None);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert_eq!(vec, vec!["C++"]);
        }

        // Clear.
        ring_buffer.clear();

        {
            assert_eq!(ring_buffer.len(), 0);
            assert_eq!(ring_buffer.head, 0); // Head does not move.
            assert_eq!(ring_buffer.tail, 0); // Tail does not move.
            assert_eq!(ring_buffer.count, 0); // Count is zero.
            let mut iter = ring_buffer.iter();
            assert_eq!(iter.next(), None);
            assert_eq!(ring_buffer.iter().count(), 0);

            let vec = ring_buffer
                .iter()
                .collect::<Vec<&SmallStringBackingStore>>();
            assert!(vec.is_empty());
        }
    }
}
