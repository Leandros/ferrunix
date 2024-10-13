#![allow(
    dead_code,
    clippy::option_if_let_else,
    clippy::min_ident_chars,
    clippy::manual_unwrap_or_default
)]
use std::borrow::Cow;

use darling::ast::Fields;
use darling::util::{Override, SpannedValue};
use darling::{util, FromDeriveInput, FromField};
use quote::quote;
use syn::Type;

use crate::utils::{transform_type, TransformType};

#[cfg(test)]
#[path = "./attr_tests.rs"]
mod tests;

#[derive(Debug, Clone, FromField)]
#[darling(attributes(inject), forward_attrs(allow, doc, cfg))]
pub(crate) struct DeriveField {
    // Magic types:
    /// The identifier of the passed-in field, or `None` for tuple fields.
    ident: Option<syn::Ident>,
    /// The visibility of the passed-in field.
    vis: syn::Visibility,
    /// The type of the passed-in field.
    ty: syn::Type,
    /// The forwarded attributes from the passed in field. These are controlled using the
    /// `forward_attrs` attribute.
    attrs: Vec<syn::Attribute>,

    //  ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫ Custom: ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
    /// Whether the member is injected as a transient. Defaults to `false`.
    #[darling(default)]
    transient: bool,

    /// Whether the member is injected as a singleton. Defaults to `false`.
    #[darling(default)]
    singleton: bool,

    /// Whether this member is constructed using `Default::default()`. Defaults to `false`.
    #[darling(default)]
    default: bool,

    /// If it's neither a transient, singleton, or default constructed, this is used as a
    /// constructor.
    ctor: Option<SpannedValue<String>>,
}

impl DeriveField {
    /// Get a reference to the identifier. Might be `None` for a tuple struct.
    pub(crate) fn ident(&self) -> Option<&syn::Ident> {
        self.ident.as_ref()
    }

    /// Get a reference to the type.
    pub(crate) fn ty(&self) -> &syn::Type {
        &self.ty
    }

    /// Get a reference to all attributes of the field.
    pub(crate) fn attrs(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    /// Whether the member is injected as a transient. Defaults to `false`.
    pub(crate) fn is_transient(&self) -> bool {
        self.transient
    }

    /// Whether the member is injected as a singleton. Defaults to `false`.
    pub(crate) fn is_singleton(&self) -> bool {
        self.singleton
    }

    /// Whether this member is constructed using `Default::default()`. Defaults to `false`.
    pub(crate) fn is_using_default_ctor(&self) -> bool {
        // The `ctor` overrides default construction.
        self.ctor.is_none() && self.default
    }

    /// If it's neither a transient, singleton, or default constructed, this is used as a
    /// constructor.
    pub(crate) fn ctor(&self) -> Option<&SpannedValue<String>> {
        self.ctor.as_ref()
    }
}

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(inject, provides), supports(struct_any))]
pub(crate) struct DeriveAttrInput {
    // Magic types:
    ident: syn::Ident,
    // generics: syn::Generics,
    data: darling::ast::Data<util::Ignored, DeriveField>,
    // attrs: Vec<syn::Attribute>,

    //  ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫ Custom: ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
    transient: Option<Override<Type>>,
    singleton: Option<Override<Type>>,
}

impl DeriveAttrInput {
    /// Iterator over the struct fields.
    pub(crate) fn fields(&self) -> Fields<DeriveField> {
        self.data.clone().take_struct().expect(
            "only structs supported. this should be enforced by darling.",
        )
    }

    /// Whether the `provides` attribute has set a transient value.
    /// Returns the value, or `Self`, when set, and `None`, when unset.
    ///
    /// Accepted forms are:
    ///   * `#[provides(transient)]`
    ///   * `#[provides(transient = "MyType")]`
    ///
    /// When the first form is used, the type is set to `Self`.
    pub(crate) fn transient(&self) -> Option<Cow<'_, Type>> {
        match &self.transient {
            Some(attr) => match attr {
                Override::Inherit => {
                    let tokens = quote!(Self);
                    let ty = syn::parse2(tokens).expect("Self to be valid");
                    Some(Cow::Owned(ty))
                }
                Override::Explicit(ty) => {
                    let ret = transform_type(ty, TransformType::Transient)
                        .expect("a well-formed type");
                    Some(ret)
                }
            },

            None => None,
        }
    }

    /// Whether the `provides` attribute has set a singleton value.
    /// Returns the value, or `Self`, when set, and `None`, when unset.
    ///
    /// Accepted forms are:
    ///   * `#[provides(singleton)]`
    ///   * `#[provides(singleton = "MyType")]`
    ///
    /// When the first form is used, the type is set to `Self`.
    pub(crate) fn singleton(&self) -> Option<Cow<'_, Type>> {
        match &self.singleton {
            Some(attr) => match attr {
                Override::Inherit => {
                    let tokens = quote!(Self);
                    let ty = syn::parse2(tokens).expect("Self to be valid");
                    Some(Cow::Owned(ty))
                }
                Override::Explicit(ty) => {
                    let ret = transform_type(ty, TransformType::Singleton)
                        .expect("a well-formed type");
                    Some(ret)
                }
            },

            None => None,
        }
    }
}
