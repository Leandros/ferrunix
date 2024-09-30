use darling::{FromDeriveInput, FromField};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromField)]
#[darling(attributes(inject), forward_attrs(allow, doc, cfg))]
struct Attrs {
    // Magic types:
    ident: Option<syn::Ident>,
    attrs: Vec<syn::Attribute>,
    ty: syn::Type,
    // Custom:
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inject))]
struct DeriveAttrInput {
    /// The struct ident.
    ident: syn::Ident,

    /// The type's generics. You'll need these any time your trait is expected
    /// to work with types that declare generics.
    generics: syn::Generics,

    // Receives the body of the struct or enum. We don't care about
    // struct fields because we previously told darling we only accept structs.
    data: darling::ast::Data<(), Attrs>,
}

#[proc_macro_derive(Inject, attributes(inject))]
pub fn derive_inject(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

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
