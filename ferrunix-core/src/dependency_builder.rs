//! Implementation of [`DepBuilder`] for tuples to be used with
//! [`Registry::with_deps`].
#![allow(clippy::missing_errors_doc, clippy::manual_async_fn)]

use std::any::TypeId;

use crate::error::ResolveError;
use crate::types::Registerable;
use crate::Registry;

/// Required for sealing the trait. *Must not be public*.
pub(crate) mod private {
    /// This token is used to seal the [`DepBuilder`] trait from downstream
    /// crates.
    #[allow(missing_debug_implementations)]
    #[derive(Clone, Copy)]
    pub struct SealToken;
}

/// The [`DepBuilder`] trait is the key to specify a variable amount of
/// dependencies in the [`Registry::with_deps`] call from [`Registry`].
///
/// The trait is implemented by the `DepBuilderImpl!` macro for 0-ary, to 10-ary
/// tuples (e.g., `(T1,)`, `(T1, T2)`, etc.), which allows these tuples to be
/// passed as a single type parameter into [`Registry::with_deps`].
///
/// This trait is sealed, meaning it cannot be implemented or called by any
/// downstream crates.
pub trait DepBuilder<R> {
    /// When implemented, this should validate that all dependencies which are
    /// part of `Self` exist to construct the type `R`. If the dependencies
    /// cannot be fulfilled, `None` must be returned.
    ///
    /// If the dependencies can be fulfilled, they must be constructed as an
    /// N-ary tuple (same length and types as `Self`) and passed as the
    /// argument to `ctor`. `ctor` is a user provided constructor for the
    /// type `R`.
    ///
    /// An implementation for tuples is provided by `DepBuilderImpl!`.
    ///
    /// It's advised to avoid *manually* implementing `build`.
    #[cfg(not(feature = "tokio"))]
    fn build(
        registry: &Registry,
        ctor: &(dyn crate::types::TransientCtorFallibleDeps<R, Self>),
        _: private::SealToken,
    ) -> Result<R, ResolveError>
    where
        R: Sized;

    /// Similar to [`DepBuilder::build`], except that it takes a boxed `dyn FnOnce` closure.
    /// This constructor is used for singletons.
    ///
    /// Similarly to `build`, this is also implemented by `DepBuilderImpl!`.
    ///
    /// It's advised to avoid *manually* implementing `build`.
    #[cfg(not(feature = "tokio"))]
    fn build_once(
        registry: &Registry,
        ctor: Box<dyn crate::types::SingletonCtorFallibleDeps<R, Self>>,
        _: private::SealToken,
    ) -> Result<R, ResolveError>
    where
        R: Sized,
        Self: Sized;

    /// When implemented, this should validate that all dependencies which are
    /// part of `Self` exist to construct the type `R`. If the dependencies
    /// cannot be fulfilled, `None` must be returned.
    ///
    /// If the dependencies can be fulfilled, they must be constructed as an
    /// N-ary tuple (same length and types as `Self`) and passed as the
    /// argument to `ctor`. `ctor` is a user provided constructor for the
    /// type `R`.
    ///
    /// An implementation for tuples is provided by `DepBuilderImpl!`.
    ///
    /// It's advised to avoid *manually* implementing `build`.
    #[cfg(feature = "tokio")]
    fn build(
        registry: &Registry,
        ctor: &(dyn crate::types::TransientCtorFallibleDeps<R, Self> + Send),
        _: private::SealToken,
    ) -> impl std::future::Future<Output = Result<R, ResolveError>> + Send + Sync
    where
        R: Sized;

    /// Similar to [`DepBuilder::build`], except that it takes a boxed `dyn FnOnce` closure.
    /// This constructor is used for singletons.
    ///
    /// Similarly to `build`, this is also implemented by `DepBuilderImpl!`.
    ///
    /// It's advised to avoid *manually* implementing `build`.
    #[cfg(feature = "tokio")]
    fn build_once(
        registry: &Registry,
        ctor: Box<dyn crate::types::SingletonCtorFallibleDeps<R, Self> + Send>,
        _: private::SealToken,
    ) -> impl std::future::Future<Output = Result<R, ResolveError>> + Send + Sync
    where
        R: Sized,
        Self: Sized;

