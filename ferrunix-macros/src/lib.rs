#![allow(clippy::panic, clippy::module_name_repetitions)]

use darling::FromDeriveInput;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

use self::attr::DeriveAttrInput;

mod attr;

/// # Panics
/// TODO
#[proc_macro_derive(Inject, attributes(provides, inject))]
pub fn derive_inject(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    // eprintln!("input: {input:#?}");

    let attr_input =
        DeriveAttrInput::from_derive_input(&input).expect("invalid attributes");
    // eprintln!("attr_input: {attr_input:#?}");

    let struct_name = input.ident.clone();
    let struct_name_lowercase = input.ident.to_string().to_lowercase();
    let mod_name = format_ident!("__inner_register_{struct_name_lowercase}");

    let registration = build_registration(&input, &attr_input);
    let expanded = quote! {
        mod #mod_name {
            use super::*;

            #[automatically_derived]
            impl #struct_name {
                pub(crate) fn register(registry: &mut ::ferrunix::Registry) {
                    #registration
                }
            }

            ::ferrunix::inventory_submit!(::ferrunix::RegistrationFunc(|registry| {
                <#struct_name>::register(registry);
            }));
        }
    };

    eprintln!("{expanded}");
    proc_macro::TokenStream::from(expanded)
}

fn build_registration(
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> proc_macro2::TokenStream {
    if attrs.transient().is_some() {
        build_registration_transient(input, attrs)
    } else if attrs.singleton().is_some() {
        build_registration_singleton(input, attrs)
    } else {
        panic!("missing transient or singleton annotation")
    }
}

fn build_registration_transient(
    input: &DeriveInput,
    attrs: &DeriveAttrInput,
) -> proc_macro2::TokenStream {
    let fields = attrs.fields();
    let raw_ty = attrs.transient().as_ref().expect("transient attribute");

    let concrete_ty = &input.ident;
    if fields.is_empty() {
        // TODO: Implement `ctor` and `default` attr.
        // let attr_field = &fields.fields[i];
        quote! {
            registry.transient::<Box<#raw_ty>>(|| {
                Box::new(<#concrete_ty>::default())
            });
        }
    } else {
        let (is_named, struct_fields) = get_fields_from_struct(&input.data);
        let (idents, types, ctors) = {
            let mut idents = Vec::new();
            let mut types = Vec::new();
            let mut ctors = Vec::new();
            for i in 0..fields.len() {
                // TODO: Implement `ctor` and `default` attr.
                // let attr_field = &fields.fields[i];
                let input_field = &struct_fields[i];

                let ident = input_field
                    .ident
                    .clone()
                    .unwrap_or_else(|| format_ident!("field{i}"));

                idents.push(ident.clone());
                types.push(input_field.ty.clone());
                ctors.push(quote! {
                    #ident.get()
                });
            }
            (idents, types, ctors)
        };

        let construct = if is_named {
            quote! {
                #concrete_ty {
                    #(#idents : #ctors),*
                }
            }
        } else {
            quote! {
                #concrete_ty (
                    #(<#idents>.get()),*
                )
            }
        };
        quote! {
            registry
                .with_deps::<Box<#raw_ty>, (
                    #(::ferrunix::Transient<#types>),*
                )>()
                .transient(|(#(#idents),*)| {
                    Box::new(#construct)
                });
        }
    }
}

fn build_registration_singleton(
    _input: &DeriveInput,
    _attrs: &DeriveAttrInput,
) -> proc_macro2::TokenStream {
    panic!()
}

fn get_fields_from_struct(data: &Data) -> (bool, Punctuated<Field, Comma>) {
    match data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref named) => (true, named.named.clone()),
            Fields::Unnamed(ref unnamed) => (false, unnamed.unnamed.clone()),
            Fields::Unit => panic!("structs must be constructible"),
        },
        Data::Enum(_) | Data::Union(_) => panic!("not supported"),
    }
}

// fn type_is_boxed(ty: &Type) -> bool {
//     match ty {
//         Type::Array(_) => false,
//         Type::BareFn(_) => false,
//         Type::Group(_) => false,
//         Type::ImplTrait(_) => false,
//         Type::Infer(_) => false,
//         Type::Macro(_) => false,
//         Type::Never(_) => false,
//         Type::Paren(_) => false,
//         Type::Path(path) => {
//         // Let else not stabilized on 1.64.0. TODO: Replace.
//             let Some(first_segement) = path.path.segments.get(0) else {
//                 return false;
//             };
//             first_segement.ident == "Box"
//         }
//         Type::Ptr(_) => false,
//         Type::Reference(_) => false,
//         Type::Slice(_) => false,
//         Type::TraitObject(_) => false,
//         Type::Tuple(_) => false,
//         Type::Verbatim(_) => false,
//         _ => false,
//     }
// }
