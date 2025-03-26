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
//! # Ferrunix
//!
//! A simple, idiomatic, and lightweight [dependency injection] framework for Rust.
//!
//! **For a comprehensive introduction check out the [user guide].**
//!
//! ## Documentation
//!
//! Due to how the various features affect the public API of the library, the
//! documentation is provided seperately for each major feature.
//!
//! _Documentation on [docs.rs] is compiled with the `multithread` feature turned on_
//!
//! |    Feature Flags    | Link to Documentation |
//! | ------------------- | --------------------- |
//! | `none`              | [link to docs](https://leandros.github.io/ferrunix/docs-default/ferrunix/)     |
//! | `multithread`       | [link to docs](https://leandros.github.io/ferrunix/docs-multithread/ferrunix/) |
//! | `tokio`             | [link to docs](https://leandros.github.io/ferrunix/docs-multithread/ferrunix/) |
//!
//! ## Getting Started
//!
//! The easiest way to get started is with the multi-threaded registry, and the derive macro,
//! you can enable this as follows:
//!
//! ```toml
//! [dependencies]
//! ferrunix = { version = "0.4", features = ["multithread"] }
//! ```
//!
//! ## Cargo Feature Flags
//!
//! Ferrunix has the following [features] to enable further functionality.
//! Features enabled by default are marked with `*`.
//!
//! - `multithread`: Enables support for accessing the registry from multiple
//!     threads. This adds a bound that all registered types must be `Send`.
//! - `derive` (`*`): Enables support for the `#[derive(Inject)]` macro.
//! - `tokio`: Enables support for `async` constructors. Bumps the MSRV up to
//!     `1.75.0` because some of the internal traits require [RPITIT].
//! - `tracing`: Enables support for [tracing] and annotates all public functions with
//!     [`tracing::instrument`].
//!
//! [dependency injection]: https://en.wikipedia.org/wiki/Dependency_injection
//! [docs.rs]: https://docs.rs/ferrunix
//! [user guide]: https://leandros.github.io/ferrunix/user-guide/first-steps.html
//! [RPITIT]: https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html#whats-stabilizing
//! [features]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
//! [tracing]: https://docs.rs/tracing/latest/tracing/index.html
//! [`tracing::instrument`]: https://docs.rs/tracing/latest/tracing/attr.instrument.html

pub use ferrunix_core::dependencies;
pub use ferrunix_core::dependency_builder;
pub use ferrunix_core::registry;
pub use ferrunix_core::types;

pub use dependencies::Singleton;
pub use dependencies::Transient;
pub use registry::Registry;

#[cfg(feature = "derive")]
pub use ferrunix_macros::Inject;

/// Register a [`RegistrationFunc`]. Usually invoked by the derive macro.
///
pub use ferrunix_core::registration::autoregister;
pub use ferrunix_core::registration::RegistrationFunc;

pub use ferrunix_core::types::Ref;
