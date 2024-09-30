use darling::{util, FromDeriveInput, FromField};
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
    inject: Option<bool>,
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
    inject: Option<bool>,
}

#[proc_macro_derive(Inject, attributes(inject))]
pub fn derive_inject(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    // eprintln!("input: {input:#?}");

    // let attr_input = DeriveAttrInput::from_derive_input(&input).unwrap();
    // eprintln!("attr_input: {attr_input:#?}");


    let expanded = quote! {
        fn foo() -> i32 { 42 }
    };

    proc_macro::TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attrs() {
        let input = r#"
#[derive(Inject)]
pub struct Foo {
    #[inject]
    bar: u8,
    baz: i64,
}"#;

        let parsed = syn::parse_str(input).unwrap();
        let receiver = DeriveAttrInput::from_derive_input(&parsed).unwrap();
        println!("receiver: {receiver:#?}");
    }
}
