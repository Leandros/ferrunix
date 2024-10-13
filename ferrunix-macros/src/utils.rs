//! Utilities for the proc-macro
#![allow(unused)]

use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Data, Field, Fields};

pub(crate) fn get_fields_from_struct(
    data: &Data,
) -> (bool, Punctuated<Field, Comma>) {
    match data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref named) => (true, named.named.clone()),
            Fields::Unnamed(ref unnamed) => (false, unnamed.unnamed.clone()),
            Fields::Unit => panic!("structs must be constructible"),
        },
        Data::Enum(_) | Data::Union(_) => panic!("not supported"),
    }
}

pub(crate) fn get_ctor_for(
    ty: &syn::Type,
    inner: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let span = ty.span();
    match ty {
        syn::Type::Path(ref path) => {
            let segments = &path.path.segments;
            if let Some(first) = segments.first() {
                let supported_types = [
                    ("Box", "new"),
                    ("Rc", "new"),
                    ("Arc", "new"),
                    ("RwLock", "new"),
                    ("Mutex", "new"),
                    ("Option", "new"),
                    ("Result", "new"),
                    ("Vec", "new"),
                    ("Cell", "new"),
                    ("RefCell", "new"),
                ];

                if let Some((_, ctor)) =
                    supported_types.iter().find(|(ident, _ctor)| {
                        first.ident == format_ident!("{ident}")
                    })
                {
                    let ident = first.ident.clone();
                    let ctor = format_ident!("{ctor}");
                    return Ok(quote! {
                        #ident::#ctor(#inner)
                    });
                }
            }

            Ok(inner)
        }

        unsupported => {
            Err(syn::Error::new(span, "unsupported type: {unsupported}"))
        }
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
