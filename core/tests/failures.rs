// Copyright (C) 2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! Tests for error paths in `#[test_log(...)]` attribute parsing.

use proc_macro2::TokenStream;

use syn::parse_quote;
use syn::ItemFn;
use syn::Meta;


/// Try to expand a function that has the given `#[test_log(...)]`
/// attribute, returning the error produced by `try_test`.
fn expand_err(mut input: ItemFn) -> String {
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

  test_log_core::try_test(attr_args, input)
    .unwrap_err()
    .to_string()
}


/// Verify that non-NameValue syntax like `#[test_log(debug)]` is rejected.
#[test]
fn reject_non_name_value() {
  let err = expand_err(parse_quote! {
    #[test_log::test]
    #[test_log(debug)]
    fn bad() {}
  });
  assert!(
    err.contains("Expected NameValue syntax"),
    "unexpected error: {err}",
  );
}

/// Test that multi-segment paths like `#[test_log(foo::bar = "x")]` are rejected.
#[test]
fn reject_multi_segment_path() {
  let err = expand_err(parse_quote! {
    #[test_log::test]
    #[test_log(foo::bar = "debug")]
    fn bad() {}
  });
  assert!(
    err.contains("Expected NameValue syntax"),
    "unexpected error: {err}",
  );
}

/// Check that unrecognized attribute names are rejected.
#[test]
fn reject_unrecognized_attr() {
  let err = expand_err(parse_quote! {
    #[test_log::test]
    #[test_log(bogus = "x")]
    fn bad() {}
  });
  assert!(
    err.contains("Unrecognized attribute"),
    "unexpected error: {err}",
  );
}

/// Verify that non-string values like `#[test_log(default_log_filter = 10)]`
/// are rejected.
#[test]
fn reject_non_string_value() {
  let err = expand_err(parse_quote! {
    #[test_log::test]
    #[test_log(default_log_filter = 10)]
    fn bad() {}
  });
  assert!(
    err.contains("Failed to parse value, expected a string"),
    "unexpected error: {err}",
  );
}
