//! Use this file for iterating on the derive code. You can view the
//! expanded code for any given configuration by updating this file and
//! running:
//!
//! ```sh
//! cargo expand --test=prototype
//! ```


#[test_log::test]
fn it_works() {
  tracing::debug!("tracing::DEBUG");
  tracing::info!("tracing::INFO");
  tracing::warn!("tracing::WARN");
  tracing::error!("tracing::ERROR");

  logging::debug!("log::DEBUG");
  logging::info!("log::INFO");
  logging::warn!("log::WARN");
  logging::error!("log::ERROR");

  assert_eq!(2 + 2, 4);
}
