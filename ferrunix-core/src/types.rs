//! Type aliases.
#![allow(
    clippy::single_char_lifetime_names,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    dead_code
)]

mod private {
    /// This is used for sealing the traits [`SingletonCtor`] and [`SingletonCtorDeps`].
    #[derive(Debug, Clone, Copy)]
    pub struct SealToken;
}

use std::any::TypeId;

use crate::cycle_detection::{DependencyValidator, VisitorContext};

// Alias types used in [`DependencyValidator`].
pub(crate) struct Visitor(
    pub(crate)  fn(
        &DependencyValidator,
        &HashMap<TypeId, Visitor>,
        &mut VisitorContext,
    ) -> petgraph::graph::NodeIndex,
);

/// Types that are enabled when the `multithread` feature is set.
#[cfg(all(feature = "multithread", not(feature = "tokio")))]
#[path = "./types_sync.rs"]
mod sync;

/// Types that are enabled when the `multithread` feature is **NOT** set.
#[cfg(all(not(feature = "multithread"), not(feature = "tokio")))]
#[path = "./types_unsync.rs"]
mod unsync;

#[cfg(feature = "tokio")]
#[path = "./types_tokio.rs"]
mod tokio_ext;

#[cfg(all(feature = "multithread", not(feature = "tokio")))]
pub use sync::*;

#[cfg(all(not(feature = "multithread"), not(feature = "tokio")))]
pub use unsync::*;

#[cfg(feature = "tokio")]
pub use tokio_ext::*;
