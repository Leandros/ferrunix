use std::any::TypeId;

use crate::Registry;

pub(crate) mod private {
    /// This token is used to seal the [`DepBuilder`] trait from downstream crates.
    pub struct SealToken;
}

/// The [`DepBuilder`] trait is the key to specify a variable amount of dependencies in the
/// [`Registry::with_deps`] call from [`Registry`].
///
/// The trait is implemented by the [`DepBuilderImpl`] macro for 0-ary, to 10-ary tuples (e.g.,
/// `(T1,)`, `(T1, T2)`, etc.), which allows these tuples to be passed as a single type parameter
/// into [`Registry::with_deps`].
///
/// This trait is sealed, meaning it cannot be implemented or called by any downstream crates.
pub trait DepBuilder<R> {
    fn build(registry: &Registry, ctor: fn(Self) -> R, _: private::SealToken) -> Option<R>
    where
        R: Sized;

    fn as_typeids(_: private::SealToken) -> Vec<TypeId>;
}

impl<R> DepBuilder<R> for ()
where
    R: Send + Sync + 'static,
{
    fn build(_registry: &Registry, ctor: fn(Self) -> R, _: private::SealToken) -> Option<R> {
        Some(ctor(()))
    }

    fn as_typeids(_: private::SealToken) -> Vec<TypeId> {
        Vec::new()
    }
}

macro_rules! DepBuilderImpl {
    ($n:expr, { $($ts:ident),+ }) => {
        impl<R, $($ts,)*> $crate::dependency_builder::DepBuilder<R> for ($($ts,)*)
        where
            R: Send + Sync + 'static,
            $($ts: $crate::dependencies::Dep + Send + Sync + 'static,)*
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
