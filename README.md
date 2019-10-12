[![pipeline](https://gitlab.com/d-e-s-o/test-env-log/badges/master/pipeline.svg)](https://gitlab.com/d-e-s-o/test-env-log/commits/master)
[![crates.io](https://img.shields.io/crates/v/test-env-log.svg)](https://crates.io/crates/test-env-log)
[![Docs](https://docs.rs/test-env-log/badge.svg)](https://docs.rs/test-env-log)
[![rustc](https://img.shields.io/badge/rustc-1.32+-blue.svg)](https://blog.rust-lang.org/2019/01/17/Rust-1.32.0.html)

test-env-log
============

- [Documentation][docs-rs]
- [Changelog](CHANGELOG.md)

**test-env-log** is a crate that takes care of automatically
initializing `env_logger` for Rust tests.

When running Rust tests it can often be helpful to have easy access to
the verbose log messages emitted by the code under test. Assuming said
code uses the [`log`](https://crates.io/crates/log) backend for its
logging purposes that may not be straight forward, however. The problem
is that all crates making use of `log` require some form of
initialization to be usable.

The commonly used [`env_logger`](https://crates.io/crates/env_logger),
for example, needs to be initialized like this:
```rust
let _ = env_logger::builder().is_test(true).try_init();
```
in **each and every** test.

This crate takes care of this per-test initialization in an intuitive
way.


Usage
-----

The crate provides a custom `#[test]` attribute that, when used for
running a particular test, takes care of initializing `env_logger`
beforehand.
As such, usage is as simple as importing and using said attribute:
```rust
use test_env_log::test;

#[test]
fn it_works() {
  info!("Checking whether it still works...");
  assert_eq!(2 + 2, 4);
  info!("Looks good!");
}
```

It is of course also possible to initialize logging for a chosen set of
tests, by only annotating these with the custom attribute:
```rust
#[test_env_log::test]
fn it_still_works() {
  // ...
}
```

[docs-rs]: https://docs.rs/crate/test-env-log
