//! Utilities for the proc-macro
#![allow(unused)]

use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Data, Field, Fields};

pub(crate) fn get_fields_from_struct(data: &Data) -> (bool, Punctuated<Field, Comma>) {
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
