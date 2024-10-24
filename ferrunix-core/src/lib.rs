#![cfg_attr(
    feature = "multithread",
    doc = "_You're viewing the documentation with the **`multithread` feature** turned on._\n\n"
)]
#![cfg_attr(
    feature = "tokio",
    doc = "_You're viewing the documentation with the **`tokio` feature** turned on._\n\n"
)]
#![cfg_attr(
    all(not(feature = "tokio"), not(feature = "multithread")),
    doc = "#### _You're viewing the documentation with **`no features`** turned on._\n\n"
)]
//! Core types, traits, and implementations for [`ferrunix`].
//!
//! [`ferrunix`]: https://crates.io/crates/ferrunix

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
