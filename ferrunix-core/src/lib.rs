//! Core types, traits, and implementations for [`ferrunix`].
//!
//! [`ferrunix`]: https://crates.io/crates/ferrunix
//! [`ferrunix-core`]: https://crates.io/crates/ferrunix-core

pub mod dependencies;
pub mod dependency_builder;
pub mod error;
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
