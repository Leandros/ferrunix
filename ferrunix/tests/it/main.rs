//! Entrypoint for all integration tests.

mod common;

#[cfg(feature = "derive")]
// mod derive_simple;

#[cfg(not(feature = "tokio"))]
mod manual;
#[cfg(not(feature = "tokio"))]
mod manual_non_object_safe;
#[cfg(not(feature = "tokio"))]
mod manual_traits;
#[cfg(not(feature = "tokio"))]
mod validate_traits;

#[cfg(feature = "tokio")]
mod manual_async;
