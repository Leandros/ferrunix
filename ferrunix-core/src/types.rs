//! Type aliases.
#![allow(
    clippy::single_char_lifetime_names,
    clippy::missing_docs_in_private_items,
    dead_code
)]

/// Types that are enabled when the `multithread` feature is set.
#[cfg(feature = "multithread")]
mod sync {
    use std::any::Any;

    use crate::Registry;

    pub(crate) type OnceCell<T> = once_cell::sync::OnceCell<T>;

    // `RwLock` types.
    pub(crate) type RwLock<T> = parking_lot::RwLock<T>;
    pub(crate) type MappedRwLockReadGuard<'a, T> =
        parking_lot::MappedRwLockReadGuard<'a, T>;
    pub(crate) type MappedRwLockWriteGuard<'a, T> =
        parking_lot::MappedRwLockWriteGuard<'a, T>;
    pub(crate) type RwLockReadGuard<'a, T> =
        parking_lot::RwLockReadGuard<'a, T>;
    pub(crate) type RwLockWriteGuard<'a, T> =
        parking_lot::RwLockWriteGuard<'a, T>;

    // Hashmap types.
    pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V>;

    // Alias types used in [`Registry`].
    pub(crate) type BoxedAny = Box<dyn Any + Send + Sync>;
    pub(crate) type RefAny = Ref<dyn Any + Send + Sync>;
    pub(crate) type BoxedCtor =
        Box<dyn Fn(&Registry) -> Option<BoxedAny> + Send + Sync>;
    pub(crate) type SingletonCell = OnceCell<RefAny>;
    pub(crate) type BoxedSingletonGetter =
        Box<dyn Fn(&Registry, &SingletonCell) -> Option<RefAny> + Send + Sync>;
    pub(crate) type Validator = Box<dyn Fn(&Registry) -> bool + Send + Sync>;

    /// A generic reference type that's used as the default type for types with the singleton
    /// lifetime.
    ///
    /// When the `multithread` feature is set, this defaults to [`std::sync::Arc`].
    /// When the `multithread` feature is **NOT** set, this defaults to [`std::rc::Rc`].
    ///
    /// It's advised to use [`Ref`] instead of the concrete type because it simplifies enabling
    /// `multithread` when required.
    pub type Ref<T> = std::sync::Arc<T>;

    /// A marker trait for all types that can be registered with [`Registry`].
    ///
    /// It's automatically implemented for all types that are valid. Generally, those are all types
    /// with a `'static` lifetime, that are also `Send` and `Sync`.
    pub trait Registerable: Send + Sync + 'static {}

    impl<T> Registerable for T where T: Send + Sync + 'static {}
}

/// Types that are enabled when the `multithread` feature is **NOT** set.
#[cfg(not(feature = "multithread"))]
mod unsync {
    use std::any::Any;

    use crate::Registry;

    pub(crate) type OnceCell<T> = once_cell::unsync::OnceCell<T>;

    // `RwLock` types.
    pub(crate) type RwLock<T> = RwLockLike<T>;
    pub(crate) type MappedRwLockReadGuard<'a, T> = std::cell::Ref<'a, T>;
    pub(crate) type MappedRwLockWriteGuard<'a, T> = std::cell::RefMut<'a, T>;
    pub(crate) type RwLockReadGuard<'a, T> = std::cell::Ref<'a, T>;
    pub(crate) type RwLockWriteGuard<'a, T> = std::cell::RefMut<'a, T>;

    /// Replacement type for `parking_lot::RwLock` that's single-threaded.
    pub(crate) struct RwLockLike<T> {
        /// We're wrapping a [`std::cell::RefCell`], and exposing an `RwLock`-like
        /// API, so that we can configure what we want to use.
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
    pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V>;

    // Alias types used in [`Registry`].
    pub(crate) type BoxedAny = Box<dyn Any>;
    pub(crate) type RefAny = Ref<dyn Any>;
    pub(crate) type BoxedCtor = Box<dyn Fn(&Registry) -> Option<BoxedAny>>;
    pub(crate) type SingletonCell = OnceCell<RefAny>;
    pub(crate) type BoxedSingletonGetter =
        Box<dyn Fn(&Registry, &SingletonCell) -> Option<RefAny>>;
    pub(crate) type Validator = Box<dyn Fn(&Registry) -> bool>;

    /// A generic reference type that's used as the default type for types with the singleton
    /// lifetime.
    ///
    /// When the `multithread` feature is **NOT** set, this defaults to [`std::rc::Rc`].
    /// When the `multithread` feature is set, this defaults to [`std::sync::Arc`].
    ///
    /// It's advised to use [`Ref`] instead of the concrete type because it simplifies enabling
    /// `multithread` when required.
    pub type Ref<T> = std::rc::Rc<T>;

    /// A marker trait for all types that can be registered with [`Registry`].
    ///
    /// It's automatically implemented for all types that are valid. Generally, those are all types
    /// with a `'static` lifetime.
    pub trait Registerable: 'static {}

    impl<T> Registerable for T where T: 'static {}
}

#[cfg(feature = "multithread")]
pub use sync::*;

#[cfg(not(feature = "multithread"))]
pub use unsync::*;
