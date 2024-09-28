use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

pub trait Dep {
    fn new(registry: &Registry) -> Self;
}

pub struct Transient<T> {
    inner: T,
}

impl<T: Send + Sync + 'static> Transient<T> {
    pub fn get(self) -> T {
        self.inner
    }
}

impl<T: Send + Sync + 'static> Dep for Transient<T> {
    fn new(registry: &Registry) -> Self {
        Self {
            inner: registry.get_transient::<T>().unwrap(),
        }
    }
}

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
        impl<R: Send + Sync + 'static, $($ts: Dep + Send + Sync + 'static,)*> DepBuilder<R> for $ty {
            fn build(registry: &Registry, ctor: fn(Self) -> R) -> R {
                ctor(
                    (
                        $(
                            <$ts>::new(registry),
                        )*
                    )
                )
            }

            fn as_typeids(&self) -> Vec<TypeId> {
                vec![ $(TypeId::of::<$ts>(),)* ]
            }
        }
    };
}

DepBuilderImpl!(1, { T1 }, (T1,));
DepBuilderImpl!(2, { T1, T2 }, (T1, T2));
DepBuilderImpl!(3, { T1, T2, T3 }, (T1, T2, T3));

pub struct Ctor {
    ctor: Box<dyn Fn(&Registry) -> Box<dyn Any + Send + Sync>>,
}

pub struct Registry {
    objects: HashMap<std::any::TypeId, Ctor>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            objects: HashMap::new(),
        }
    }

    pub fn transient<T>(&mut self, ctor: fn() -> T)
    where
        T: Send + Sync + 'static,
    {
        self.objects.insert(
            TypeId::of::<T>(),
            Ctor {
                ctor: Box::new(move |_| -> Box<dyn Any + Send + Sync> {
                    let obj = ctor();
                    Box::new(obj)
                }),
            },
        );
    }

    pub fn get_transient<T: Send + Sync + 'static>(&self) -> Option<T> {
        if let Some(ctor) = self.objects.get(&TypeId::of::<T>()) {
            let boxed = (ctor.ctor)(self);
            if let Ok(obj) = boxed.downcast::<T>() {
                return Some(*obj);
            }
        }

        None
    }

    pub fn with_deps<T, Deps>(&mut self) -> Builder<T, Deps>
    where
        Deps: DepBuilder<T>,
    {
        Builder {
            registry: self,
            _marker: PhantomData,
            _marker1: PhantomData,
        }
    }
}

pub struct Builder<'a, T, Deps> {
    registry: &'a mut Registry,
    _marker: PhantomData<T>,
    _marker1: PhantomData<Deps>,
}

impl<'a, T, Deps> Builder<'a, T, Deps>
where
    Deps: DepBuilder<T> + 'static,
    T: Send + Sync + 'static,
{
    pub fn transient(&mut self, ctor: fn(Deps) -> T) {
        self.registry.objects.insert(
            TypeId::of::<T>(),
            Ctor {
                ctor: Box::new(move |this| -> Box<dyn Any + Send + Sync> {
                    let obj = Deps::build(this, ctor);
                    Box::new(obj)
                }),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut registry = Registry::new();
        registry.transient(|| 1_u8);

        let x = registry.get_transient::<u8>();
        assert_eq!(x, Some(1_u8));

        registry
            .with_deps::<_, (Transient<u8>,)>()
            .transient(|(i,)| {
                let i = i.get();
                u16::from(i) + 1_u16
            });

        let x1 = registry.get_transient::<u16>();
        assert_eq!(x1, Some(2_u16));

        registry
            .with_deps::<_, (Transient<u8>, Transient<u16>)>()
            .transient(|(i,j)| {
                let i = i.get();
                let j = j.get();
                u32::from(i) + u32::from(j) + 1_u32
            });

        let x2 = registry.get_transient::<u32>();
        assert_eq!(x2, Some(4_u32));
    }
}
