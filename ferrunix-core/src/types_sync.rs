//! Various type aliases for the multithreaded implementation.
use std::any::Any;
use std::error::Error;

use crate::object_builder::{SingletonGetter, TransientBuilder};

pub(crate) type OnceCell<T> = once_cell::sync::OnceCell<T>;

// `RwLock` types.
pub(crate) type RwLock<T> = parking_lot::RwLock<T>;
pub(crate) type NonAsyncRwLock<T> = parking_lot::RwLock<T>;
pub(crate) type MappedRwLockReadGuard<'a, T> =
    parking_lot::MappedRwLockReadGuard<'a, T>;
pub(crate) type MappedRwLockWriteGuard<'a, T> =
    parking_lot::MappedRwLockWriteGuard<'a, T>;
pub(crate) type RwLockReadGuard<'a, T> = parking_lot::RwLockReadGuard<'a, T>;
pub(crate) type RwLockWriteGuard<'a, T> = parking_lot::RwLockWriteGuard<'a, T>;

// Hashmap types.
pub(crate) type HashMap<K, V> = hashbrown::HashMap<K, V>;

// Alias types used in [`Registry`].
pub(crate) type BoxedAny = Box<dyn Any>;
pub(crate) type RefAny = Ref<dyn Any + Send + Sync + 'static>;
pub(crate) type SingletonCell = OnceCell<RefAny>;
pub(crate) type BoxedTransientBuilder =
    Box<dyn TransientBuilder + Send + Sync + 'static>;
pub(crate) type BoxedSingletonGetter =
    Box<dyn SingletonGetter + Send + Sync + 'static>;
pub(crate) type BoxErr = Box<dyn Error + Send + Sync + 'static>;

/// Constructor closure for transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Fn() -> T` is provided.
pub trait TransientCtor<T>: Fn() -> T + Send + Sync + 'static {
    /// Calls the constructor. Equivalent to `Ok((self)())`.
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, F> TransientCtor<T> for F
where
    F: Fn() -> T + Send + Sync + 'static,
{
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr> {
        Ok((self)())
    }
}

/// Constructor closure for transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Fn(Deps) -> T` is provided.
pub trait TransientCtorDeps<T, D>: Fn(D) -> T + Send + Sync + 'static {
    /// Calls the constructor. Equivalent to `(self)()`.
    fn call(self, deps: D, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, D, F> TransientCtorDeps<T, D> for F
where
    T: 'static,
    F: Fn(D) -> T + Send + Sync + 'static,
    D: crate::dependency_builder::DepBuilder<T> + 'static,
{
    fn call(self, deps: D, _: super::private::SealToken) -> Result<T, BoxErr> {
        Ok((self)(deps))
    }
}

/// Constructor closure for *fallible* transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Fn() -> Result<T, Err>` is provided.
pub trait TransientCtorFallible<T>:
    Fn() -> Result<T, BoxErr> + Send + Sync + 'static
{
    /// Calls the constructor.
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, F> TransientCtorFallible<T> for F
where
    T: 'static,
    F: Fn() -> Result<T, BoxErr> + Send + Sync + 'static,
{
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr> {
        (self)()
    }
}

/// Constructor closure for *fallible* transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Fn(Deps) -> Result<T, Err>` is provided.
pub trait TransientCtorFallibleDeps<T, Deps>:
    Fn(Deps) -> Result<T, BoxErr> + Send + Sync + 'static
{
    /// Call the constructor.
    fn call(
        self,
        deps: Deps,
        _: super::private::SealToken,
    ) -> Result<T, BoxErr>;
}
impl<T, F, Deps> TransientCtorFallibleDeps<T, Deps> for F
where
    T: 'static,
    F: Fn(Deps) -> Result<T, BoxErr> + Send + Sync + 'static,
    Deps: crate::dependency_builder::DepBuilder<T> + 'static,
{
    fn call(
        self,
        deps: Deps,
        _: super::private::SealToken,
    ) -> Result<T, BoxErr> {
        (self)(deps)
    }
}

/// A generic constructor for singletons.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `FnOnce() -> T` is provided.
pub trait SingletonCtor<T>: FnOnce() -> T + Send + Sync + 'static {
    /// Calls the construcor.
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, F> SingletonCtor<T> for F
where
    F: FnOnce() -> T + Send + Sync + 'static,
{
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr> {
        Ok((self)())
    }
}

/// A generic constructor for *fallible* singletons.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `FnOnce() -> Result<T, Err>` is provided.
pub trait SingletonCtorFallible<T>:
    FnOnce() -> Result<T, BoxErr> + Send + Sync + 'static
{
    /// Calls the construcor.
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, F> SingletonCtorFallible<T> for F
where
    T: Send + Sync + 'static,
    F: FnOnce() -> Result<T, BoxErr> + Send + Sync + 'static,
{
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr> {
        (self)()
    }
}

/// A generic constructor for singletons with dependencies.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `FnOnce(Deps) -> T` is provided.
pub trait SingletonCtorDeps<T, Deps>:
    FnOnce(Deps) -> T + Send + Sync + 'static
{
    /// Calls the construcor.
    fn call(
        self,
        deps: Deps,
        _: super::private::SealToken,
    ) -> Result<T, BoxErr>;
}
impl<T, F, Deps> SingletonCtorDeps<T, Deps> for F
where
    F: FnOnce(Deps) -> T + Send + Sync + 'static,
    Deps: crate::dependency_builder::DepBuilder<T> + 'static,
{
    fn call(
        self,
        deps: Deps,
        _: super::private::SealToken,
    ) -> Result<T, BoxErr> {
        Ok((self)(deps))
    }
}

/// A generic constructor for *fallible* singletons with dependencies.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `FnOnce(Deps) -> T` is provided.
pub trait SingletonCtorFallibleDeps<T, Deps>:
    FnOnce(Deps) -> Result<T, BoxErr> + Send + Sync + 'static
{
    /// Calls the construcor.
    fn call(
        self,
        deps: Deps,
        _: super::private::SealToken,
    ) -> Result<T, BoxErr>;
}
impl<T, F, Deps> SingletonCtorFallibleDeps<T, Deps> for F
where
    T: Send + Sync + 'static,
    F: FnOnce(Deps) -> Result<T, BoxErr> + Send + Sync + 'static,
    Deps: crate::dependency_builder::DepBuilder<T> + 'static,
{
    fn call(
        self,
        deps: Deps,
        _: super::private::SealToken,
    ) -> Result<T, BoxErr> {
        (self)(deps)
    }
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

/// A generic *weak* reference type that's used as the return type for newly created child
/// registries.
pub type RefWeak<T> = std::sync::Weak<T>;

/// A marker trait for all types that can be registered with `Registry::transient`.
///
/// It's automatically implemented for all types that are valid. Generally,
/// those are all types with a `'static` lifetime.
pub trait Registerable: 'static {}

impl<T> Registerable for T where T: 'static {}

/// A marker trait for all types that can be registered with `Registry::singleton`.
///
/// It's automatically implemented for all types that are valid. Generally,
/// those are all types with a `'static` lifetime, that are also `Send`
/// and `Sync`.
pub trait RegisterableSingleton: Send + Sync + 'static {}

impl<T> RegisterableSingleton for T where T: Send + Sync + 'static {}
