// Copyright (C) 2026 Jason Orendorff <jason.orendorff@gmail.com>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! Tests for accurate source spans.

use std::ops::Range;
use std::str::FromStr as _;

use proc_macro2::TokenStream;
use syn::ItemFn;
use syn::Meta;

/// Strip the `#[test_log::test(...)]` attribute off `input`, returning
/// its argument tokens.
fn take_test_log_args(input: &mut ItemFn) -> TokenStream {
  let pos = input
    .attrs
    .iter()
    .position(|a| {
      let segs = &a.path().segments;
      segs.len() == 2 && segs[0].ident == "test_log" && segs[1].ident == "test"
    })
    .expect("input must carry a #[test_log::test(...)] attribute");
  let attr = input.attrs.remove(pos);
  match attr.meta {
    Meta::List(list) => list.tokens,
    _ => TokenStream::new(),
  }
}

/// Parse `src` (which must contain a single annotated function) and
/// run it through `try_test`, returning `(input_block_byte_range,
/// expanded_block_byte_range)`.
fn expand_and_get_brace_ranges(src: &str) -> (Range<usize>, Range<usize>) {
  let tokens = TokenStream::from_str(src).expect("parse source");
  let mut input = syn::parse2::<ItemFn>(tokens).expect("parse fn");
  let args = take_test_log_args(&mut input);
  let input_range = input.block.brace_token.span.join().byte_range();

  let expanded = test_log_core::try_test(args, input).expect("expansion succeeds");
  let expanded_fn = syn::parse2::<ItemFn>(expanded).expect("re-parse expanded fn");
  let expanded_range = expanded_fn.block.brace_token.span.join().byte_range();

  (input_range, expanded_range)
}

/// The outer block of the expanded function body must keep the exact
/// source byte range of the user-written body -- i.e. expansion must
/// *not* wrap it in a fresh, call-site-spanned block.
#[test]
fn preserves_body_span() {
  let src = "\
#[test_log::test]
fn it_works() {
    let _x = 1;
}
";
  let (input, expanded) = expand_and_get_brace_ranges(src);
  assert_eq!(input, expanded, "body braces must retain their source span");
}

/// Same invariant when forwarding to an inner test attribute such as
/// `tokio::test`.
#[test]
fn preserves_body_span_with_inner_attribute() {
  let src = "\
#[test_log::test(tokio::test)]
async fn it_works() {
    let _x = 1;
}
";
  let (input, expanded) = expand_and_get_brace_ranges(src);
  assert_eq!(input, expanded, "body braces must retain their source span");
}

/// Same invariant for functions with an explicit return type.
#[test]
fn preserves_body_span_with_return_type() {
  let src = "\
#[test_log::test]
fn returns_result() -> Result<(), String> {
    Ok(())
}
";
  let (input, expanded) = expand_and_get_brace_ranges(src);
  assert_eq!(input, expanded, "body braces must retain their source span");
}
