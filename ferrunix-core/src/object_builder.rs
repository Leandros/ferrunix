//! Builder singleton or transient objects, with our without dependencies.

#[cfg(not(feature = "tokio"))]
#[path = "./object_builder_sync.rs"]
pub(crate) mod inner;

#[cfg(feature = "tokio")]
#[path = "./object_builder_async.rs"]
pub(crate) mod inner;

pub(crate) use inner::*;

/// All possible "objects" that can be held by the registry.
#[cfg(not(feature = "tokio"))]
pub(crate) enum Object {
    /// An object with transient lifetime.
    Transient(crate::types::BoxedTransientBuilder),
    /// An object with singleton lifetime.
    Singleton(crate::types::BoxedSingletonGetter),
}

/// All possible "objects" that can be held by the registry.
#[cfg(feature = "tokio")]
pub(crate) enum Object {
    /// An object with transient lifetime.
    AsyncTransient(Box<dyn AsyncTransientBuilder + Send + Sync>),
    /// An object with singleton lifetime.
    AsyncSingleton(Box<dyn AsyncSingleton + Send + Sync>),
}
