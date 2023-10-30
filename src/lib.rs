// Copyright (C) 2019-2023 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![deny(missing_docs)]

//! A crate providing a replacement #[[macro@test]] attribute that
//! initializes logging and/or tracing infrastructure before running
//! tests.

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;

use quote::quote;

use syn::{parse::Parse, parse_macro_input, Attribute, Expr, ItemFn, Lit, Meta};

/// A procedural macro for the `test` attribute.
///
/// The attribute can be used to define a test that has the `env_logger`
/// and/or `tracing` crates initialized (depending on the features used).
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
/// #[test_log::test]
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
/// use test_log::test;
///
/// #[test]
/// fn it_still_works() {
///   // ...
/// }
/// # }
/// ```
///
/// You can also wrap another attribute. For example, suppose you use
/// [`#[tokio::test]`](https://docs.rs/tokio/1.4.0/tokio/attr.test.html)
/// to run async tests:
/// ```
/// # mod fordoctest {
/// use test_log::test;
///
/// #[test(tokio::test)]
/// async fn it_still_works() {
///   // ...
/// }
/// # }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as ItemFn);
  try_test(attr, item)
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

fn try_test(attr: TokenStream, input: ItemFn) -> syn::Result<Tokens> {
  let inner_test = if attr.is_empty() {
    quote! { ::core::prelude::v1::test }
  } else {
    attr.into()
  };
  let mut attribute_args = AttributeArgs::default();

  let ItemFn {
    attrs,
    vis,
    sig,
    block,
  } = input;

  let mut non_test_log_attrs = vec![];
  for attr in attrs {
    let matched = attribute_args.try_parse_attr_single(&attr)?;
    // Keep only attrs that didn't match the #[test_log(_)] syntax.
    if !matched {
      non_test_log_attrs.push(attr);
    }
  }

  let logging_init = expand_logging_init(&attribute_args);
  let tracing_init = expand_tracing_init(&attribute_args);

  let result = quote! {
    #[#inner_test]
    #(#non_test_log_attrs)*
    #vis #sig {
      // We put all initialization code into a separate module here in
      // order to prevent potential ambiguities that could result in
      // compilation errors. E.g., client code could use traits that
      // could have methods that interfere with ones we use as part of
      // initialization; with a `Foo` trait that is implemented for T
      // and that contains a `map` (or similarly common named) method
      // that could cause an ambiguity with `Iterator::map`, for
      // example.
      // The alternative would be to use fully qualified call syntax in
      // all initialization code, but that's much harder to control.
      mod init {
        pub fn init() {
          #logging_init
          #tracing_init
        }
      }

      init::init();

      #block
    }
  };
  Ok(result)
}

#[derive(Debug, Default)]
struct AttributeArgs {
  default_log_filter: Option<String>,
}

impl AttributeArgs {
  fn try_parse_attr_single(&mut self, attr: &Attribute) -> syn::Result<bool> {
    if !attr.path().is_ident("test_log") {
      return Ok(false);
    }

    let nested_meta = attr.parse_args_with(Meta::parse)?;
    let Meta::NameValue(name_value) = nested_meta else {
      return Err(syn::Error::new_spanned(
        &nested_meta,
        "Expected NameValue syntax, e.g. 'default_log_filter = \"debug\"'.",
      ));
    };

    let Some(ident) = name_value.path.get_ident() else {
      return Err(syn::Error::new_spanned(
        &name_value.path,
        "Expected NameValue syntax, e.g. 'default_log_filter = \"debug\"'.",
      ));
    };

    let arg_ref = if ident == "default_log_filter" {
      &mut self.default_log_filter
    } else {
      return Err(syn::Error::new_spanned(
        &name_value.path,
        "Unrecognized attribute, see documentation for details.",
      ));
    };

    if let Expr::Lit(lit) = &name_value.value {
      if let Lit::Str(lit_str) = &lit.lit {
        *arg_ref = Some(lit_str.value());
      }
    }

    // If we couldn't parse the value on the right-hand side because it was some
    // unexpected type, e.g. #[test_log::log(default_log_filter=10)], return an error.
    if arg_ref.is_none() {
      return Err(syn::Error::new_spanned(
        &name_value.value,
        "Failed to parse value, expected a string!",
      ));
    }

    Ok(true)
  }
}

/// Expand the initialization code for the `log` crate.
fn expand_logging_init(attribute_args: &AttributeArgs) -> Tokens {
  let add_default_log_filter = if let Some(default_log_filter) = &attribute_args.default_log_filter
  {
    quote! {
      let env_logger_builder = env_logger_builder
        .parse_env(env_logger::Env::default().default_filter_or(#default_log_filter));
    }
  } else {
    quote! {}
  };
  #[cfg(feature = "log")]
  quote! {
    {
      let mut env_logger_builder = ::env_logger::builder();
      #add_default_log_filter
      let _ = env_logger_builder.is_test(true).try_init();
    }
  }
  #[cfg(not(feature = "log"))]
  quote! {}
}

/// Expand the initialization code for the `tracing` crate.
#[cfg(not(feature = "trace"))]
fn expand_tracing_init(_attribute_args: &AttributeArgs) -> Tokens {
  quote! {}
}

#[cfg(feature = "trace")]
fn expand_tracing_init(attribute_args: &AttributeArgs) -> Tokens {
  let env_filter = if let Some(default_log_filter) = &attribute_args.default_log_filter {
    quote! {
      ::tracing_subscriber::EnvFilter::builder()
        .with_default_directive(#default_log_filter.parse().expect("test-log: default_log_filter must be valid"))
        .from_env_lossy()
    }
  } else {
    quote! { ::tracing_subscriber::EnvFilter::from_default_env() }
  };

  quote! {
    {
      let __internal_event_filter = {
        use ::tracing_subscriber::fmt::format::FmtSpan;

        match ::std::env::var_os("RUST_LOG_SPAN_EVENTS") {
          Some(mut value) => {
            value.make_ascii_lowercase();
            let value = value.to_str().expect("test-log: RUST_LOG_SPAN_EVENTS must be valid UTF-8");
            value
              .split(",")
              .map(|filter| match filter.trim() {
                "new" => FmtSpan::NEW,
                "enter" => FmtSpan::ENTER,
                "exit" => FmtSpan::EXIT,
                "close" => FmtSpan::CLOSE,
                "active" => FmtSpan::ACTIVE,
                "full" => FmtSpan::FULL,
                _ => panic!("test-log: RUST_LOG_SPAN_EVENTS must contain filters separated by `,`.\n\t\
                  For example: `active` or `new,close`\n\t\
                  Supported filters: new, enter, exit, close, active, full\n\t\
                  Got: {}", value),
              })
              .fold(FmtSpan::NONE, |acc, filter| filter | acc)
          },
          None => FmtSpan::NONE,
        }
      };

      let _ = ::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(#env_filter)
        .with_span_events(__internal_event_filter)
        .with_test_writer()
        .try_init();
    }
  }
}
