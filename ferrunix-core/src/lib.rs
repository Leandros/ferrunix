//! Core types, traits, and implementations for [`ferrunix`].
//!
//! [`ferrunix`]: https://crates.io/crates/ferrunix
//! [`ferrunix-core`]: https://crates.io/crates/ferrunix-core
// #![doc(test(attr(cfg(not(feature = "tokio")))))]

pub mod dependencies;
pub mod dependency_builder;
pub mod error;
#[doc(hidden)]
pub mod object_builder;
#[doc(hidden)]
pub mod registration;
pub mod registry;
#[doc(hidden)]
pub mod types;

// Public re-exports for easier access.
// These are the main types users use for interacting with ferrunix.
pub use dependencies::Singleton;
pub use dependencies::Transient;
pub use registry::Registry;

#[cfg(all(feature = "tokio", feature = "multithread"))]
compile_error!(
    "the `ferrunix-core/tokio` and `ferrunix-core/multithread` feature are mutually exclusive"
);
