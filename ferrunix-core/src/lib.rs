//! Core types, traits, and implementations for [`ferrunix`].
//!
//! [`ferrunix`]: https://crates.io/crates/ferrunix
//! [`ferrunix-core`]: https://crates.io/crates/ferrunix-core

pub mod dependencies;
pub mod dependency_builder;
pub mod error;
pub mod object_builder;
pub mod registration;
pub mod registry;
pub mod types;

// Public re-exports for easier access.
// These are the main types users use for interacting with ferrunix.
#[doc(inline)]
pub use dependencies::Singleton;
#[doc(inline)]
pub use dependencies::Transient;
#[doc(inline)]
pub use registry::Registry;
#[doc(inline)]
pub use types::Ref;
