//! `#[derive(Inject)]` implementation.
//!
//! Specifically, not in `lib.rs` to create module encapsulation.

use darling::ast::Fields;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput};

use crate::attr::{DeriveAttrInput, DeriveField};

pub(crate) fn derive_macro_impl(
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    let registration = registration(input, attrs)?;
    let expanded = quote! {
        #[automatically_derived]
        impl #struct_name {
            #[allow(clippy::use_self)]
            pub(crate) fn register(registry: &::ferrunix::Registry) {
                #registration
            }
        }

        ::ferrunix::autoregister!(::ferrunix::RegistrationFunc::new(
            <#struct_name>::register
        ));
    };

    Ok(expanded)
}

fn registration(
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> syn::Result<proc_macro2::TokenStream> {
    if attrs.transient().is_some() {
        registration_transient(input, attrs)
    } else if attrs.singleton().is_some() {
        registration_singleton(input, attrs)
    } else {
        // eprintln!("input: {input:#?}");
        // eprintln!("attrs: {attrs:#?}");
        Err(syn::Error::new(
            input.span(),
            "missing transient or singleton annotation.",
        ))
    }
}

fn registration_transient(
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_is_empty = attrs.fields().is_empty();

    if fields_is_empty {
        Ok(registration_transient_empty(input, attrs))
    } else {
        registration_transient_fields(input, attrs)
    }
}

fn registration_transient_empty(
    _input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> proc_macro2::TokenStream {
    let registered_ty = attrs.transient().expect("transient attribute");

    let tokens = quote! {
        registry.transient::<#registered_ty>(|| {
            #registered_ty {}
        });
    };

    tokens
}

fn registration_transient_fields(
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> syn::Result<proc_macro2::TokenStream> {
    // let current_ty = &input.ident;
    let registered_ty = attrs.transient().expect("transient attribute");

    let fields = attrs.fields();
    let dependency_tuple = into_dependency_tuple(&fields);
    let dependency_idents = into_dependency_idents(&fields);
    let constructor = type_ctor(input, &fields)?;

    let tokens = match (dependency_tuple, dependency_idents) {
        (Some(types), Some(idents)) => {
            quote! {
                registry
                    .with_deps::<#registered_ty, #types>()
                    .transient(|#idents| {
                        #constructor
                    });
            }
        }

        _ => {
            quote! {
                registry.transient::<#registered_ty>(|| {
                    #constructor
                });
            }
        }
    };

    Ok(tokens)
}

fn into_dependency_idents(
    fields: &Fields<DeriveField>,
) -> Option<proc_macro2::TokenStream> {
    let idents = fields
        .iter()
        .enumerate()
        .filter_map(|(i, field)| {
            let ident = field
                .ident()
                .cloned()
                .unwrap_or_else(|| format_ident!("_{i}"));
            (field.is_transient() || field.is_singleton()).then_some(ident)
        })
        .collect::<Vec<_>>();
    if !idents.is_empty() {
        return Some(quote! { ( #(#idents,)* ) });
    }

    None
}

fn into_dependency_tuple(
    fields: &Fields<DeriveField>,
) -> Option<proc_macro2::TokenStream> {
    let types = fields
        .iter()
        .filter_map(into_dependency_type)
        .collect::<Vec<_>>();
    if !types.is_empty() {
        return Some(quote! { ( #(#types,)* ) });
    }

    None
}

fn into_dependency_type(
    field: &DeriveField,
) -> Option<proc_macro2::TokenStream> {
    let ty = field.ty();
    if field.is_transient() {
        Some(quote! { ::ferrunix::Transient<#ty> })
    } else if field.is_singleton() {
        Some(quote! { ::ferrunix::Singleton<#ty> })
    } else {
        None
    }
}

fn type_ctor(
    input: &DeriveInput,
    fields: &Fields<DeriveField>,
) -> syn::Result<proc_macro2::TokenStream> {
    let ctors = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| field_ctor(idx, field))
        .collect::<syn::Result<Vec<_>>>()?;
    if let Data::Struct(ref s) = input.data {
        match s.fields {
            syn::Fields::Named(_) => {
                return Ok(quote!( Self { #(#ctors),* } ));
            }
            syn::Fields::Unnamed(_) => {
                return Ok(quote!( Self ( #(#ctors),* ) ));
            }
            syn::Fields::Unit => (),
        }
    }

    Err(syn::Error::new(
        input.span(),
        "only named and unnamed structs supported",
    ))
}

fn field_ctor(
    idx: usize,
    attrs: &DeriveField,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = attrs
        .ident()
        .cloned()
        .unwrap_or_else(|| format_ident!("_{idx}"));

    let ctor = {
        if attrs.is_transient() || attrs.is_singleton() {
            quote! { #ident.get() }
        } else if let Some(ctor) = attrs.ctor() {
            let parsed = syn::parse_str::<syn::Expr>(ctor);
            if let Err(err) = parsed {
                return Err(syn::Error::new(
                    ctor.span(),
                    format!("couldn't parse ctor expression: {err}\n\nTo construct a string, you need to double quote it."),
                ));
            };

            let parsed = parsed.expect("error handled above");
            quote! { #parsed }
        } else {
            // Always fall back to `Default::default()`.
            quote! { Default::default() }
        }
    };

    // We have a named struct.
    let tokens = if attrs.ident().is_some() {
        quote! { #ident : #ctor }
    } else {
        // We have an unnamed/tuple struct.
        quote! { #ctor }
    };

    Ok(tokens)
}

fn registration_singleton(
    _input: &DeriveInput,
    _attrs: &DeriveAttrInput,
) -> syn::Result<proc_macro2::TokenStream> {
    panic!()
}
