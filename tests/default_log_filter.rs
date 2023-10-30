//! This test needs to be defined in a separate file because it depends on global state
//! (logger) and can't be run in parallel / in the same process with other tests to avoid
//! flakiness (other tests initializing the env_logger first).

#[test_log::test(tokio::test)]
#[test_log(default_log_filter = "debug")]
async fn with_inner_test_attribute_and_default_log_filter_defined() {
  // Check that RUST_LOG isn't set, because that could affect the outcome of this
  // test since we're checking that we fallback to "debug" if no RUST_LOG is set.
  assert!(std::env::var(env_logger::DEFAULT_FILTER_ENV).is_err());
  assert!(logging::log_enabled!(logging::Level::Debug));
}
