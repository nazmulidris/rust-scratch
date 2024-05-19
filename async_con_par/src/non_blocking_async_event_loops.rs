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

/// More info: <https://docs.rs/tokio/latest/tokio/attr.test.html>
#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_main_loop() {
    // Register tracing subscriber.
    tracing_subscriber::fmt()
        .without_time()
        .compact()
        .with_target(false)
        .with_line_number(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    // Create channels for events and shutdown signals.
    let event_channel = tokio::sync::mpsc::channel::<String>(1_000);
    let (event_sender, mut event_receiver) = event_channel;

    let shutdown_channel = tokio::sync::broadcast::channel::<()>(1_000);
    let (shutdown_sender, _) = shutdown_channel;

    // Spawn the main event loop.
    let mut shutdown_receiver = shutdown_sender.subscribe();
    let safe_count: std::sync::Arc<std::sync::Mutex<usize>> = Default::default();
    let safe_count_clone = safe_count.clone();
    let join_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                event = event_receiver.recv() => {
                    tracing::info!(?event, "task got event: event");
                    let mut count = safe_count_clone.lock().unwrap();
                    *count += 1;
                }
                _ = shutdown_receiver.recv() => {
                    tracing::info!("task got shutdown signal");
                    break;
                }
            }
        }
    });

    // Send events, in parallel.
    let mut handles = vec![];
    for i in 0..10 {
        let event_sender_clone = event_sender.clone();
        let join_handle = tokio::spawn(async move {
            tracing::info!(i, "sending event");
            let event = format!("event {}", i);
            let _ = event_sender_clone.send(event).await;
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        });
        handles.push(join_handle);
    }

    // Wait for all events to be sent using tokio.
    futures::future::join_all(handles).await;

    // Shutdown the event loops.
    shutdown_sender.send(()).unwrap();

    // Wait for the event loop to shutdown.
    join_handle.await.unwrap();

    // Assertions.
    assert_eq!(shutdown_sender.receiver_count(), 1);
    assert_eq!(*safe_count.lock().unwrap(), 10);
}
