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

/// The [Slice::split_first()] is equivalent to this function.
pub fn split_first<T>(slice: &[T]) -> Option<(&T, &[T])> {
    if slice.is_empty() {
        None
    } else {
        Some((&slice[0], &slice[1..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_first() {
        let slice = [1, 2, 3];
        let result = split_first(&slice);

        // Compare to array slice.
        assert_eq!(Some((&1, [2, 3].as_slice())), result);

        // Compare to slice from slice.
        assert_eq!(Some((&1, &slice[1..])), result);
    }
}
