// Copyright (c) 2026 Nazmul Idris. Licensed under Apache License, Version 2.0.

// cspell:words Condvar Wrkr

#![allow(unused_imports, dead_code, unused_variables, clippy::wildcard_imports)]

use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

/// # Explanation of Rust [`Condvar`]
///
/// A condition variable (often abbreviated as *condvar*) is a synchronization primitive
/// used in multithreaded programming. It allows one or more threads to **block** until a
/// specific condition becomes true, at which point another thread can wake them up.
///
/// Condition variables solve the problem of **busy waiting**. Busy waiting occurs
/// when a thread constantly polls a condition in a tight loop to check if a shared
/// resource is ready. While synchronization primitives like *[`spinlocks`]* intentionally
/// use busy waiting for extremely short delays to avoid the overhead of context
/// switching, using this polling approach for unpredictable or long-term waits wastes
/// valuable CPU cycles. Instead of burning CPU time repeatedly asking "Are we there
/// yet?", a thread can use a condition variable to efficiently block its execution
/// until it receives a notification.
///
/// ## Comparison with Java: `synchronized`, `wait()`, and `notify()`
///
/// In traditional Java, every object has a built-in mutex and a single condition
/// variable. You access this hidden ["monitor"] using the `synchronized` keyword
/// alongside the `wait()` and `notify()` methods. If you forget to hold the lock before
/// calling `wait()`, Java catches the mistake at runtime and throws an
/// `IllegalMonitorStateException`.
///
/// Rust takes a more explicit approach by separating these concepts into distinct
/// [`Mutex<T>`] and [`Condvar`] types. This explicit separation closely mirrors modern
/// Java's `ReentrantLock` and `Condition` classes.
///
/// However, Rust goes a step further to protect against concurrency bugs. In Rust, a
/// [`Mutex`] physically owns the data it protects. When you lock the mutex, you receive
/// a [`MutexGuard`] token. The [`Condvar::wait()`] method requires you to pass in this
/// exact token. By leveraging the ownership system, the compiler forces you to hold the
/// lock before waiting—transforming Java's runtime crashes into strict compile-time
/// guarantees.
///
/// [`spinlocks`]: https://en.wikipedia.org/wiki/Spinlock
/// ["monitor"]: https://en.wikipedia.org/wiki/Monitor_(synchronization)
/// [`Condvar`]: std::sync::Condvar
/// [`Mutex<T>`]: std::sync::Mutex
/// [`MutexGuard`]: std::sync::MutexGuard
/// [`Mutex`]: std::sync::Mutex
/// [`Condvar::wait()`]: std::sync::Condvar::wait
struct StateMonitor<T> {
    pub value: Mutex<T>,
    pub condvar: Condvar,
}
type SafeStateMonitor<T> = Arc<StateMonitor<T>>;
type Number = usize;

fn main() {
    let state_monitor = StateMonitor {
        value: Mutex::new(0),
        condvar: Condvar::new(),
    };
    let safe_state_monitor = Arc::new(state_monitor);

    // Spawn worker thread to do some work.
    let safe_state_monitor_move = safe_state_monitor.clone();
    let _unused_join_handle =
        thread::spawn(move || worker_thread_entry_point(safe_state_monitor_move));

    // Main thread should not busy wait / spinlock for this work to complete.
    main_thread_entry_point(safe_state_monitor);
}

use console::*;

fn worker_thread_entry_point(safe_state_monitor: SafeStateMonitor<Number>) {
    log(ThreadLabel::Worker, "Starting...");

    let StateMonitor { value, condvar } = &*safe_state_monitor;

    // Simulate some long running operations.
    thread::sleep(Duration::from_secs(2));

    // Update value in state.
    {
        let mut value_guard = value.lock().unwrap();
        *value_guard = value_guard.saturating_add(10);
        log(
            ThreadLabel::Worker,
            &format!("Work is complete -> State updated to {value_guard}"),
        );
    }

    // Notify other threads that state is updated.
    condvar.notify_one();

    log(ThreadLabel::Worker, "Signal other thread to unblock");
}

/// 0 means condition not met -> work is not done.
fn is_not_complete(value: &mut Number) -> bool {
    *value == 0
}

fn main_thread_entry_point(safe_state_monitor: SafeStateMonitor<Number>) {
    let StateMonitor { value, condvar } = &*safe_state_monitor;

    log(
        ThreadLabel::Main,
        "Blocked for work to complete in worker thread...",
    );

    let /* NOT mut */ value_guard = value.lock().unwrap();

    // Block the main thread while work is not completed by worker thread.
    let new_value_guard = condvar
        .wait_while(
            // This consumes the guard & drops the lock, so worker thread can put stuff in
            // the value.
            value_guard,
            is_not_complete,
        )
        .unwrap();

    log(
        ThreadLabel::Main,
        &format!("Unblocked -> Work is complete {new_value_guard}"),
    );
}

mod console {
    use r3bl_tui::{fg_frozen_blue, fg_lizard_green};

    use super::*;

    pub enum ThreadLabel {
        Main,
        Worker,
    }

    pub fn log(thread_label: ThreadLabel, msg: &str) {
        let prefix = match thread_label {
            ThreadLabel::Main => fg_lizard_green("Main"),
            ThreadLabel::Worker => fg_frozen_blue("Wrkr"),
        };
        println!("{prefix}: {msg}");
    }
}
