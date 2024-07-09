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

#[tokio::test]
async fn test_sleep_right_and_wrong_ways() {
    let sleep_time = 100;
    let sleep = tokio::time::sleep(std::time::Duration::from_millis(sleep_time));
    tokio::pin!(sleep);
    let mut count = 5;

    loop {
        tokio::select! {
            // This branch is executed a non deterministic number of times. This is
            // because the sleep future is not pinned. It is dropped when the other branch
            // is executed. Then on the next iteration, a new sleep future is created.
            _ = tokio::time::sleep(std::time::Duration::from_millis(sleep_time)) => {
                println!("sleep");
            }
            // This branch executes a deterministic number of times. The same sleep future
            // is re-used on each iteration.
            _ = &mut sleep => {
                println!("tick : {count}");
                count -= 1;
                if count == 0 {
                    break;
                }
            }
        }
    }
}

#[tokio::test]
async fn test_safe_cancel_example() {
    let sleep_time = 100;
    let mut count = 5;
    let sleep = tokio::time::sleep(std::time::Duration::from_millis(sleep_time));
    tokio::pin!(sleep);

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let mut vec: Vec<usize> = vec![];

    loop {
        tokio::select! {
            _ = &mut sleep => {
                println!("tick : {count}");
                vec.push(count);

                count = count.saturating_sub(1);

                if count == 0 {
                    _ = tx.send(()).await;
                }
            }
            _ = rx.recv() => {
                    println!("shut down");
                    break;
            }
        }
    }

    assert_eq!(vec, vec![5, 4, 3, 2, 1]);
}

#[cfg(test)]
pub mod test_unsafe_cancel_example {
    use async_stream::stream;
    use futures_core::Stream;
    use futures_util::StreamExt;

    use std::{io::Error, pin::Pin};
    pub type PinnedInputStream = Pin<Box<dyn Stream<Item = Result<usize, Error>>>>;

    /// There's a 100ms delay between each event.
    pub fn gen_input_stream() -> PinnedInputStream {
        let it = stream! {
            for event in get_input_vec() {
                // wait for 100ms
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                println!("yielding event: {event}");
                yield Ok(event);
            }
        };
        Box::pin(it)
    }

    pub fn get_input_vec() -> Vec<usize> {
        vec![1, 2, 3, 4]
    }

    /// This is just to see how to use the async stream [gen_input_stream()].
    #[tokio::test]
    async fn test_generate_event_stream_pinned() {
        let mut count = 0;
        let mut stream = gen_input_stream();
        while let Some(item) = stream.next().await {
            let lhs = item.unwrap();
            let rhs = get_input_vec()[count];
            assert_eq!(lhs, rhs);
            count += 1;
        }
    }

    /// This function reads 3 items from the stream. It awaits for each item to be ready.
    /// This is not cancel safe. If this future is cancelled in the middle of reading the
    /// items, the stream will not have the items that are supposed to be added to the
    /// `vec`. But the `vec` will be dropped.
    async fn read_3_items_not_cancel_safe(stream: &mut PinnedInputStream) -> Vec<usize> {
        // This is state that is contained in this future. So when this future is dropped
        // before it gets a chance to finish, whatever is in this vec will be dropped.
        let mut vec = vec![];

        for _ in 0..3 {
            let item = stream.next().await.unwrap().unwrap();
            vec.push(item);
        }

        vec
    }

    #[tokio::test]
    async fn test_unsafe_cancel_example() {
        let mut stream = gen_input_stream();
        let sleep_time = 300;
        let sleep = tokio::time::sleep(std::time::Duration::from_millis(sleep_time));
        tokio::pin!(sleep);

        loop {
            tokio::select! {
                _ = &mut sleep => {
                    println!("time is up - end");
                    break;
                }
                it = read_3_items_not_cancel_safe(&mut stream) => {
                    println!("got 3 items: {it:?}");
                }
            }
        }

        // Note that the [1, 2] is consumed, and dropped, now the stream is at [3, 4, 5].
        let it = stream.next().await.unwrap().unwrap();
        assert_eq!(it, 3);
    }
}
