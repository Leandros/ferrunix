//! Entrypoint for all integration tests.

mod common;
#[cfg(feature = "derive")]
mod derive_simple;
mod manual;
mod manual_non_object_safe;
mod manual_traits;
mod validate_traits;
