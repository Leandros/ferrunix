//! Entrypoint for all integration tests.

mod common;
mod validate_traits;

#[cfg(feature = "derive")]
// TODO
// mod derive_simple;
#[cfg(not(feature = "tokio"))]
mod manual;
#[cfg(not(feature = "tokio"))]
mod manual_non_object_safe;
#[cfg(not(feature = "tokio"))]
mod manual_traits;

#[cfg(feature = "tokio")]
mod manual_async;
