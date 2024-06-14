#![no_std]

use test_log::test;
use tracing::debug;

#[test]
fn test_without_std() {
  debug!("It works without std!");
  assert_eq!(1 + 1, 2);
}
