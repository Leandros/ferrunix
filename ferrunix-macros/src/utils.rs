//! Utilities for the proc-macro
#![allow(unused)]

use std::borrow::Cow;

use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Data, Field, Fields, PathArguments, PathSegment};

#[cfg(test)]
#[path = "./utils_test.rs"]
mod tests;

pub(crate) fn get_ctor_for(
    ty: &syn::Type,
    inner: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    // eprintln!("get_ctor_for: {ty:?}");
    let span = ty.span();
    match ty {
        syn::Type::Path(ref path) => {
            let segments = &path.path.segments.iter().collect::<Vec<_>>();
            let len = segments.len();
            let is_std_type = segments
                .first()
                .map_or_else(|| false, |seg| seg.ident == format_ident!("std"));
            let is_our_type = segments.first().map_or_else(
                || false,
                |seg| seg.ident == format_ident!("ferrunix"),
            );

            let supported_types = [
                ("Box", "::std::boxed::Box", "new"),
                ("Rc", "::std::rc::Rc", "new"),
                ("Arc", "::std::sync::Arc", "new"),
                ("RwLock", "::sync::RwLock", "new"),
                ("Mutex", "::std::sync::Mutex", "new"),
                ("Option", "::std::option::Option", "new"),
                ("Result", "::std::result::Result", "new"),
                ("Vec", "::std::vec::Vec", "new"),
                ("Cell", "::std::cell::Cell", "new"),
                ("RefCell", "::std::cell::RefCell", "new"),
                ("Ref", "::ferrunix::Ref", "new"),
            ];

            let is_supported_type = |segment: &PathSegment| {
                if let Some((_name, fullname, ctor)) =
                    supported_types.iter().find(|(ident, _fullname, _ctor)| {
                        segment.ident == format_ident!("{ident}")
                    })
                {
                    let fullname: syn::Type =
                        syn::parse_str(fullname).expect("fullname to be valid");
                    let ctor = format_ident!("{ctor}");
                    return Some(quote! {
                        #fullname::#ctor(#inner)
                    });
                }

                None
            };

            if is_std_type || is_our_type {
                for segment in segments {
                    if let Some(tokens) = is_supported_type(segment) {
                        return Ok(tokens);
                    }
                }
            } else if let Some(segment) = segments.first() {
                if let Some(tokens) = is_supported_type(segment) {
                    return Ok(tokens);
                }
            }

            Ok(inner)
        }

        unsupported => Err(syn::Error::new(
            span,
            format!("unsupported type: {unsupported:?}"),
        )),
    }
}

pub(crate) enum TransformType {
    Transient,
    Singleton,
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn transform_type(
    ty: &'_ syn::Type,
    what: TransformType,
) -> syn::Result<Cow<'_, syn::Type>> {
    let span = ty.span();
    match what {
        TransformType::Transient => match ty {
            syn::Type::TraitObject(obj) => {
                let ret: syn::Type =
                    syn::parse2(quote! { ::std::boxed::Box<#obj> })?;
                Ok(Cow::Owned(ret))
            }

            _ => Ok(Cow::Borrowed(ty)),
        },

        TransformType::Singleton => match ty {
            syn::Type::Path(path) => Ok(Cow::Borrowed(ty)),

            syn::Type::TraitObject(obj) => {
                let ret: syn::Type =
                    syn::parse2(quote! { ::std::boxed::Box<#obj> })?;
                Ok(Cow::Owned(ret))
            }

            _ => Ok(Cow::Borrowed(ty)),
        },
    }
}

#[allow(clippy::similar_names)]
pub(crate) fn strip_arc_rc_ref(
    ty: &'_ syn::Type,
) -> syn::Result<Cow<'_, syn::Type>> {
    let span = ty.span();
    match ty {
        syn::Type::Path(path) => {
            let segments = &path.path.segments.iter().collect::<Vec<_>>();
            let len = segments.len();

            let std_arc = (|| -> Option<PathArguments> {
                if segments.first()?.ident == format_ident!("std") {
                    if segments.get(1)?.ident == format_ident!("sync")
                        && segments.get(2)?.ident == format_ident!("Arc")
                    {
                        return Some(segments.get(2)?.arguments.clone());
                    }
                } else if segments.first()?.ident == format_ident!("Arc") {
                    return Some(segments.first()?.arguments.clone());
                }

                None
            })();
            let std_rc = (|| -> Option<PathArguments> {
                if segments.first()?.ident == format_ident!("std") {
                    if segments.get(1)?.ident == format_ident!("rc")
                        && segments.get(2)?.ident == format_ident!("Rc")
                    {
                        return Some(segments.get(2)?.arguments.clone());
                    }
                } else if segments.first()?.ident == format_ident!("Rc") {
                    return Some(segments.first()?.arguments.clone());
                }

                None
            })();
            let ferrunix_ref = (|| -> Option<PathArguments> {
                if segments.first()?.ident == format_ident!("ferrunix") {
                    if segments.get(1)?.ident == format_ident!("Ref") {
                        return Some(segments.get(1)?.arguments.clone());
                    }
                } else if segments.first()?.ident == format_ident!("Ref") {
                    return Some(segments.first()?.arguments.clone());
                }

                None
            })();

            let ((None, None, Some(path_args))
            | (None, Some(path_args), None)
            | (Some(path_args), None, None)) = (std_arc, std_rc, ferrunix_ref)
            else {
                return Ok(Cow::Borrowed(ty));
            };

            match path_args {
                PathArguments::AngleBracketed(args) => {
                    let first =
                        args.args.first().expect("missing generic argument");
                    match first {
                        syn::GenericArgument::Type(generic_ty) => {
                            return Ok(Cow::Owned(generic_ty.clone()))
                        }
                        unsupported => {
                            return Err(syn::Error::new(
                                span,
                                format!(
                                    "unsupported generic arg: {unsupported:?}"
                                ),
                            ))
                        }
                    }
                }

                unsupported => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported type path: {unsupported:?}"),
                    ))
                }
            }

            Ok(Cow::Borrowed(ty))
        }

        unsupported => Err(syn::Error::new(
            span,
            format!("unsupported type: {unsupported:?}"),
        )),
    }
}
