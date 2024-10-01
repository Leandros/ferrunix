use std::borrow::Cow;

use darling::util::Override;
use darling::{util, FromDeriveInput, FromField, FromMeta};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromField)]
#[darling(attributes(inject), forward_attrs(allow, doc, cfg))]
struct DeriveField {
    // Magic types:
    ident: Option<syn::Ident>,
    ty: syn::Type,
    // attrs: Vec<syn::Attribute>,

    // Custom:
    #[darling(default)]
    default: bool,
    ctor: Option<String>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inject))]
struct DeriveAttrInput {
    // Magic types:
    ident: syn::Ident,
    // generics: syn::Generics,
    data: darling::ast::Data<util::Ignored, DeriveField>,
    // attrs: Vec<syn::Attribute>,

    // Custom:
    provides: Option<String>,
}

impl DeriveAttrInput {
    // fn opts(&self) -> Cow<'_, Options> {
    //     match &self.opts {
    //         Override::Explicit(value) => Cow::Borrowed(value),
    //         Override::Inherit => Cow::Owned(Options {
    //             ty: self.ident.to_string(),
    //         }),
    //     }
    // }
}

#[derive(Debug, Clone, FromMeta)]
struct Options {
    ty: String,
}

#[proc_macro_derive(Inject, attributes(provides, inject))]
pub fn derive_inject(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    eprintln!("input: {input:#?}");

    // let attr_input = DeriveAttrInput::from_derive_input(&input).unwrap();
    // eprintln!("attr_input: {attr_input:#?}");

    let expanded = quote! {};

    proc_macro::TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attrs() {
        let input = r#"
#[derive(Inject)]
#[provides(dyn FooTrait)]
pub struct Foo {
    #[inject(default)]
    bar: u8,
    #[inject(ctor = "-1")]
    baz: i64,
}"#;

        let parsed = syn::parse_str(input).unwrap();
        let receiver = DeriveAttrInput::from_derive_input(&parsed).unwrap();
        println!("receiver: {receiver:#?}");
    }
}
