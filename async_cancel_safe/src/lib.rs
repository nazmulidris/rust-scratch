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

/// Equivalent to [test_sleep_right_and_wrong_ways_v2]. This test uses
/// [`tokio::pin`] and [`tokio::time::sleep`].
/// Run the test using:
/// `cargo test -- --nocapture test_sleep_right_and_wrong_ways_v1`
#[tokio::test]
async fn test_sleep_right_and_wrong_ways_v1() {
    let mut count = 5;

    let sleep_time = 100;
    let duration = std::time::Duration::from_millis(sleep_time);

    let sleep = tokio::time::sleep(duration);
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            // Branch 1 (right way)
            // This branch executes a deterministic number of times. The same
            // sleep future is re-used on each iteration.
            _ = &mut sleep => {
                println!("branch 1 - tick : {count}");
                count -= 1;
                if count == 0 {
                    break;
                }
            }

            // Branch 2 (wrong way)
            // This branch is executed a non deterministic number of times.
            // This is because the sleep future is not pinned. It is dropped
            // when the other branch is executed. Then on the next iteration,
            // a new sleep future is created.
            _ = tokio::time::sleep(duration) => {
                println!("branch 2 - sleep");
            }
        }
    }
}

/// Equivalent to [test_sleep_right_and_wrong_ways_v1]. This test uses
/// [`tokio::time::interval()`]
/// Run the test using:
/// `cargo test -- --nocapture test_sleep_right_and_wrong_ways_v2`
#[tokio::test]
async fn test_sleep_right_and_wrong_ways_v2() {
    let mut count = 5;

    let sleep_time = 100;
    let duration = std::time::Duration::from_millis(sleep_time);

    let mut interval = tokio::time::interval(duration);

    loop {
        tokio::select! {
            // Branch 1 (right way)
            // This branch executes a deterministic number of times. The same
            // sleep future is re-used on each iteration.
            _ = interval.tick() => {
                println!("branch 1 - tick : {count}");
                count -= 1;
                if count == 0 {
                    break;
                }
            }

            // Branch 2 (wrong way)
            // This branch is executed a non deterministic number of times.
            // This is because the sleep future is not pinned. It is dropped
            // when the other branch is executed. Then on the next iteration,
            // a new sleep future is created.
            _ = tokio::time::sleep(duration) => {
                println!("branch 2 - sleep");
            }
        }
    }
}

/// Run the test using:
/// `cargo test -- --nocapture test_safe_cancel_example`
#[tokio::test]
async fn test_safe_cancel_example() {
    let sleep_time = 100;
    let duration = std::time::Duration::from_millis(sleep_time);

    let mut count = 5;
    let mut interval = tokio::time::interval(duration);

    // Shutdown channel.
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let mut vec: Vec<usize> = vec![];

    loop {
        tokio::select! {
            // Branch 1.
            _ = interval.tick() => {
                println!("branch 1 - tick : count {}", count);

                vec.push(count);
                count = count.saturating_sub(1);

                if count == 0 {
                    _ = tx.try_send(());
                }
            }
            // Branch 2.
            _ = rx.recv() => {
                println!("branch 2 => shut down");
                break;
            }
        }
    }

    assert_eq!(vec, vec![5, 4, 3, 2, 1]);
}

#[cfg(test)]
pub mod test_unsafe_cancel_example {
    use r3bl_test_fixtures::{gen_input_stream_with_delay, PinnedInputStream};

    pub fn get_input_vec() -> Vec<usize> {
        vec![1, 2, 3, 4]
    }

    pub fn get_stream_delay() -> std::time::Duration {
        std::time::Duration::from_millis(100)
    }

    fn get_input_stream() -> PinnedInputStream<usize> {
        gen_input_stream_with_delay(get_input_vec(), get_stream_delay())
    }

    /// This is just to see how to use the async stream [gen_input_stream()].
    #[tokio::test]
    async fn test_generate_event_stream_pinned() {
        use futures_util::StreamExt;

        let mut count = 0;
        let mut stream = get_input_stream();

        while let Some(item) = stream.next().await {
            let lhs = item;
            let rhs = get_input_vec()[count];
            assert_eq!(lhs, rhs);
            count += 1;
        }
    }

    /// There is no need to [futures_util::FutureExt::fuse()] the items in each
    /// [tokio::select!] branch. This is because Tokio's event loop is designed to handle
    /// this efficiently by remembering the state of each future across iterations.
    ///
    /// More info: <https://gemini.google.com/app/e55fd62339b674fb>
    #[rustfmt::skip]
    async fn read_3_items_not_cancel_safe(stream: &mut PinnedInputStream<usize>)
        -> Vec<usize>
    {
        use futures_util::StreamExt;
        let mut vec = vec![];

        println!("branch 2 => entering read_3_items_not_cancel_safe");

        for _ in 0..3 {
            let item = stream.next() /* .fuse() */ .await.unwrap();
            println!("branch 2 => read_3_items_not_cancel_safe got item: {item}");
            vec.push(item);
            println!("branch 2 => vec so far contains: {vec:?}");
        }

        vec
    }

    /// There is no need to [futures_util::FutureExt::fuse()] the items in each
    /// [tokio::select!] branch. This is because Tokio's event loop is designed to handle
    /// this efficiently by remembering the state of each future across iterations.
    ///
    /// More info: <https://gemini.google.com/app/e55fd62339b674fb>
    #[tokio::test]
    async fn test_unsafe_cancel_stream() {
        use futures_util::StreamExt;

        let mut stream = get_input_stream();
        let sleep_time = 300;
        let duration = std::time::Duration::from_millis(sleep_time);
        let sleep = tokio::time::sleep(duration);
        tokio::pin!(sleep);

        loop {
            tokio::select! {
                // Branch 1 - Timeout.
                _ = &mut sleep => {
                    println!("branch 1 - time is up - end");
                    break;
                }
                // Branch 2 - Read from stream.
                it = read_3_items_not_cancel_safe(&mut stream) /* .fuse() */ => {
                    println!("branch 2 - got 3 items: {it:?}");
                }
            }
        }

        println!("loop exited");

        // Only [1, 2] is consumed by Branch 2 before the timeout happens
        // in Branch 1.
        let it = stream.next().await.unwrap();
        assert_eq!(it, 3);
    }
}
