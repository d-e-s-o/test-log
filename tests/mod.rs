// Copyright (C) 2019 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

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
}
