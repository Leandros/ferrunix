//! Various type aliases for the tokio implementation.

use std::any::Any;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

// Alias types used in [`Registry`].
pub(crate) type BoxedAny = Box<dyn Any + Send + Sync + 'static>;
pub(crate) type RefAny = Ref<dyn Any + Send + Sync + 'static>;
pub(crate) type BoxErr = Box<dyn Error + Send + Sync + 'static>;
pub(crate) type BoxFuture<'a, T> =
    Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;

// `RwLock` types.
pub(crate) type NonAsyncRwLock<T> = parking_lot::RwLock<T>;
pub(crate) type RwLock<T> = ::tokio::sync::RwLock<T>;

// Hashmap types.
pub(crate) type HashMap<K, V> = hashbrown::HashMap<K, V>;

// Cell types.
pub(crate) type OnceCell<T> = ::tokio::sync::OnceCell<T>;
pub(crate) type SingletonCell = ::tokio::sync::OnceCell<RefAny>;

/// Constructor closure for transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Fn() -> T` is provided.
pub trait TransientCtor<T>:
    Fn() -> BoxFuture<'static, T> + Send + Sync + 'static
{
}
impl<T, F> TransientCtor<T> for F where
    F: Fn() -> BoxFuture<'static, T> + Send + Sync + 'static
{
}

/// Constructor closure for transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `AsyncFn(Deps) -> T` is provided.
pub trait TransientCtorDeps<T, D>:
    Fn(D) -> BoxFuture<'static, T> + Send + Sync + 'static
{
}
impl<T, D, F> TransientCtorDeps<T, D> for F
where
    F: Fn(D) -> BoxFuture<'static, T> + Send + Sync + 'static,
    D: crate::dependency_builder::DepBuilder<T> + 'static,
{
}

/// Constructor closure for *fallible* transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Fn() -> Result<T, Err>` is provided.
pub trait TransientCtorFallible<T>:
    Fn() -> BoxFuture<'static, Result<T, BoxErr>> + Send + Sync + 'static
{
}
impl<T, F> TransientCtorFallible<T> for F where
    F: Fn() -> BoxFuture<'static, Result<T, BoxErr>> + Send + Sync + 'static
{
}

/// Constructor closure for *fallible* transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Fn(Deps) -> Result<T, Err>` is provided.
pub trait TransientCtorFallibleDeps<T, Deps>:
    Fn(Deps) -> BoxFuture<'static, Result<T, BoxErr>> + Send + Sync + 'static
{
}
impl<T, F, Deps> TransientCtorFallibleDeps<T, Deps> for F
where
    F: Fn(Deps) -> BoxFuture<'static, Result<T, BoxErr>>
        + Send
        + Sync
        + 'static,
    Deps: crate::dependency_builder::DepBuilder<T> + 'static,
{
}

/// A generic constructor for singletons.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `FnOnce() -> T` is provided.
pub trait SingletonCtor<T>:
    FnOnce() -> BoxFuture<'static, T> + Send + Sync + 'static
{
}
impl<T, F> SingletonCtor<T> for F where
    F: FnOnce() -> BoxFuture<'static, T> + Send + Sync + 'static
{
}

/// A generic constructor for *fallible* singletons.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `AsyncFnOnce() -> Result<T, Err>` is provided.
pub trait SingletonCtorFallible<T>:
    FnOnce() -> BoxFuture<'static, Result<T, BoxErr>> + Send + Sync + 'static
{
}
impl<T, F> SingletonCtorFallible<T> for F where
    F: FnOnce() -> BoxFuture<'static, Result<T, BoxErr>>
        + Send
        + Sync
        + 'static
{
}

/// A generic constructor for singletons with dependencies.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Once(Deps) -> T` is provided.
pub trait SingletonCtorDeps<T, Deps>:
    FnOnce(Deps) -> BoxFuture<'static, T> + Send + Sync + 'static
{
}
impl<T, F, Deps> SingletonCtorDeps<T, Deps> for F
where
    F: FnOnce(Deps) -> BoxFuture<'static, T> + Send + Sync + 'static,
    Deps: crate::dependency_builder::DepBuilder<T> + 'static,
{
}

/// A generic constructor for *fallible* singletons with dependencies.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Once(Deps) -> T` is provided.
pub trait SingletonCtorFallibleDeps<T, Deps>:
    FnOnce(Deps) -> BoxFuture<'static, Result<T, BoxErr>> + Send + Sync + 'static
{
}
impl<T, F, Deps> SingletonCtorFallibleDeps<T, Deps> for F
where
    T: Send + Sync + 'static,
    F: FnOnce(Deps) -> BoxFuture<'static, Result<T, BoxErr>>
        + Send
        + Sync
        + 'static,
    Deps: crate::dependency_builder::DepBuilder<T> + 'static,
{
}

/// A generic reference type that's used as the default type for types with
/// the singleton lifetime.
///
/// When the `multithread` feature is set, this defaults to
/// [`std::sync::Arc`]. When the `multithread` feature is **NOT** set,
/// this defaults to [`std::rc::Rc`].
///
/// It's advised to use [`Ref`] instead of the concrete type because it
/// simplifies enabling `multithread` when required.
pub type Ref<T> = std::sync::Arc<T>;

/// A marker trait for all types that can be registered with `Registry::transient`.
///
/// It's automatically implemented for all types that are valid. Generally,
/// those are all types with a `'static` lifetime, that are also `Send`.
pub trait Registerable: Send + Sync + 'static {}

impl<T> Registerable for T where T: Send + Sync + 'static {}

/// A marker trait for all types that can be registered with `Registry::singleton`.
///
/// It's automatically implemented for all types that are valid. Generally,
/// those are all types with a `'static` lifetime, that are also `Send`
/// and `Sync`.
pub trait RegisterableSingleton: Send + Sync + 'static {}

impl<T> RegisterableSingleton for T where T: Send + Sync + 'static {}
