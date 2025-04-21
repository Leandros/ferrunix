//! Various type aliases for the single-threaded implementation.

use std::any::Any;
use std::error::Error;

use crate::object_builder::{SingletonGetter, TransientBuilder};

pub(crate) type OnceCell<T> = once_cell::unsync::OnceCell<T>;

// `RwLock` types.
pub(crate) type RwLock<T> = RwLockLike<T>;
pub(crate) type NonAsyncRwLock<T> = RwLockLike<T>;
pub(crate) type MappedRwLockReadGuard<'a, T> = std::cell::Ref<'a, T>;
pub(crate) type MappedRwLockWriteGuard<'a, T> = std::cell::RefMut<'a, T>;
pub(crate) type RwLockReadGuard<'a, T> = std::cell::Ref<'a, T>;
pub(crate) type RwLockWriteGuard<'a, T> = std::cell::RefMut<'a, T>;

/// Replacement type for `parking_lot::RwLock` that's single-threaded.
pub(crate) struct RwLockLike<T> {
    /// We're wrapping a [`std::cell::RefCell`], and exposing an
    /// `RwLock`-like API, so that we can configure what we want to
    /// use.
    inner: std::cell::RefCell<T>,
}

impl<T> RwLockLike<T> {
    pub(crate) const fn new(value: T) -> Self {
        Self {
            inner: std::cell::RefCell::new(value),
        }
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.borrow()
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.inner.borrow_mut()
    }
}

// Hashmap types.
pub(crate) type HashMap<K, V> = hashbrown::HashMap<K, V>;

// Alias types used in [`Registry`].
pub(crate) type BoxedAny = Box<dyn Any>;
pub(crate) type RefAny = Ref<dyn Any>;
pub(crate) type SingletonCell = OnceCell<RefAny>;
pub(crate) type BoxedTransientBuilder = Box<dyn TransientBuilder>;
pub(crate) type BoxedSingletonGetter = Box<dyn SingletonGetter>;
pub(crate) type BoxErr = Box<dyn Error>;

/// Constructor closure for transients.
///
/// This is a marker trait to identify all valid constructors usable by singletons.
/// It's not implementable by other crates.
///
/// A blanket implementation for `Fn() -> T` is provided.
pub trait TransientCtor<T>: Fn() -> T + 'static {
    /// Calls the constructor. Equivalent to `Ok((self)())`.
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, F> TransientCtor<T> for F
where
    F: Fn() -> T + 'static,
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
pub trait TransientCtorDeps<T, D>: Fn(D) -> T + 'static {
    /// Calls the constructor. Equivalent to `(self)()`.
    fn call(self, deps: D, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, D, F> TransientCtorDeps<T, D> for F
where
    T: 'static,
    F: Fn(D) -> T + 'static,
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
pub trait TransientCtorFallible<T>: Fn() -> Result<T, BoxErr> {
    /// Calls the constructor.
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, F> TransientCtorFallible<T> for F
where
    F: Fn() -> Result<T, BoxErr>,
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
    Fn(Deps) -> Result<T, BoxErr>
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
    F: Fn(Deps) -> Result<T, BoxErr>,
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
pub trait SingletonCtor<T>: FnOnce() -> T + 'static {
    /// Calls the construcor.
    fn call(self, _: super::private::SealToken) -> Result<T, BoxErr>;
}
impl<T, F> SingletonCtor<T> for F
where
    F: FnOnce() -> T + 'static,
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
pub trait SingletonCtorDeps<T, Deps>: FnOnce(Deps) -> T + 'static {
    /// Calls the construcor.
    fn call(
        self,
        deps: Deps,
        _: super::private::SealToken,
    ) -> Result<T, BoxErr>;
}
impl<T, F, Deps> SingletonCtorDeps<T, Deps> for F
where
    F: FnOnce(Deps) -> T + 'static,
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
    FnOnce(Deps) -> Result<T, BoxErr> + 'static
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
    T: 'static,
    F: FnOnce(Deps) -> Result<T, BoxErr> + 'static,
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
/// When the `multithread` feature is **NOT** set, this defaults to
/// [`std::rc::Rc`]. When the `multithread` feature is set, this
/// defaults to [`std::sync::Arc`].
///
/// It's advised to use [`Ref`] instead of the concrete type because it
/// simplifies enabling `multithread` when required.
pub type Ref<T> = std::rc::Rc<T>;

/// A marker trait for all types that can be registered with `Registry::transient`.
///
/// It's automatically implemented for all types that are valid. Generally,
/// those are all types with a `'static` lifetime.
pub trait Registerable: 'static {}

impl<T> Registerable for T where T: 'static {}

/// A marker trait for all types that can be registered with `Registry::singleton`.
///
/// It's automatically implemented for all types that are valid. Generally,
/// those are all types with a `'static` lifetime.
pub trait RegisterableSingleton: 'static {}

impl<T> RegisterableSingleton for T where T: 'static {}
