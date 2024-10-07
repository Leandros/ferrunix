use std::any::TypeId;

use crate::Registry;

pub trait DepBuilder<R> {
    fn build(registry: &Registry, ctor: fn(Self) -> R) -> Option<R>
    where
        R: Sized;
    fn as_typeids() -> Vec<TypeId>;
}

impl<R: Send + Sync + 'static> DepBuilder<R> for () {
    fn build(_registry: &Registry, ctor: fn(Self) -> R) -> Option<R> {
        Some(ctor(()))
    }

    fn as_typeids() -> Vec<TypeId> {
        Vec::new()
    }
}

macro_rules! DepBuilderImpl {
    ($n:expr, { $($ts:ident),+ }, $ty:ty) => {
        impl<R: Send + Sync + 'static, $($ts: crate::dependencies::Dep + Send + Sync + 'static,)*> crate::dependency_builder::DepBuilder<R> for $ty {
            fn build(registry: &crate::registry::Registry, ctor: fn(Self) -> R) -> Option<R> {
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

            fn as_typeids() -> ::std::vec::Vec<::std::any::TypeId> {
                ::std::vec![ $(<$ts>::type_id(),)* ]
            }
        }
    };
}

DepBuilderImpl!(1, { T1 }, (T1,));
DepBuilderImpl!(2, { T1, T2 }, (T1, T2));
DepBuilderImpl!(3, { T1, T2, T3 }, (T1, T2, T3));
