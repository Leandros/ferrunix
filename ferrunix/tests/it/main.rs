//! Entrypoint for all integration tests.

mod common;
mod cycle_test;
mod stress;
mod validate_traits;

#[cfg(all(feature = "derive", feature = "tokio"))]
mod derive_async;
#[cfg(all(feature = "derive", not(feature = "tokio")))]
mod derive_simple;

#[cfg(not(feature = "tokio"))]
mod manual;
#[cfg(not(feature = "tokio"))]
mod manual_non_object_safe;
#[cfg(not(feature = "tokio"))]
mod manual_traits;

#[cfg(feature = "tokio")]
mod manual_async;
