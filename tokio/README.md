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

# Did not work # 1

```rust
/// Middleware that returns `None` and simply logs the action.
async fn mw_1(
  action: Action,
  results: Vec<Action>,
) {
  println!("{:?}", action);
}

/// Middleware that does something w/ the action, and returns another action.
async fn mw_2(
  action: Action,
  results: Vec<Action>,
) {
  println!("{:?}", action);
}

// type MwFn<R>
// where
//   R: Future<Output = Action>,
// = fn(Action, Vec<Action>) -> R;
type MwFn<R> = fn(Action, Vec<Action>) -> R;
```

# Did not work # 2

```rust
trait Middleware {
  type Output;
  fn run(
    self,
    action: &Action,
    // results: &mut Vec<Action>,
  ) -> Self::Output;
}

struct MyMw1 {
  name: String,
}
impl Middleware for MyMw1 {
  type Output = ();
  fn run(
    self,
    action: &Action,
    // results: &mut Vec<Action>,
  ) -> Self::Output {
    // results.push(action.clone());
    println!("{:?}", action);
  }
}

struct MyMw2 {
  name: String,
}
impl Middleware for MyMw2 {
  type Output = ();
  fn run(
    self,
    action: &Action,
    // results: &mut Vec<Action>,
  ) -> Self::Output {
    println!("{:?}", action);
  }
}
```