use tokio_example_lib::my_middleware::{adder_mw, logger_mw, Action};

// Imports.
#[tokio::main]
async fn main() {
  {
    let mw_fun = logger_mw();
    mw_fun.spawn(Action::Add(1, 2)).await.unwrap();
    mw_fun.spawn(Action::Add(1, 2)).await.unwrap();
  }

  {
    let mw_fun = adder_mw();
    println!("{:?}", mw_fun.spawn(Action::Add(1, 2)).await.unwrap());
    println!("{:?}", mw_fun.spawn(Action::Add(1, 2)).await.unwrap());
  }
}
