use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use thiserror::Error;

static DEFAULT_REGISTRY: OnceCell<RwLock<Registry>> = OnceCell::new();

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("already a default registry set for this process")]
    DefaultAlreadySet,
}

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

pub struct Singleton<T> {
    inner: Arc<T>,
}

impl<T: Send + Sync + 'static> Singleton<T> {
    pub fn get(self) -> Arc<T> {
        self.inner
    }
}

impl<T: Send + Sync + 'static> Dep for Singleton<T> {
    fn new(registry: &Registry) -> Self {
        Self {
            inner: registry.get_singleton::<T>().unwrap(),
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

type BoxedCtor = Box<dyn Fn(&Registry) -> Box<dyn Any + Send + Sync> + Send + Sync>;

pub enum Object {
    Transient(BoxedCtor),
    Singleton(Arc<dyn Any + Send + Sync>),
}

#[derive(Default)]
pub struct Registry {
    objects: HashMap<std::any::TypeId, Object>,
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
            Object::Transient(Box::new(move |_| -> Box<dyn Any + Send + Sync> {
                let obj = ctor();
                Box::new(obj)
            })),
        );
    }

    pub fn singleton<T>(&mut self, singleton: T)
    where
        T: Send + Sync + 'static,
    {
        self.objects
            .insert(TypeId::of::<T>(), Object::Singleton(Arc::new(singleton)));
    }

    pub fn get_transient<T>(&self) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        if let Some(Object::Transient(ctor)) = self.objects.get(&TypeId::of::<T>()) {
            let boxed = (ctor)(self);
            if let Ok(obj) = boxed.downcast::<T>() {
                return Some(*obj);
            }
        }

        None
    }

    pub fn get_singleton<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        if let Some(Object::Singleton(singleton)) = self.objects.get(&TypeId::of::<T>()) {
            if let Ok(obj) = Arc::clone(singleton).downcast::<T>() {
                return Some(obj);
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

    pub fn current() -> RwLockReadGuard<'static, Self> {
        DEFAULT_REGISTRY.get().unwrap().read()
    }

    pub fn current_mut() -> RwLockWriteGuard<'static, Self> {
        DEFAULT_REGISTRY.get().unwrap().write()
    }

    pub fn make_current(self) -> Result<(), RegistryError> {
        let res = DEFAULT_REGISTRY.set(RwLock::new(self));
        if res.is_err() {
            Err(RegistryError::DefaultAlreadySet)
        } else {
            Ok(())
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
            Object::Transient(Box::new(move |this| -> Box<dyn Any + Send + Sync> {
                let obj = Deps::build(this, ctor);
                Box::new(obj)
            })),
        );
    }

    pub fn singleton(&mut self, ctor: fn(Deps) -> T) {
        let obj = Deps::build(self.registry, ctor);
        self.registry
            .objects
            .insert(TypeId::of::<T>(), Object::Singleton(Arc::new(obj)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Env {
        fn get(&self, var: &str) -> Option<String>;
    }

    struct RealEnv {}
    impl Env for RealEnv {
        fn get(&self, var: &str) -> Option<String> {
            std::env::var(var).ok()
        }
    }

    struct MockEnv {}
    impl Env for MockEnv {
        fn get(&self, _var: &str) -> Option<String> {
            Some("TEST".to_owned())
        }
    }

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
            .transient(|(i, j)| {
                let i = i.get();
                let j = j.get();
                u32::from(i) + u32::from(j) + 1_u32
            });

        let x2 = registry.get_transient::<u32>();
        assert_eq!(x2, Some(4_u32));

        registry.make_current().unwrap();

        Registry::current_mut().transient(|| -1_i8);
        let x3 = Registry::current().get_transient::<i8>();
        assert_eq!(x3, Some(-1_i8));

        Registry::current_mut().transient::<Box<dyn Env + Send + Sync>>(|| Box::new(MockEnv {}));
        let env = Registry::current().get_transient::<Box<dyn Env + Send + Sync>>();
        assert!(env.is_some());
        assert_eq!(env.unwrap().get("anything"), Some("TEST".to_owned()));
    }

    #[test]
    fn singletons_without_deps() {
        let mut registry = Registry::new();
        registry.transient(|| 1_u8);
        registry.transient(|| 1_u16);
        registry.transient(|| 1_u32);
        registry.singleton(8_i8);
        registry.singleton(16_i16);
        registry.singleton(32_i32);

        let x1 = registry.get_singleton::<i8>();
        assert_eq!(*x1.unwrap(), 8_i8);
        let x2 = registry.get_singleton::<i16>();
        assert_eq!(*x2.unwrap(), 16_i16);
        let x3 = registry.get_singleton::<i32>();
        assert_eq!(*x3.unwrap(), 32_i32);
    }

    #[test]
    fn singletons_with_deps() {
        let mut registry = Registry::new();
        registry.transient(|| 1_u8);
        registry.singleton(8_i8);

        registry
            .with_deps::<_, (Transient<u8>, Singleton<i8>)>()
            .singleton(|(i, j)| {
                let i = i.get();
                let j = j.get();
                i32::from(i) + i32::from(*j)
            });

        let x1 = registry.get_transient::<u8>();
        assert_eq!(x1.unwrap(), 1_u8);
        let x2 = registry.get_singleton::<i8>();
        assert_eq!(*x2.unwrap(), 8_i8);
        let x3 = registry.get_singleton::<i32>();
        assert_eq!(*x3.unwrap(), 9_i32);

    }
}
