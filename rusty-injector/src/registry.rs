use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::atomic::Ordering;

use crate::dependency_builder::DepBuilder;
use crate::error::RegistryError;
use crate::{
    Arc, HashMap, RegistrationFunc, RwLock, RwLockReadGuard, RwLockWriteGuard, AUTOREGISTERED,
    DEFAULT_REGISTRY,
};

type BoxedCtor = Box<dyn Fn(&Registry) -> Box<dyn Any + Send + Sync> + Send + Sync>;

pub enum Object {
    Transient(BoxedCtor),
    Singleton(Arc<dyn Any + Send + Sync>),
}

#[derive(Default)]
pub struct Registry {
    objects: HashMap<TypeId, Object>,
}

impl Registry {
    pub fn new() -> Self {
        let mut registry = Registry {
            objects: HashMap::new(),
        };

        if AUTOREGISTERED
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
        {
            eprintln!("run auto registration");
            for register in inventory::iter::<RegistrationFunc> {
                (register.0)(&mut registry);
            }
        }

        registry
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

    pub unsafe fn reset_registry() {
        AUTOREGISTERED.store(false, Ordering::SeqCst);
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
