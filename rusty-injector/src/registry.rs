use std::any::{Any, TypeId};
use std::marker::PhantomData;

use once_cell::sync::OnceCell;

use crate::dependency_builder::DepBuilder;
use crate::{Arc, HashMap, RegistrationFunc, RwLock, DEFAULT_REGISTRY};

type BoxedAny = Box<dyn Any + Send + Sync>;
type ArcAny = Arc<dyn Any + Send + Sync>;
type BoxedCtor = Box<dyn Fn(&Registry) -> Option<BoxedAny> + Send + Sync>;
type SingletonCell = OnceCell<ArcAny>;
type BoxedSingletonGetter = Box<dyn Fn(&Registry, &SingletonCell) -> Option<ArcAny> + Send + Sync>;
type Validator = Box<dyn Fn(&Registry) -> bool + Send + Sync>;

pub enum Object {
    Transient(BoxedCtor),
    Singleton(BoxedSingletonGetter, SingletonCell),
}

#[derive(Default)]
pub struct Registry {
    objects: RwLock<HashMap<TypeId, Object>>,
    validation: RwLock<HashMap<TypeId, Validator>>,
}

impl Registry {
    pub fn empty() -> Self {
        Self {
            objects: RwLock::new(HashMap::new()),
            validation: RwLock::new(HashMap::new()),
        }
    }

    pub fn transient<T>(&self, ctor: fn() -> T)
    where
        T: Send + Sync + 'static,
    {
        self.objects.write().insert(
            TypeId::of::<T>(),
            Object::Transient(Box::new(move |_| -> Option<BoxedAny> {
                let obj = ctor();
                Some(Box::new(obj))
            })),
        );
        self.validation
            .write()
            .insert(TypeId::of::<T>(), Box::new(|_| true));
    }

    pub fn singleton<T>(&self, ctor: fn() -> T)
    where
        T: Send + Sync + 'static,
    {
        let getter = Box::new(move |_this: &Registry, cell: &SingletonCell| -> Option<ArcAny> {
            let rc = cell.get_or_init(|| Arc::new(ctor()));
            Some(Arc::clone(rc))
        });
        self.objects.write().insert(
            TypeId::of::<T>(),
            Object::Singleton(getter, OnceCell::new()),
        );
        self.validation
            .write()
            .insert(TypeId::of::<T>(), Box::new(|_| true));
    }

    pub fn get_transient<T>(&self) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        if let Some(Object::Transient(ctor)) = self.objects.read().get(&TypeId::of::<T>()) {
            let boxed = (ctor)(self)?;
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
        if let Some(Object::Singleton(getter, cell)) = self.objects.read().get(&TypeId::of::<T>()) {
            let singleton = (getter)(self, cell)?;
            if let Ok(obj) = singleton.downcast::<T>() {
                return Some(obj);
            }
        }

        None
    }

    pub fn with_deps<T, Deps>(&self) -> Builder<T, Deps>
    where
        Deps: DepBuilder<T>,
    {
        Builder {
            registry: self,
            _marker: PhantomData,
            _marker1: PhantomData,
        }
    }

    pub fn validate_all(&self) -> bool {
        let lock = self.validation.read();
        lock.iter().all(|(_, validator)| (validator)(self))
    }

    pub fn validate<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        let lock = self.validation.read();
        if let Some(validator) = lock.get(&TypeId::of::<T>()) {
            (validator)(self)
        } else {
            false
        }
    }

    pub fn global() -> &'static Self {
        DEFAULT_REGISTRY.get_or_init(|| {
            let mut registry = Self::empty();

            eprintln!("run auto registration");
            for register in inventory::iter::<RegistrationFunc> {
                (register.0)(&mut registry);
            }

            registry
        })
    }

    pub unsafe fn reset_global() {
        let registry = Self::global();
        {
            let mut lock = registry.objects.write();
            lock.clear();
        }

        for register in inventory::iter::<RegistrationFunc> {
            (register.0)(registry);
        }
    }
}

pub struct Builder<'a, T, Deps> {
    registry: &'a Registry,
    _marker: PhantomData<T>,
    _marker1: PhantomData<Deps>,
}

impl<'a, T, Deps> Builder<'a, T, Deps>
where
    Deps: DepBuilder<T> + 'static,
    T: Send + Sync + 'static,
{
    pub fn transient(&self, ctor: fn(Deps) -> T) {
        self.registry.objects.write().insert(
            TypeId::of::<T>(),
            Object::Transient(Box::new(move |this| -> Option<BoxedAny> {
                match Deps::build(this, ctor) {
                    Some(obj) => Some(Box::new(obj)),
                    None => None,
                }
            })),
        );
        self.registry.validation.write().insert(
            TypeId::of::<T>(),
            Box::new(|registry: &Registry| {
                let type_ids = Deps::as_typeids();
                type_ids.iter().all(|el| {
                    if let Some(validator) = registry.validation.read().get(el) {
                        return (validator)(registry);
                    }

                    false
                })
            }),
        );
    }

    pub fn singleton(&self, ctor: fn(Deps) -> T) {
        let getter = Box::new(
            move |this: &Registry, cell: &SingletonCell| -> Option<ArcAny> {
                match Deps::build(this, ctor) {
                    Some(obj) => {
                        let rc = cell.get_or_init(|| Arc::new(obj));
                        Some(Arc::clone(rc))
                    }
                    None => None,
                }
            },
        );
        self.registry.objects.write().insert(
            TypeId::of::<T>(),
            Object::Singleton(getter, OnceCell::new()),
        );
        self.registry.validation.write().insert(
            TypeId::of::<T>(),
            Box::new(|registry: &Registry| {
                let type_ids = Deps::as_typeids();
                type_ids.iter().all(|el| {
                    if let Some(validator) = registry.validation.read().get(el) {
                        return (validator)(registry);
                    }

                    false
                })
            }),
        );
    }
}
