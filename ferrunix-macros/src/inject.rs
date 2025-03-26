//! `#[derive(Inject)]` implementation.
//!
//! Specifically, not in `lib.rs` to create module encapsulation.

use darling::ast::Fields;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput};

use crate::attr::{DeriveAttrInput, DeriveField};
use crate::utils::{get_ctor_for, strip_arc_rc_ref};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencyType {
    Singleton,
    Transient,
}

impl ToTokens for DependencyType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Singleton => tokens.append(format_ident!("singleton")),
            Self::Transient => tokens.append(format_ident!("transient")),
        }
    }
}

pub(crate) fn derive_macro_impl(
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    let registration = registration(input, attrs)?;
    let sig = register_func_sig();
    let boxed_registration = box_if_required(&registration);

    let autoregistration = {
        if attrs.no_registration() {
            None
        } else {
            Some(quote! {
                ::ferrunix::autoregister!(::ferrunix::RegistrationFunc::new(
                        <#struct_name>::register
                ));
            })
        }
    };

    let expanded = quote! {
        #[automatically_derived]
        impl #struct_name {
            #[allow(clippy::use_self, dead_code)]
            #sig {
                #boxed_registration
            }
        }

        #autoregistration
    };

    Ok(expanded)
}

fn register_func_sig() -> proc_macro2::TokenStream {
    #[cfg(not(feature = "tokio"))]
    quote! { pub(crate) fn register(registry: &::ferrunix::Registry) }

    #[cfg(feature = "tokio")]
    quote! {
        pub(crate) fn register<'reg>(
            registry: &'reg ::ferrunix::Registry,
        ) -> ::std::pin::Pin<
            ::std::boxed::Box<dyn ::std::future::Future<Output = ()> + Send + 'reg>,
        >
        where
            Self: Sync + 'static,
    }
}

