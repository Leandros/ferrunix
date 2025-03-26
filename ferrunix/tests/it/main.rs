//! Entrypoint for all integration tests.

mod common;
mod cycle_test;
mod stress;
mod validate_traits;

#[cfg(all(feature = "derive", feature = "tokio"))]
mod derive_async;
#[cfg(feature = "derive")]
mod derive_ctor;
#[cfg(feature = "derive")]
mod derive_registration;
#[cfg(feature = "derive")]
mod derive_regression;
#[cfg(all(feature = "derive", not(feature = "tokio")))]
mod derive_simple;
#[cfg(all(feature = "derive", not(feature = "tokio")))]
mod derive_singleton;

#[cfg(not(feature = "tokio"))]
mod manual;
#[cfg(not(feature = "tokio"))]
mod manual_non_object_safe;
#[cfg(not(feature = "tokio"))]
mod manual_traits;

#[cfg(feature = "tokio")]
mod manual_async;
