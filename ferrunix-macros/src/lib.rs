//! Proc-macro crate for [`ferrunix`].
//!
//! [`ferrunix`]: https://crates.io/crates/ferrunix
#![allow(
    clippy::panic,
    clippy::min_ident_chars,
    clippy::module_name_repetitions,
    clippy::missing_docs_in_private_items
)]

use darling::FromDeriveInput;
use syn::{parse_macro_input, DeriveInput};

use self::attr::DeriveAttrInput;
use self::inject::derive_macro_impl;

mod attr;
mod inject;
mod utils;

/// `#[derive(Inject)]` proc-macro implementation.
///
/// # Panics
/// If this panics, it's a bug!
#[proc_macro_derive(Inject, attributes(provides, inject))]
pub fn derive_inject(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    // eprintln!("input: {input:#?}");

    let attr_input =
        DeriveAttrInput::from_derive_input(&input).map_err(syn::Error::from);
    if let Err(err) = attr_input {
        return err.into_compile_error().into();
    }
    let attr_input = attr_input.expect("error is returned above");
    // eprintln!("attr_input: {attr_input:#?}");

    derive_macro_impl(&input, &attr_input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
