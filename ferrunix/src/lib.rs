//! A lightweight run-time dependency injection framework for Rust.
//!

pub use ferrunix_core::dependencies;
pub use ferrunix_core::dependency_builder;
pub use ferrunix_core::registry;

pub use dependencies::Singleton;
pub use dependencies::Transient;
pub use registry::Registry;

pub use ferrunix_macros::Inject;

pub use ferrunix_core::registration::autoregister;
pub use ferrunix_core::registration::RegistrationFunc;

pub use ferrunix_core::types::Ref;
