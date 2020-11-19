// Copyright (C) 2019-2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream as Tokens;

use quote::quote;

use syn::AttributeArgs;
use syn::ItemFn;
use syn::Meta;
use syn::NestedMeta;
use syn::parse_macro_input;
use syn::parse_quote;
use syn::Path;
use syn::ReturnType;


/// A procedural macro for the `test` attribute.
///
/// The attribute can be used to define a test that has the `env_logger`
/// initialized.
///
/// # Example
///
/// Specify the attribute on a per-test basis:
/// ```rust
/// # // doctests seemingly run in a slightly different environment where
/// # // `super`, which is what our macro makes use of, is not available.
/// # // By having a fake module here we work around that problem.
/// # #[cfg(feature = "log")]
/// # mod fordoctest {
/// # use logging::info;
/// # // Note that no test would actually run, regardless of `no_run`,
/// # // because we do not invoke the function.
/// #[test_env_log::test]
/// fn it_works() {
///   info!("Checking whether it still works...");
///   assert_eq!(2 + 2, 4);
///   info!("Looks good!");
/// }
/// # }
/// ```
///
/// It can be very convenient to convert over all tests by overriding
/// the `#[test]` attribute on a per-module basis:
/// ```rust,no_run
/// # mod fordoctest {
/// use test_env_log::test;
///
/// #[test]
/// fn it_still_works() {
///   // ...
/// }
/// # }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
  let args = parse_macro_input!(attr as AttributeArgs);
  let input = parse_macro_input!(item as ItemFn);

  let inner_test = match args.as_slice() {
    [] => parse_quote! { test },
    [NestedMeta::Meta(Meta::Path(path))] => path.clone(),
    _ => panic!("unsupported attributes supplied: {}", quote! { args }),
  };

  expand_wrapper(&inner_test, &input)
}


/// Expand the initialization code for the `log` crate.
fn expand_logging_init() -> Tokens {
  #[cfg(feature = "log")]
  quote! {
    {
      let _ = ::env_logger::builder().is_test(true).try_init();
    }
  }
  #[cfg(not(feature = "log"))]
  quote! {}
}


/// Expand the initialization code for the `tracing` crate.
fn expand_tracing_init() -> Tokens {
  #[cfg(feature = "trace")]
  quote! {
    let _guard = {
      use ::tracing_subscriber::layer::SubscriberExt;
      let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
        .finish()
        .with(::tracing_error::ErrorLayer::default());
      ::tracing::subscriber::set_default(subscriber)
    };
  }
  #[cfg(not(feature = "trace"))]
  quote! {}
}


/// Emit code for a wrapper function around a test function.
// We have to jump through a lot of hoops here to allow for the case
// where the user imports test_env_log::test as test. In such a case
// #[test] -- which is what our macro emits -- would just be
// indefinitely expanded as the attribute we define in this very crate.
//
// To work around this problem we introduce a new module (named after
// the test, to allow users to still pattern match as expected on test
// names, as this module will now appear in the test's path), which
// ensures access to the built-in #[test] (it is unclear whether there
// exists syntax to reference it; if so, it is well hidden). With that
// we have a different problem: now the user's return type may not be
// defined, because it could reference data from the outer module. If
// we were to glob import super::* we would be back at square one with
// #[test] being our attribute.
// So instead, we introduce a type alias for the return type in the
// outer module. Now we can reference only this type def from the
// inner one. In your face.
fn expand_wrapper(inner_test: &Path, wrappee: &ItemFn) -> TokenStream {
  let attrs = &wrappee.attrs;
  let async_ = &wrappee.sig.asyncness;
  let await_ = if async_.is_some() {
    quote! {.await}
  } else {
    quote! {}
  };
  let body = &wrappee.block;
  let test_name = &wrappee.sig.ident;

  // The type alias we use for the return type. Note that we cannot
  // simply use #test_name as a type of that name would clash with a
  // module of the same name.
  //
  // Note that we need to rely on proc_macro2 here, because while the
  // compiler provided proc_macro has `Ident` and `Span` types, they
  // cannot be interpolated with quote!{} for lack of quote::ToTokens
  // implementations.
  let alias = format!("__alias_{}", test_name);
  let alias_name = Ident::new(&alias, Span::call_site());

  // Rust does not allow us to have a test function with #[should_panic]
  // that returns anything but (). Unfortunately, it does not check
  // whether a type alias actually just "expands" to (), but errors out.
  // So we need to special case that by referencing () directly.
  let (alias_ref, ret_type) = match &wrappee.sig.output {
    ReturnType::Default => (
      quote! {()},
      quote! {()},
    ),
    ReturnType::Type(_, type_) => (
      quote! {super::#alias_name},
      quote! {#type_},
    ),
  };

  let logging_init = expand_logging_init();
  let tracing_init = expand_tracing_init();

  let result = quote! {
    #async_ fn #test_name() -> #ret_type {
      #body
    }

    #[allow(unused)]
    type #alias_name = #ret_type;

    mod #test_name {
      use super::#test_name;
      #[#inner_test]
      #(#attrs)*
      #async_ fn f() -> #alias_ref {
        #logging_init
        #tracing_init

        super::#test_name()#await_
      }
    }
  };
  result.into()
}
