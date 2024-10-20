//! Builder singleton or transient objects, with our without dependencies.

#[cfg(all(not(feature = "multithread"), not(feature = "tokio")))]
#[path = "./object_builder_unsync.rs"]
pub(crate) mod inner;

#[cfg(feature = "tokio")]
#[path = "./object_builder_async.rs"]
pub(crate) mod inner;

#[cfg(feature = "multithread")]
#[path = "./object_builder_sync.rs"]
pub(crate) mod inner;

pub(crate) use inner::*;

/// All possible "objects" that can be held by the registry.
#[cfg(not(feature = "tokio"))]
pub(crate) enum Object {
    Transient(Box<dyn TransientBuilder>),
    Singleton(Box<dyn SingletonGetter>),
}

/// All possible "objects" that can be held by the registry.
#[cfg(feature = "tokio")]
pub(crate) enum Object {
    AsyncTransient(Box<dyn AsyncTransientBuilder + Send + Sync>),
    AsyncSingleton(Box<dyn AsyncSingleton + Send + Sync>),
}
