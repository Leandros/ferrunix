use std::any::TypeId;

use crate::Registry;

pub trait DepBuilder<R> {
    fn build(registry: &Registry, ctor: fn(Self) -> R) -> R;
    fn as_typeids(&self) -> Vec<TypeId>;
}

impl<R: Send + Sync + 'static> DepBuilder<R> for () {
    fn build(_registry: &Registry, ctor: fn(Self) -> R) -> R {
        ctor(())
    }

    fn as_typeids(&self) -> Vec<TypeId> {
        Vec::new()
    }
}

macro_rules! DepBuilderImpl {
    ($n:expr, { $($ts:ident),+ }, $ty:ty) => {
        impl<R: Send + Sync + 'static, $($ts: crate::dependencies::Dep + Send + Sync + 'static,)*> crate::dependency_builder::DepBuilder<R> for $ty {
            fn build(registry: &crate::registry::Registry, ctor: fn(Self) -> R) -> R {
                ctor(
                    (
                        $(
                            <$ts>::new(registry),
                        )*
                    )
                )
            }

            fn as_typeids(&self) -> ::std::vec::Vec<::std::any::TypeId> {
                ::std::vec![ $(TypeId::of::<$ts>(),)* ]
            }
        }
    };
}

DepBuilderImpl!(1, { T1 }, (T1,));
DepBuilderImpl!(2, { T1, T2 }, (T1, T2));
DepBuilderImpl!(3, { T1, T2, T3 }, (T1, T2, T3));
