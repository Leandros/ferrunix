//! All possible dependencies for injected types.
//!
//! The following dependency types are available, as of right now:
//!   * [`Transient`]: Dependencies that are created from scratch when
//!     requested.
//!   * [`Singleton`]: Dependencies that are created once for every registry.
//!
//! All dependency types implement the [`Dep`] trait, and can get access to the
//! inner type via `.get`.
//!
//! # Examples
//! ```ignore,no_run
//! use ferrunix_core::{Registry, Singleton, Transient};
//!
//! struct Template {
//!     template: &'static str,
//! }
//!
//! let registry = Registry::empty();
//! registry.transient(|| 1_u8);
//! registry.singleton(|| Template { template: "u8 is:" });
//!
//! registry
//!     .with_deps::<_, (Transient<u8>, Singleton<Template>)>()
//!     .transient(|(num, template)| {
//!         let num = num.get(); // grab the inner `u8`.
//!         format!("{} {num}", template.template) // you can also use the `Deref` impl.
//!     });
//!
//! let s = registry.transient::<String>().unwrap();
//! assert_eq!(s, "u8 is: 1".to_string());
//! ```
#![allow(clippy::manual_async_fn)]

use std::any::TypeId;

use crate::types::{Registerable, RegisterableSingleton};
use crate::{types::Ref, Registry};

/// Required for sealing the `Dep` trait. *Must not be public*.
mod private {
    /// Private trait for sealing [`Dep`].
    pub trait Sealed {}
}

/// Trait to specify a dependency. Every possible dependency type is
/// implementing this trait.
///
/// Current implementors:
///   * [`Transient`]
///   * [`Singleton`]
///
/// This trait is sealed, it cannot be implemented outside of this crate.
pub trait Dep: Registerable + private::Sealed {
    /// Looks up the dependency in `registry`, and constructs a new [`Dep`].
    ///
    /// This function is allowed to panic, if the type isn't registered.
    #[cfg(not(feature = "tokio"))]
    fn new(registry: &Registry) -> Self;

    /// Looks up the dependency in `registry`, and constructs a new [`Dep`].
    ///
    /// This function is allowed to panic, if the type isn't registered.
    #[cfg(feature = "tokio")]
    fn new(
        registry: &Registry,
    ) -> impl std::future::Future<Output = Self> + Send + Sync
    where
        Self: Sized;

    /// Returns [`std::any::TypeId`] of the dependency type.
    fn type_id() -> TypeId;
}

/// Transient dependencies.
///
/// This dependency is created from scratch every time it's requested.
#[repr(transparent)]
pub struct Transient<T> {
    /// The resolved type.
    inner: T,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Transient<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Transient")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: Registerable> std::ops::Deref for Transient<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Registerable> std::ops::DerefMut for Transient<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Registerable> Transient<T> {
    /// Access the inner `T`.
    #[must_use]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub fn get(self) -> T {
        self.inner
    }
}

// Required for implementing `Dep`.
impl<T> private::Sealed for Transient<T> {}

impl<T: Registerable> Dep for Transient<T> {
    /// Create a new [`Transient`].
    ///
    /// # Panic
    /// This function panics if the `T` isn't registered.
    #[cfg(not(feature = "tokio"))]
    fn new(registry: &Registry) -> Self {
        Self {
            inner: registry.transient::<T>().expect(
                "transient dependency must only be constructed if it's \
                 fulfillable",
            ),
        }
    }

    /// Create a new [`Transient`], asynchronously.
    ///
    /// # Panic
    /// This function panics if the `T` isn't registered.
    #[cfg(feature = "tokio")]
    fn new(
        registry: &Registry,
    ) -> impl std::future::Future<Output = Self> + Send + Sync {
        async move {
            Self {
                inner: registry.transient::<T>().await.expect(
                    "transient dependency must only be constructed if it's \
                 fulfillable",
                ),
            }
        }
    }

    /// Returns [`std::any::TypeId`] of the inner type `T`.
    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

/// Singleton dependencies.
///
/// This dependency is created only once for the specified registry. It's
/// created lazily on-demand.
#[repr(transparent)]
pub struct Singleton<T> {
    /// The resolved type.
    inner: Ref<T>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Singleton<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Singleton")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: RegisterableSingleton> From<Singleton<T>> for Ref<T> {
    fn from(value: Singleton<T>) -> Self {
        value.inner
    }
}

impl<T: RegisterableSingleton> std::ops::Deref for Singleton<T> {
    type Target = Ref<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: RegisterableSingleton> std::ops::DerefMut for Singleton<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: RegisterableSingleton> Singleton<T> {
    /// Access the inner dependency, returns a ref-counted object.
    #[must_use]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub fn get(self) -> Ref<T> {
        self.inner
    }
}

// Required for implementing `Dep`.
impl<T> private::Sealed for Singleton<T> {}

impl<T: RegisterableSingleton> Dep for Singleton<T> {
    /// Create a new [`Singleton`].
    ///
    /// # Panic
    /// This function panics if the `T` isn't registered.
    #[cfg(not(feature = "tokio"))]
    fn new(registry: &Registry) -> Self {
        Self {
            inner: registry.singleton::<T>().expect(
                "singleton dependency must only be constructed if it's \
                 fulfillable",
            ),
        }
    }

    /// Create a new [`Singleton`], asynchronously.
    ///
    /// # Panic
    /// This function panics if the `T` isn't registered.
    #[cfg(feature = "tokio")]
    fn new(
        registry: &Registry,
    ) -> impl std::future::Future<Output = Self> + Send + Sync {
        async move {
            Self {
                inner: registry.singleton::<T>().await.expect(
                    "singleton dependency must only be constructed if it's \
                 fulfillable",
                ),
            }
        }
    }

    /// Returns [`std::any::TypeId`] of the inner type `T`.
    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}
