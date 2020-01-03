// Copyright (C) 2019-2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

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
