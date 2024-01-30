# Simple example kv.rs
- This is the primary example. This is a solid crate, easy to use and works w/ multiple
  processes.
- Uses kv crate. Key value store that uses `sled` as the backend. Written by 1 person.

# Advanced example rkv.rs
- This is the deprecated example. This crate does not work well with multiple processes.
- Uses rkv crate. Key value store that is written by Mozilla. Written by a big team.
- Has support for many different backends: LMDB, and SafeModeDatabase (we are using this one).
