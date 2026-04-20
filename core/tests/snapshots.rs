// Copyright (C) 2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! Snapshot tests for the macro expansion logic.

use proc_macro2::TokenStream;

use syn::parse2;
use syn::parse_quote;
use syn::ItemFn;
use syn::Meta;


#[cfg(not(any(feature = "log", feature = "trace")))]
const BACKEND: &str = "none";
#[cfg(all(feature = "log", not(feature = "trace")))]
const BACKEND: &str = "log";
#[cfg(all(feature = "trace", not(feature = "log")))]
const BACKEND: &str = "trace";
#[cfg(all(feature = "log", feature = "trace"))]
const BACKEND: &str = "log_trace";


/// Accept a fully-defined test function annotated with
/// `#[test_log::test(...)]`, extract the attribute arguments, strip
/// that attribute from the function, and expand via `try_test`.
fn expand(mut input: ItemFn) -> String {
  let pos = input
    .attrs
    .iter()
    .position(|a| {
      let segs = &a.path().segments;
      segs.len() == 2 && segs[0].ident == "test_log" && segs[1].ident == "test"
    })
    .expect("input must carry a #[test_log::test(...)] attribute");

  let attr = input.attrs.remove(pos);
  let attr_args = match &attr.meta {
    Meta::List(list) => list.tokens.clone(),
    _ => TokenStream::new(),
  };

  let tokens = test_log_core::try_test(attr_args, input).unwrap();
  let file = parse2(tokens).unwrap();

  // Format in a nice way, but halve indentation to match the project's
  // 2-space convention.
  prettyplease::unparse(&file)
    .lines()
    .map(|line| {
      let indent = line.len() - line.trim_start_matches(' ').len();
      let new_indent = indent / 2;
      format!("{}{}", &" ".repeat(new_indent), &line[indent..])
    })
    .collect::<Vec<_>>()
    .join("\n")
}

macro_rules! assert_snapshot {
  ($output:expr) => {
    insta::with_settings!({snapshot_suffix => BACKEND}, {
      insta::assert_snapshot!($output);
    });
  };
}


/// Check expansion of a plain `#[test_log::test]` on a sync function.
#[test]
fn bare_test() {
  let output = expand(parse_quote! {
    #[test_log::test]
    fn it_works() {
      assert_eq!(2 + 2, 4);
    }
  });
  assert_snapshot!(output);
}

/// Verify that an inner test attribute is forwarded and no `#[test]`
/// is added.
#[test]
fn inner_tokio_test() {
  let output = expand(parse_quote! {
    #[test_log::test(tokio::test)]
    async fn with_async() {
      assert_eq!(async { 42 }.await, 42);
    }
  });
  assert_snapshot!(output);
}

/// Make sure that an existing `#[test]` attribute is not duplicated.
#[test]
fn stacked_test_attribute() {
  let output = expand(parse_quote! {
    #[test_log::test]
    #[test]
    fn already_has_test() {}
  });
  assert_snapshot!(output);
}

/// Verify that a fully qualified `#[::core::prelude::v1::test]` is
/// recognized.
#[test]
fn stacked_core_prelude_test() {
  let output = expand(parse_quote! {
    #[test_log::test]
    #[::core::prelude::v1::test]
    fn already_has_test() {}
  });
  assert_snapshot!(output);
}

/// Test that a return type is preserved through expansion.
#[test]
fn with_return_type() {
  let output = expand(parse_quote! {
    #[test_log::test]
    fn returns_result() -> Result<(), String> {
      Ok(())
    }
  });
  assert_snapshot!(output);
}

/// Verify that `default_log_filter` is propagated into the init code.
#[cfg(feature = "unstable")]
#[test]
fn default_log_filter() {
  let output = expand(parse_quote! {
    #[test_log::test]
    #[test_log(default_log_filter = "debug")]
    fn with_filter() {}
  });
  assert_snapshot!(output);
}
