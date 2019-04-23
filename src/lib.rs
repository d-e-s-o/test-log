// Copyright (C) 2019 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

use syn::ItemFn;
use syn::parse_macro_input;
use syn::ReturnType;


/// A procedural macro for the `test` attribute.
///
/// The attribute can be used to define a test that has the `env_logger`
/// initialized.
///
/// # Example
///
/// Test functionality on an arbitrary Nitrokey device (i.e., Pro or
/// Storage):
/// ```rust
/// # use log::info;
/// # // Note that no test would actually run, regardless of `no_run`,
/// # // because we do not invoke the function.
/// #[test_env_log::test]
/// fn it_works() {
///   info!("Checking whether it still works...");
///   assert_eq!(2 + 2, 4);
///   info!("Looks good!");
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
  // Bail out if user tried to pass additional arguments. E.g.,
  // #[test_env_log::test(foo = "bar")
  if !attr.is_empty() {
    panic!("unsupported attributes supplied: {}", attr);
  }
  // Make clippy happy.
  drop(attr);

  let input = parse_macro_input!(item as ItemFn);
  expand_wrapper(&input)
}


/// Emit code for a wrapper function around a test function.
fn expand_wrapper(wrappee: &ItemFn) -> TokenStream {
  let attrs = &wrappee.attrs;
  let decl = &wrappee.decl;
  let body = &wrappee.block;
  let test_name = &wrappee.ident;

  let ret_type = match &decl.output {
    ReturnType::Default => quote! {()},
    ReturnType::Type(_, type_) => quote! {#type_},
  };

  let result = quote! {
    #[test]
    #(#attrs)*
    fn #test_name() -> #ret_type {
      fn #test_name() -> #ret_type {
        #body
      }

      let _ = ::env_logger::builder().is_test(true).try_init();
      #test_name()
    }
  };
  result.into()
}
