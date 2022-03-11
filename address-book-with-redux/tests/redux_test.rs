// Integration tests
// Functions that are in scope: <https://stackoverflow.com/a/45151641/2085356>
// About integration tests: <https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory>
// Tokio test macro: <https://docs.rs/tokio/latest/tokio/attr.test.html>

#[tokio::test]
async fn test_logger_mw_works() {
  assert!(true);
}

#[tokio::test]
async fn test_adder_mw_works() {
  assert!(true);
}
