//! Implementation of [`DepBuilder`] for tuples to be used with
//! [`Registry::with_deps`].

use std::any::TypeId;

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
    /// We advise against *manually* implementing `build`.
    fn build(
        registry: &Registry,
        ctor: fn(Self) -> R,
        _: private::SealToken,
    ) -> Option<R>
    where
        R: Sized;

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
    /// We advise against *manually* implementing `build`.
    #[cfg(feature = "tokio")]
    fn build_async(
        registry: &Registry,
        ctor: fn(
            Self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = R> + Send + Sync>,
        >,
        _: private::SealToken,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<R>> + Send + Sync + '_>,
    >
    where
        R: Sized;

    /// Constructs a [`Vec`] of [`std::any::TypeId`]s from the types in `Self`.
    /// The resulting vector must have the same length as `Self`.
    ///
    /// An implementation for tuples is provided by `DepBuilderImpl!`.
    ///
    /// We advise against *manually* implementing `as_typeids`.
    fn as_typeids(_: private::SealToken) -> Vec<TypeId>;
}

impl<R> DepBuilder<R> for ()
where
    R: Registerable,
{
    fn build(
        _registry: &Registry,
        ctor: fn(Self) -> R,
        _: private::SealToken,
    ) -> Option<R> {
        Some(ctor(()))
    }

    #[cfg(feature = "tokio")]
    fn build_async(
        _registry: &Registry,
        ctor: fn(
            Self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = R> + Send + Sync>,
        >,
        _: private::SealToken,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<R>> + Send + Sync + '_>,
    > {
        Box::pin(async move { Some(ctor(()).await) })
    }

    fn as_typeids(_: private::SealToken) -> Vec<TypeId> {
        Vec::new()
    }
}

/// Generates the implementation for [`DepBuilder`].
macro_rules! DepBuilderImpl {
    ($n:expr, { $($ts:ident),+ }) => {
        impl<R, $($ts,)*> $crate::dependency_builder::DepBuilder<R> for ($($ts,)*)
        where
            R: $crate::types::Registerable,
            $($ts: $crate::dependencies::Dep,)*
        {
            fn build(registry: &$crate::registry::Registry, ctor: fn(Self) -> R, _: private::SealToken) -> Option<R> {
                if !registry.validate::<R>() {
                    return None;
                }

                let deps = (
                    $(
                        <$ts>::new(registry),
                    )*
                );

                Some(ctor(deps))
            }

            #[cfg(feature = "tokio")]
            fn build_async(
                registry: &Registry,
                ctor: fn(
                    Self,
                ) -> std::pin::Pin<
                    Box<dyn std::future::Future<Output = R> + Send + Sync>,
                >,
                _: private::SealToken,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Option<R>> + Send + Sync + '_>,
            > {
                if !registry.validate::<R>() {
                    return Box::pin(async move { None });
                }

                Box::pin(async move {
                    let deps = (
                        $(
                            <$ts>::new_async(registry).await,
                        )*
                    );

                    Some(ctor(deps).await)
                })
            }

            fn as_typeids(_: private::SealToken) -> ::std::vec::Vec<::std::any::TypeId> {
                ::std::vec![ $(<$ts>::type_id(),)* ]
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
