//! Type aliases.
#![allow(
    clippy::single_char_lifetime_names,
    clippy::missing_docs_in_private_items,
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
mod sync {
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
    pub(crate) type RwLockReadGuard<'a, T> =
        parking_lot::RwLockReadGuard<'a, T>;
    pub(crate) type RwLockWriteGuard<'a, T> =
        parking_lot::RwLockWriteGuard<'a, T>;

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

    /// A generic constructor for singletons.
    ///
    /// This is a marker trait to identify all valid constructors usable by singletons.
    /// It's not implementable by other crates.
    ///
    /// A blanket implementation for `FnOnce() -> T` is provided.
    pub trait SingletonCtor<T>: FnOnce() -> T + Send + Sync + 'static {
        /// Calls the construcor.
        fn call(self, _: super::private::SealToken) -> T;
    }

    impl<T, F> SingletonCtor<T> for F
    where
        F: FnOnce() -> T + Send + Sync + 'static,
    {
        fn call(self, _: super::private::SealToken) -> T {
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
        fn call(self, deps: Deps, _: super::private::SealToken) -> T;
    }

    #[cfg(not(feature = "tokio"))]
    impl<T, F, Deps> SingletonCtorDeps<T, Deps> for F
    where
        F: FnOnce(Deps) -> T + Send + Sync + 'static,
        Deps: crate::dependency_builder::DepBuilder<T> + 'static,
    {
        fn call(self, deps: Deps, _: super::private::SealToken) -> T {
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
}

/// Types that are enabled when the `multithread` feature is **NOT** set.
#[cfg(all(not(feature = "multithread"), not(feature = "tokio")))]
mod unsync {
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

    /// TODO: Docs
    pub trait TransientCtor<T>: Fn() -> T + 'static {
        /// TODO: Seal trait
        fn call(self) -> Result<T, BoxErr>;
    }

    impl<T, F> TransientCtor<T> for F
    where
        F: Fn() -> T + 'static,
    {
        fn call(self) -> Result<T, BoxErr> {
            Ok((self)())
        }
    }

    /// TODO: Docs
    pub trait TransientCtorDeps<T, D>: Fn(D) -> T + 'static {
        /// TODO: Seal trait
        fn call(self, deps: D) -> Result<T, BoxErr>;
    }

    impl<T, D, F> TransientCtorDeps<T, D> for F
    where
        T: 'static,
        F: Fn(D) -> T + 'static,
        D: crate::dependency_builder::DepBuilder<T> + 'static,
    {
        fn call(self, deps: D) -> Result<T, BoxErr> {
            Ok((self)(deps))
        }
    }

    /// TODO: Docs
    pub trait TransientCtorFn<T>: Fn() -> Result<T, BoxErr> {
        /// TODO: Seal trait
        fn call(self) -> Result<T, BoxErr>;
    }

    impl<T, F> TransientCtorFn<T> for F
    where
        F: Fn() -> Result<T, BoxErr>,
    {
        fn call(self) -> Result<T, BoxErr> {
            (self)()
        }
    }

    /// TODO: Docs
    pub trait TransientCtorFnDeps<T, Deps>:
        Fn(Deps) -> Result<T, BoxErr>
    {
        /// TODO: Seal trait
        fn call(self, deps: Deps) -> Result<T, BoxErr>;
    }

    impl<T, F, Deps> TransientCtorFnDeps<T, Deps> for F
    where
        F: Fn(Deps) -> Result<T, BoxErr>,
        Deps: crate::dependency_builder::DepBuilder<T> + 'static,
    {
        fn call(self, deps: Deps) -> Result<T, BoxErr> {
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
        fn call(self, _: super::private::SealToken) -> T;
    }

    impl<T, F> SingletonCtor<T> for F
    where
        F: FnOnce() -> T + 'static,
    {
        fn call(self, _: super::private::SealToken) -> T {
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
        fn call(self, deps: Deps, _: super::private::SealToken) -> T;
    }
    impl<T, F, Deps> SingletonCtorDeps<T, Deps> for F
    where
        F: FnOnce(Deps) -> T + 'static,
        Deps: crate::dependency_builder::DepBuilder<T> + 'static,
    {
        fn call(self, deps: Deps, _: super::private::SealToken) -> T {
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
}

#[cfg(feature = "tokio")]
mod tokio_ext {
    use std::any::Any;

    // Alias types used in [`Registry`].
    pub(crate) type BoxedAny = Box<dyn Any + Send>;
    pub(crate) type RefAny = Ref<dyn Any + Send + Sync + 'static>;

    // `RwLock` types.
    pub(crate) type NonAsyncRwLock<T> = parking_lot::RwLock<T>;
    pub(crate) type RwLock<T> = ::tokio::sync::RwLock<T>;

    // Hashmap types.
    pub(crate) type HashMap<K, V> = hashbrown::HashMap<K, V>;

    // Cell types.
    pub(crate) type OnceCell<T> = ::tokio::sync::OnceCell<T>;
    pub(crate) type SingletonCell = ::tokio::sync::OnceCell<RefAny>;

    /// A generic constructor for singletons.
    ///
    /// This is a marker trait to identify all valid constructors usable by singletons.
    /// It's not implementable by other crates.
    ///
    /// A blanket implementation for `FnOnce() -> T` is provided.
    pub trait SingletonCtor<T>:
        FnOnce()
            -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>
        + Send
        + Sync
        + 'static
    {
        /// Calls the construcor.
        fn call(
            self,
            _: super::private::SealToken,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
    }

    impl<T, F> SingletonCtor<T> for F
    where
        F: FnOnce() -> std::pin::Pin<
                Box<dyn std::future::Future<Output = T> + Send>,
            > + Send
            + Sync
            + 'static,
    {
        fn call(
            self,
            _: super::private::SealToken,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>
        {
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
        FnOnce(
            Deps,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>
        + Send
        + Sync
        + 'static
    {
        /// Calls the construcor.
        fn call(
            self,
            deps: Deps,
            _: super::private::SealToken,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
    }

    impl<T, F, Deps> SingletonCtorDeps<T, Deps> for F
    where
        F: FnOnce(
                Deps,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = T> + Send>,
            > + Send
            + Sync
            + 'static,
        Deps: crate::dependency_builder::DepBuilder<T> + Sync + 'static,
    {
        fn call(
            self,
            deps: Deps,
            _: super::private::SealToken,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>
        {
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
}

#[cfg(all(feature = "multithread", not(feature = "tokio")))]
pub use sync::*;

#[cfg(all(not(feature = "multithread"), not(feature = "tokio")))]
pub use unsync::*;

#[cfg(feature = "tokio")]
pub use tokio_ext::*;
