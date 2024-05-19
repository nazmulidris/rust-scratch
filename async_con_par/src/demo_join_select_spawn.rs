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

use std::time::Duration;
use tokio::time::sleep;

pub async fn task_1(time: u64) {
    sleep(Duration::from_millis(time)).await;
    println!("task_1");
}

pub async fn task_2(time: u64) {
    sleep(Duration::from_millis(time)).await;
    println!("task_2");
}

pub async fn task_3(time: u64) {
    sleep(Duration::from_millis(time)).await;
    println!("task_3");
}

#[tokio::test]
async fn test_join() {
    tokio::join!(task_1(100), task_2(200), task_3(300));
    println!("all tasks done");
}

#[tokio::test]
async fn test_select() {
    tokio::select! {
        _ = task_1(100) => println!("task_1 done"),
        _ = task_2(200) => println!("task_2 done"),
        _ = task_3(300) => println!("task_3 done"),
    }
    println!("one task done");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_spawn() {
    let handle_1 = tokio::spawn(task_1(100));
    let handle_2 = tokio::spawn(task_2(100));
    let handle_3 = tokio::spawn(task_3(100));

    handle_1.await.unwrap();
    handle_2.await.unwrap();
    handle_3.await.unwrap();
    println!("all tasks done");
}
