// Copyright (C) 2019-2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! Procedural macro powering `test-log`.

use proc_macro::TokenStream;

use syn::parse_macro_input;
use syn::Error;
use syn::ItemFn;

use test_log_core::try_test;


// Documented in `test-log` crate's re-export.
#[allow(missing_docs)]
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as ItemFn);
  try_test(attr.into(), item)
    .unwrap_or_else(Error::into_compile_error)
    .into()
}
