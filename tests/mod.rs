// Copyright (C) 2019-2023 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![allow(clippy::eq_op)]

use tokio::runtime::Builder;

use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::instrument;


mod something {
  pub type Error = String;
}

use something::Error;


#[test_log::test]
fn without_return_type() {
  assert_eq!(2 + 2, 4);
}

#[test_log::test]
fn with_return_type() -> Result<(), Error> {
  Ok(())
}

#[test_log::test]
#[should_panic(expected = "success")]
fn with_panic() {
  panic!("success")
}

#[test_log::test(tokio::test)]
async fn with_inner_test_attribute_and_async() {
  assert_eq!(async { 42 }.await, 42)
}

#[test_log::test(test_case::test_case(-2, -4))]
fn with_inner_test_attribute_and_test_args(x: i8, y: i8) {
  assert_eq!(x, -2);
  assert_eq!(y, -4);
}

#[test_log::test(test_case::test_case(-2, -4; "my test name"))]
fn with_inner_test_attribute_and_test_args_and_name(x: i8, y: i8) {
  assert_eq!(x, -2);
  assert_eq!(y, -4);
}

#[should_panic]
#[test_log::test(test_case::test_case(-2, -4))]
fn with_inner_test_attribute_and_test_args_and_panic(x: i8, _y: i8) {
  assert_eq!(x, 0);
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

#[test_log::test]
fn trace_with_custom_runtime() {
  let rt = Builder::new_current_thread().build().unwrap();

  rt.block_on(async {
    instrumented(0).await;
    instrumented(1).await;
    debug!("done");
  })
}

#[test_log::test(tokio::test)]
async fn trace_with_tokio_attribute() {
  instrumented(6).await;
  instrumented(4).await;
  debug!("done");
}

#[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
async fn trace_with_tokio_attribute_with_arguments() {
  instrumented(6).await;
  instrumented(4).await;
  debug!("done");
}

// A trait containing `map` method that has the potential to cause
// ambiguities in generated initialization code and is here only to
// prevent accidental regressions. In the past we were susceptible to a
// compilation error because generated code was using Iterator::map (but
// not using fully qualified syntax).
trait Foo: Sized {
  fn map(self) {}
}

impl<T> Foo for T {}

/// Make sure that Foo::map does not interfere with generated
/// initialization code.
#[test_log::test]
fn unambiguous_map() {}


/// A module used for testing the `test` attribute after importing it
/// via `use` instead of using fuller qualified syntax.
mod local {
  use super::Error;

  use test_log::test;

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

  #[test(test_case::test_case(-2, -4))]
  fn with_inner_test_attribute_and_test_args(x: i8, y: i8) {
    assert_eq!(x, -2);
    assert_eq!(y, -4);
  }

  #[should_panic]
  #[test(test_case::test_case(-2, -4))]
  fn with_inner_test_attribute_and_test_args_and_panic(x: i8, _y: i8) {
    assert_eq!(x, 0);
  }
}
