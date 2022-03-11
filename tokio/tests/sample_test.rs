use tokio_example_lib::my_middleware::{adder_mw, logger_mw, Action};

// Integration tests
// Functions that are in scope: <https://stackoverflow.com/a/45151641/2085356>
// About integration tests: <https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory>
// Tokio test macro: <https://docs.rs/tokio/latest/tokio/attr.test.html>

#[tokio::test]
async fn test_logger_mw_works() {
  let result = logger_mw().spawn(Action::Add(1, 2)).await.unwrap();
  assert!(result.is_none());
}

#[tokio::test]
async fn test_adder_mw_works() {
  let result = adder_mw().spawn(Action::Add(1, 2)).await.unwrap();
  assert_eq!(result, Some(Action::Result(3)));
}
