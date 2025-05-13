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

// TODO: show stack alloc ring buffer
// TODO: show heap alloc ring buffer
// TODO: And pre-allocate using the pattern buffer: [Option<T>; N], pattern
// TODO: this constructor magic: buffer: [(); N].map(|_| None),
// TODO: this generic header: pub struct RingBuffer<T, const N: usize>
