// Copyright (C) 2019-2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use tokio::runtime::Builder;

use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::instrument;


mod something {
  pub type Error = String;
}

use something::Error;


#[test_env_log::test]
fn without_return_type() {
  assert_eq!(2 + 2, 4);
}

#[test_env_log::test]
fn with_return_type() -> Result<(), Error> {
  Ok(())
}

#[test_env_log::test]
#[should_panic(expected = "success")]
fn with_panic() {
  panic!("success")
}

#[test_env_log::test(tokio::test)]
async fn with_inner_test_attribute_and_async() {
  assert_eq!(async { 42 }.await, 42)
}

#[instrument]
async fn instrumented(input: usize) -> usize {
  info!("input = {}", input);
  if input == 0 || input == 4 {
    error!("here we go");
  }
  let result = input + 1;
  info!("result = {}", result);
  result
}

#[test_env_log::test]
fn trace_with_custom_runtime() {
  let rt = Builder::new_current_thread().build().unwrap();

  rt.block_on(async {
    instrumented(0).await;
    instrumented(1).await;
    debug!("done");
  })
}

#[test_env_log::test(tokio::test)]
async fn trace_with_tokio_attribute() {
  instrumented(6).await;
  instrumented(4).await;
  debug!("done");
}


#[cfg(feature = "trace")]
#[test_env_log::test]
fn test_tracing_error() {
  use tracing::{span, Level};
  use tracing_error::SpanTrace;

  let captured_inner_span;
  let captured_outer_span;

  {
    let outer_span = span!(Level::ERROR, "outer");
    let _guard = outer_span.enter();
    {
      let inner_span = span!(Level::ERROR, "inner");
      let _guard = inner_span.enter();
        captured_inner_span = SpanTrace::capture();
    }
    captured_outer_span = SpanTrace::capture();
  }
  assert!(format!("{}", captured_inner_span).contains("inner"));
  assert!(format!("{}", captured_inner_span).contains("outer"));

  assert!( ! format!("{}", captured_outer_span).contains("inner"));
  assert!(format!("{}", captured_outer_span).contains("outer"));
}


mod local {
  use super::Error;

  use test_env_log::test;

  #[test]
  fn without_return_type() {
    assert_eq!(2 + 2, 4);
  }

  #[test]
  fn with_return_type() -> Result<(), Error> {
    Ok(())
  }

  #[test]
  #[should_panic(expected = "success")]
  fn with_panic() {
    panic!("success")
  }

  #[test(tokio::test)]
  async fn with_inner_test_attribute_and_async() {
    assert_eq!(async { 42 }.await, 42)
  }
}
