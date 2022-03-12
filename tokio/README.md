# tokio

This is a quick little project to see how async and tokio function. The goal is simple, to mimic
just the middleware dispatcher part of an async redux store.

This is what a middleware function looks like.

```rust
dyn Fn(&A) -> Option<A>
fn (&A) -> Option<A>
```

Where `A` implements `Clone` and should be safe to send across thread boundaries. Any data that this
function might need has to be thread safe as well.

The middleware function has to be `async`.
