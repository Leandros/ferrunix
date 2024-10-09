#![allow(
    dead_code,
    clippy::option_if_let_else,
    clippy::min_ident_chars,
    clippy::manual_unwrap_or_default
)]
use darling::ast::Fields;
use darling::{util, FromDeriveInput, FromField};
use syn::Type;

#[cfg(test)]
#[path = "./attr_tests.rs"]
mod tests;

#[derive(Debug, Clone, FromField)]
#[darling(attributes(inject), forward_attrs(allow, doc, cfg))]
pub struct DeriveField {
    // Magic types:
    ident: Option<syn::Ident>,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,

    // Custom:
    #[darling(default)]
    transient: bool,
    #[darling(default)]
    singleton: bool,
    #[darling(default)]
    default: bool,
    ctor: Option<String>,
}

impl DeriveField {
    pub fn ident(&self) -> Option<&syn::Ident> {
        self.ident.as_ref()
    }

    pub fn ty(&self) -> &syn::Type {
        &self.ty
    }

    pub fn attrs(&self) -> &Vec<syn::Attribute> {
        &self.attrs
    }

    pub fn default_ctor(&self) -> bool {
        // The `ctor` overrides default construction.
        self.ctor.is_none() && self.default
    }

    pub fn ctor(&self) -> Option<&String> {
        self.ctor.as_ref()
    }
}

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(inject, provides), supports(struct_any))]
pub struct DeriveAttrInput {
    // Magic types:
    ident: syn::Ident,
    // generics: syn::Generics,
    data: darling::ast::Data<util::Ignored, DeriveField>,
    // attrs: Vec<syn::Attribute>,

    // Custom:
    transient: Option<Type>,
    singleton: Option<String>,
}

impl DeriveAttrInput {
    pub fn fields(&self) -> Fields<DeriveField> {
        self.data.clone().take_struct().expect(
            "only structs supported. this should be enforced by darling.",
        )
    }

    pub fn transient(&self) -> Option<&Type> {
        self.transient.as_ref()
    }

    pub fn singleton(&self) -> Option<&String> {
        self.singleton.as_ref()
    }

    // fn opts(&self) -> Cow<'_, Options> {
    //     match &self.opts {
    //         Override::Explicit(value) => Cow::Borrowed(value),
    //         Override::Inherit => Cow::Owned(Options {
    //             ty: self.ident.to_string(),
    //         }),
    //     }
    // }
}