fn box_if_required(
    tokens: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    #[cfg(not(feature = "tokio"))]
    {
        quote! { #tokens }
    }

    #[cfg(feature = "tokio")]
    {
        quote! {
            ::std::boxed::Box::pin(async move { #tokens })
        }
    }
}

#[allow(unused)]
fn box_ctor_if_required(
    registered_ty: &syn::Type,
    tokens: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    #[cfg(not(feature = "tokio"))]
    {
        quote! { #tokens }
    }

    #[cfg(feature = "tokio")]
    {
        quote! {
            ::std::boxed::Box::pin(async move { #tokens as #registered_ty })
        }
    }
}

fn await_if_needed() -> Option<proc_macro2::TokenStream> {
    (cfg!(feature = "tokio")).then(|| {
        quote! {
           .await
        }
    })
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
    let registered_ty = attrs.transient().expect("transient attribute");
    // eprintln!("transient: registered ty => {}", quote!(#registered_ty));

    if fields_is_empty {
        registration_empty(DependencyType::Transient, &registered_ty)
    } else {
        registration_fields(
            DependencyType::Transient,
            &registered_ty,
            input,
            attrs,
        )
    }
}

/// Called to create a registration for types with no direct injected dependencies.
fn registration_empty(
    dependency_type: DependencyType,
    registered_ty: &syn::Type,
) -> syn::Result<proc_macro2::TokenStream> {
    let ctor = get_ctor_for(registered_ty, quote!(Self {}))?;
    let ctor = box_ctor_if_required(registered_ty, &ctor);
    let ifawait = await_if_needed();
    let generic_args = {
        match dependency_type {
            DependencyType::Singleton => quote! { <#registered_ty, _> },
            DependencyType::Transient => quote! { <#registered_ty> },
        }
    };

    let tokens = quote! {
        registry.#dependency_type::#generic_args(|| {
            #ctor
        })#ifawait;
    };

    Ok(tokens)
}

/// Called to create a registration for types with  direct injected dependencies.
fn registration_fields(
    dependency_type: DependencyType,
    registered_ty: &syn::Type,
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> syn::Result<proc_macro2::TokenStream> {
    // let current_ty = &input.ident;

    let fields = attrs.fields();
    let dependency_tuple = into_dependency_tuple(&fields);
    let dependency_idents = into_dependency_idents(&fields);
    let constructor = type_ctor(registered_ty, input, attrs, &fields)?;
    let constructor = box_ctor_if_required(registered_ty, &constructor);
    let ifawait = await_if_needed();
    let generic_args = {
        match dependency_type {
            DependencyType::Singleton => quote! { <#registered_ty, _> },
            DependencyType::Transient => quote! { <#registered_ty> },
        }
    };

    let tokens = match (dependency_tuple, dependency_idents) {
        (Some(types), Some(idents)) => {
            quote! {
                registry
                    .with_deps::<#registered_ty, #types>()
                    .#dependency_type(|#idents| {
                        #constructor
                    })#ifawait;
            }
        }

        _ => {
            quote! {
                registry.#dependency_type::#generic_args(|| {
                    #constructor
                })#ifawait;
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
        let ty = strip_arc_rc_ref(ty).ok()?;
        Some(quote! { ::ferrunix::Singleton<#ty> })
    } else {
        None
    }
}

fn type_ctor(
    registered_ty: &syn::Type,
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
    fields: &Fields<DeriveField>,
) -> syn::Result<proc_macro2::TokenStream> {
    let params = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| !field.not_injected())
        .map(|(idx, field)| field_ctor_rhs(idx, field))
        .collect::<syn::Result<Vec<_>>>()?;
    if let Some(ctor_name) = attrs.custom_ctor() {
        let ctor_name = ctor_name.as_ident();
        let ctor = get_ctor_for(
            registered_ty,
            quote! {
                Self::#ctor_name(#(#params),*)
            },
        );
        let ctor = ctor?;

        return Ok(ctor);
    }

    let ctors = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| field_ctor(idx, field))
        .collect::<syn::Result<Vec<_>>>()?;

    if let Data::Struct(ref s) = input.data {
        match s.fields {
            syn::Fields::Named(_) => {
                let ctor = get_ctor_for(
                    registered_ty,
                    quote! { Self { #(#ctors),* } },
                )?;

                return Ok(ctor);
            }
            syn::Fields::Unnamed(_) => {
                let ctor = get_ctor_for(
                    registered_ty,
                    quote! { Self ( #(#ctors),* ) },
                )?;
                return Ok(ctor);
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

    let ctor = field_ctor_rhs(idx, attrs)?;

    // We have a named struct.
    let tokens = if attrs.ident().is_some() {
        quote! { #ident : #ctor }
    } else {
        // We have an unnamed/tuple struct.
        quote! { #ctor }
    };

    Ok(tokens)
}

fn field_ctor_rhs(
    idx: usize,
    attrs: &DeriveField,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = attrs
        .ident()
        .cloned()
        .unwrap_or_else(|| format_ident!("_{idx}"));

    if attrs.is_transient() || attrs.is_singleton() {
        Ok(quote! { #ident.get() })
    } else if let Some(ctor) = attrs.ctor() {
        let parsed = syn::parse_str::<syn::Expr>(ctor);
        if let Err(err) = parsed {
            return Err(syn::Error::new(
                ctor.span(),
                format!(
                    "couldn't parse ctor expression: {err}\n\nTo \
                         construct a string, you need to double quote it."
                ),
            ));
        };

        let parsed = parsed.expect("error handled above");
        Ok(quote! { #parsed })
    } else {
        // Always fall back to `Default::default()`.
        Ok(quote! { Default::default() })
    }
}

fn registration_singleton(
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields_is_empty = attrs.fields().is_empty();
    let registered_ty = attrs.singleton().expect("singleton attribute");
    // eprintln!("singleton: registered ty => {}", quote!(#registered_ty));

    if fields_is_empty {
        registration_empty(DependencyType::Singleton, &registered_ty)
    } else {
        registration_fields(
            DependencyType::Singleton,
            &registered_ty,
            input,
            attrs,
        )
    }
}