    /// Constructs a [`Vec`] of [`std::any::TypeId`]s from the types in `Self`.
    /// The resulting vector must have the same length as `Self`.
    ///
    /// An implementation for tuples is provided by `DepBuilderImpl!`.
    ///
    /// We advise against *manually* implementing `as_typeids`.
    fn as_typeids(_: private::SealToken) -> Vec<(TypeId, &'static str)>;
}

impl<R> DepBuilder<R> for ()
where
    R: Registerable,
{
    #[cfg(not(feature = "tokio"))]
    fn build(
        _registry: &Registry,
        ctor: &(dyn crate::types::TransientCtorFallibleDeps<R, Self>),
        _: private::SealToken,
    ) -> Result<R, ResolveError> {
        (ctor)(()).map_err(ResolveError::Ctor)
    }

    #[cfg(not(feature = "tokio"))]
    fn build_once(
        _registry: &Registry,
        ctor: Box<dyn crate::types::SingletonCtorFallibleDeps<R, Self>>,
        _: private::SealToken,
    ) -> Result<R, ResolveError>
    where
        R: Sized,
        Self: Sized,
    {
        ctor(()).map_err(ResolveError::Ctor)
    }

    #[cfg(feature = "tokio")]
    fn build(
        _registry: &Registry,
        ctor: &(dyn crate::types::TransientCtorFallibleDeps<R, Self> + Send),
        _: private::SealToken,
    ) -> impl std::future::Future<Output = Result<R, ResolveError>> + Send + Sync
    {
        async move {
            (ctor)(()).await.map_err(ResolveError::Ctor)
        }
    }

    #[cfg(feature = "tokio")]
    fn build_once(
        _registry: &Registry,
        ctor: Box<dyn crate::types::SingletonCtorFallibleDeps<R, Self> + Send>,
        _: private::SealToken,
    ) -> impl std::future::Future<Output = Result<R, ResolveError>> + Send + Sync
    where
        R: Sized,
        Self: Sized,
    {
        async move {
            (ctor)(()).await.map_err(ResolveError::Ctor)
        }
    }

    fn as_typeids(_: private::SealToken) -> Vec<(TypeId, &'static str)> {
        Vec::new()
    }
}

/// Generates the implementation for [`DepBuilder`].
macro_rules! DepBuilderImpl {
    ($n:expr, { $($ts:ident),+ }) => {
        impl<R, $($ts,)*> $crate::dependency_builder::DepBuilder<R> for ($($ts,)*)
        where
            R: $crate::types::Registerable,
            $($ts: $crate::dependencies::Dep + Send,)*
        {
            #[cfg(not(feature = "tokio"))]
            fn build(
                registry: &$crate::registry::Registry,
                ctor: &(dyn crate::types::TransientCtorFallibleDeps<R, Self>),
                _: private::SealToken,
            ) -> Result<R, ResolveError>
            {
                registry.validate::<R>()?;

                let deps = (
                    $(
                        <$ts>::new(registry),
                    )*
                );

                (ctor)(deps).map_err(ResolveError::Ctor)
            }

            #[cfg(not(feature = "tokio"))]
            fn build_once(
                registry: &$crate::registry::Registry,
                ctor: Box<dyn crate::types::SingletonCtorFallibleDeps<R, Self>>,
                _: private::SealToken,
                ) -> Result<R, ResolveError>
                where
                    R: Sized,
                    Self: Sized
            {
                registry.validate::<R>()?;

                let deps = (
                    $(
                        <$ts>::new(registry),
                    )*
                );

                ctor(deps).map_err(ResolveError::Ctor)
            }

            #[cfg(feature = "tokio")]
            fn build(
                registry: &Registry,
                ctor: &(dyn crate::types::TransientCtorFallibleDeps<R, Self> + Send),
                _: private::SealToken,
            ) -> impl std::future::Future<Output = Result<R, ResolveError>> + Send + Sync
            {
                async move {
                    registry.validate::<R>()?;

                    let deps = (
                        $(
                            <$ts>::new(registry).await,
                        )*
                    );

                    (ctor)(deps).await.map_err(ResolveError::Ctor)
                }
            }

            #[cfg(feature = "tokio")]
            fn build_once(
                registry: &Registry,
                ctor: Box<dyn crate::types::SingletonCtorFallibleDeps<R, Self> + Send>,
                _: private::SealToken,
            ) -> impl std::future::Future<Output = Result<R, ResolveError>> + Send + Sync
            {
                async move {
                    registry.validate::<R>()?;

                    let deps = (
                        $(
                            <$ts>::new(registry).await,
                        )*
                    );

                    (ctor)(deps).await.map_err(ResolveError::Ctor)
                }
            }

            fn as_typeids(_: private::SealToken) -> ::std::vec::Vec<(::std::any::TypeId, &'static str)> {
                ::std::vec![ $((<$ts>::type_id(), ::std::any::type_name::<$ts>()),)* ]
            }
        }
    };
}

DepBuilderImpl!(1, { T1 });
DepBuilderImpl!(2, { T1, T2 });
DepBuilderImpl!(3, { T1, T2, T3 });
DepBuilderImpl!(4, { T1, T2, T3, T4 });
DepBuilderImpl!(5, { T1, T2, T3, T4, T5 });
DepBuilderImpl!(6, { T1, T2, T3, T4, T5, T6 });
DepBuilderImpl!(7, { T1, T2, T3, T4, T5, T6, T8 });
DepBuilderImpl!(8, { T1, T2, T3, T4, T5, T6, T8, T9 });
