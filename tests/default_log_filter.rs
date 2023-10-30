//! This test needs to be defined in a separate file because it depends
//! on global state (logger) and can't be run in parallel/the same
//! process with other tests to avoid flakiness (other tests
//! initializing `env_logger` first).

use std::env;

use logging::log_enabled;
use logging::Level;


#[ignore = "interferes with RUST_LOG; disabled by default"]
#[test_log::test(tokio::test)]
#[test_log(default_log_filter = "debug")]
async fn with_inner_test_attribute_and_default_log_filter_defined() {
  // Check that RUST_LOG isn't set, because that could affect the
  // outcome of this test since we're checking that we fallback to
  // "debug" if no RUST_LOG is set.
  assert!(env::var(env_logger::DEFAULT_FILTER_ENV).is_err());
  assert!(log_enabled!(Level::Debug));
}
