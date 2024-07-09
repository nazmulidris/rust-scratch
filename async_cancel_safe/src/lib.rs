/// Equivalent to [test_sleep_right_and_wrong_ways_v2]. This test uses
/// [`tokio::pin`] and [`tokio::time::sleep`].
/// Run the test using `cargo test -- --nocapture test_sleep_right_and_wrong_ways_v1`.
#[tokio::test]
async fn test_sleep_right_and_wrong_ways_v1() {
    let mut count = 5;

    let sleep_time = 100;

    let sleep = tokio::time::sleep(std::time::Duration::from_millis(sleep_time));
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            // Branch 1 (right way)
            // This branch executes a deterministic number of times. The same sleep future
            // is re-used on each iteration.
            _ = &mut sleep => {
                println!("branch 1 - tick : {count}");
                count -= 1;
                if count == 0 {
                    break;
                }
            }

            // Branch 2 (wrong way)
            // This branch is executed a non deterministic number of times. This is
            // because the sleep future is not pinned. It is dropped when the other branch
            // is executed. Then on the next iteration, a new sleep future is created.
            _ = tokio::time::sleep(std::time::Duration::from_millis(sleep_time)) => {
                println!("branch 2 - sleep");
            }
        }
    }
}

/// Equivalent to [test_sleep_right_and_wrong_ways_v1]. This test uses
/// [`tokio::time::interval()`]
/// Run the test using `cargo test -- --nocapture test_sleep_right_and_wrong_ways_v2`.
#[tokio::test]
async fn test_sleep_right_and_wrong_ways_v2() {
    let mut count = 5;

    let sleep_time = 100;

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(sleep_time));

    loop {
        tokio::select! {
            // Branch 1 (right way)
            // This branch executes a deterministic number of times. The same sleep future
            // is re-used on each iteration.
            _ = interval.tick() => {
                println!("branch 1 - tick : {count}");
                count -= 1;
                if count == 0 {
                    break;
                }
            }

            // Branch 2 (wrong way)
            // This branch is executed a non deterministic number of times. This is
            // because the sleep future is not pinned. It is dropped when the other branch
            // is executed. Then on the next iteration, a new sleep future is created.
            _ = tokio::time::sleep(std::time::Duration::from_millis(sleep_time)) => {
                println!("branch 2 - sleep");
            }
        }
    }
}

/// Run the test using `cargo test -- --nocapture test_safe_cancel_example`.
#[tokio::test]
async fn test_safe_cancel_example() {
    let sleep_time = 100;
    let mut count = 5;
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(sleep_time));

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
    use async_stream::stream;
    use futures_core::Stream;
    use futures_util::StreamExt;

    use std::{io::Error, pin::Pin};
    pub type PinnedInputStream = Pin<Box<dyn Stream<Item = Result<usize, Error>>>>;

    pub fn get_input_vec() -> Vec<usize> {
        vec![1, 2, 3, 4]
    }

    /// There's a 100ms delay between each event.
    pub fn gen_input_stream() -> PinnedInputStream {
        let it = stream! {
            for item in get_input_vec() {
                // wait for 100ms
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                println!("yielding item: {item}");
                yield Ok(item);
            }
        };
        Box::pin(it)
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

    async fn read_3_items_not_cancel_safe(stream: &mut PinnedInputStream) -> Vec<usize> {
        let mut vec = vec![];

        println!("branch 2 => entering read_3_items_not_cancel_safe");

        for _ in 0..3 {
            let item = stream.next().await.unwrap().unwrap();
            println!("branch 2 => read_3_items_not_cancel_safe got item: {item}");
            vec.push(item);
            println!("branch 2 => vec so far contains: {vec:?}");
        }

        vec
    }

    #[tokio::test]
    async fn test_unsafe_cancel_stream() {
        let mut stream = gen_input_stream();
        let sleep_time = 300;
        let sleep = tokio::time::sleep(std::time::Duration::from_millis(sleep_time));
        tokio::pin!(sleep);

        loop {
            tokio::select! {
                // Branch 1 - Timeout.
                _ = &mut sleep => {
                    println!("branch 1 - time is up - end");
                    break;
                }
                // Branch 2 - Read from stream.
                it = read_3_items_not_cancel_safe(&mut stream) => {
                    println!("branch 2 - got 3 items: {it:?}");
                }
            }
        }

        println!("loop exited");

        // Only [1, 2] is consumed by Branch 2 before the timeout happens in Branch 1.
        let it = stream.next().await.unwrap().unwrap();
        assert_eq!(it, 3);
    }
}
