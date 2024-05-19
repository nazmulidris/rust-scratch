/*
 *   Copyright (c) 2024 Nazmul Idris
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

use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;

pub type PinnedInputStream = Pin<Box<dyn Stream<Item = Result<String, String>>>>;

pub fn gen_input_stream() -> PinnedInputStream {
    let it = async_stream::stream! {
        for event in get_input_vec() {
            yield Ok(event);
        }
    };
    Box::pin(it)
}

pub fn get_input_vec() -> Vec<String> {
    vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
    ]
}

#[tokio::test]
async fn test_stream() {
    let mut count = 0;
    let mut it = gen_input_stream();
    while let Some(event) = it.next().await {
        let lhs = event.unwrap();
        let rhs = get_input_vec()[count].clone();
        assert_eq!(lhs, rhs);
        count += 1;
    }
}
