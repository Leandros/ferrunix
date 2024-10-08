use std::any::TypeId;

use crate::Registry;

pub trait DepBuilder<R> {
    fn build(registry: &Registry, ctor: fn(Self) -> R) -> Option<R>
    where
        R: Sized;
    fn as_typeids() -> Vec<TypeId>;
}

impl<R> DepBuilder<R> for ()
where
    R: Send + Sync + 'static,
{
    fn build(_registry: &Registry, ctor: fn(Self) -> R) -> Option<R> {
        Some(ctor(()))
    }

    fn as_typeids() -> Vec<TypeId> {
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
            fn build(registry: &$crate::registry::Registry, ctor: fn(Self) -> R) -> Option<R> {
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

DepBuilderImpl!(1, { T1 });
DepBuilderImpl!(2, { T1, T2 });
DepBuilderImpl!(3, { T1, T2, T3 });
DepBuilderImpl!(4, { T1, T2, T3, T4 });
DepBuilderImpl!(5, { T1, T2, T3, T4, T5 });
DepBuilderImpl!(6, { T1, T2, T3, T4, T5, T6 });
DepBuilderImpl!(7, { T1, T2, T3, T4, T5, T6, T8 });
DepBuilderImpl!(8, { T1, T2, T3, T4, T5, T6, T8, T9 });
