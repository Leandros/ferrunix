//! Holds all registered types that can be injected or constructed.
#![allow(clippy::missing_docs_in_private_items)]

use std::any::TypeId;
use std::marker::PhantomData;

use crate::dependency_builder::{self, DepBuilder};
use crate::types::{
    BoxedAny, BoxedCtor, BoxedSingletonGetter, RefAny, Registerable,
    SingletonCell, Validator,
};
use crate::{
    registration::RegistrationFunc, registration::DEFAULT_REGISTRY,
    types::HashMap, types::OnceCell, types::Ref, types::RwLock,
};

/// All possible "objects" that can be held by the registry.
enum Object {
    Transient(BoxedCtor),
    Singleton(BoxedSingletonGetter, SingletonCell),
}

/// Registry for all types that can be constructed or otherwise injected.
pub struct Registry {
    objects: RwLock<HashMap<TypeId, Object>>,
    validation: RwLock<HashMap<TypeId, Validator>>,
}

impl Registry {
    /// Create a new, empty, registry. This registry contains no pre-registered
    /// types.
    ///
    /// Types that are auto-registered are also not included in this registry.
    ///
    /// To get access to the auto-registered types (types that are annotated by
    /// the derive macro), the global registry [`Registry::global`] needs to
    /// be used.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            objects: RwLock::new(HashMap::new()),
            validation: RwLock::new(HashMap::new()),
        }
    }

    /// Create an empty registry, and add all autoregistered types into it.
    ///
    /// This is the constructor for the global registry that can be acquired
    /// with [`Registry::global`].
    #[must_use]
    pub fn autoregistered() -> Self {
        let registry = Self::empty();
        for register in inventory::iter::<RegistrationFunc> {
            (register.0)(&registry);
        }

        registry
    }

    /// Register a new transient object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning the newly constructed `T`.
    ///     This constructor will be called for every `T` that is requested.
    pub fn transient<T>(&self, ctor: fn() -> T)
    where
        T: Registerable,
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

    /// Register a new singleton object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning the newly constructed `T`.
    ///     This constructor will be called once, lazily, when the first
    ///     instance of `T` is requested.
    pub fn singleton<T>(&self, ctor: fn() -> T)
    where
        T: Registerable,
    {
        let getter = Box::new(
            move |_this: &Self, cell: &SingletonCell| -> Option<RefAny> {
                let rc = cell.get_or_init(|| Ref::new(ctor()));
                Some(Ref::clone(rc))
            },
        );
        self.objects.write().insert(
            TypeId::of::<T>(),
            Object::Singleton(getter, OnceCell::new()),
        );
        self.validation
            .write()
            .insert(TypeId::of::<T>(), Box::new(|_| true));
    }

    /// Retrieves a newly constructed `T` from this registry.
    ///
    /// Returns `None` if `T` wasn't registered or failed to construct.
    pub fn get_transient<T>(&self) -> Option<T>
    where
        T: Registerable,
    {
        if let Some(Object::Transient(ctor)) =
            self.objects.read().get(&TypeId::of::<T>())
        {
            let boxed = (ctor)(self)?;
            if let Ok(obj) = boxed.downcast::<T>() {
                return Some(*obj);
            }
        }

        None
    }

    /// Retrieves the singleton `T` from this registry.
    ///
    /// Returns `None` if `T` wasn't registered or failed to construct. The
    /// singleton is a ref-counted pointer object (either `Arc` or `Rc`).
    pub fn get_singleton<T>(&self) -> Option<Ref<T>>
    where
        T: Registerable,
    {
        if let Some(Object::Singleton(getter, cell)) =
            self.objects.read().get(&TypeId::of::<T>())
        {
            let singleton = (getter)(self, cell)?;
            if let Ok(obj) = singleton.downcast::<T>() {
                return Some(obj);
            }
        }

        None
    }

    /// Register a new transient or singleton with dependencies.
    pub fn with_deps<T, Deps>(&self) -> Builder<'_, T, Deps>
    where
        Deps: DepBuilder<T>,
    {
        Builder {
            registry: self,
            _marker: PhantomData,
            _marker1: PhantomData,
        }
    }

    /// Check whether all registered types have the required dependencies.
    ///
    /// Returns true if for all registered types all of it's dependencies can be
    /// constructed, false otherwise.
    ///
    /// This is a potentially expensive call since it needs to go through the
    /// entire dependency tree for each registered type.
    ///
    /// Nontheless, it's recommended to call this before using the [`Registry`].
    pub fn validate_all(&self) -> bool {
        let lock = self.validation.read();
        lock.iter().all(|(_, validator)| (validator)(self))
    }

    /// Check whether the type `T` is registered in this registry, and all
    /// dependencies of the type `T` are also registered.
    ///
    /// Returns true if the type and it's dependencies can be constructed, false
    /// otherwise.
    pub fn validate<T>(&self) -> bool
    where
        T: Registerable,
    {
        let lock = self.validation.read();
        lock.get(&TypeId::of::<T>())
            .map_or(false, |validator| (validator)(self))
    }

    /// Access the global registry.
    ///
    /// This registry contains the types that are marked for auto-registration
    /// via the derive macro.
    #[cfg(feature = "multithread")]
    pub fn global() -> &'static Self {
        DEFAULT_REGISTRY.get_or_init(Self::autoregistered)
    }

    /// Access the global registry.
    ///
    /// This registry contains the types that are marked for auto-registration
    /// via the derive macro.
    #[cfg(not(feature = "multithread"))]
    pub fn global() -> std::rc::Rc<Self> {
        DEFAULT_REGISTRY.with(|val| {
            let ret =
                val.get_or_init(|| std::rc::Rc::new(Self::autoregistered()));
            std::rc::Rc::clone(ret)
        })
    }

    /// Reset the global registry, removing all previously registered types, and
    /// re-running the auto-registration routines.
    ///
    /// # Safety
    /// Ensure that no other thread is currently using [`Registry::global()`].
    #[allow(unsafe_code)]
    pub unsafe fn reset_global() {
        let registry = Self::global();
        {
            let mut lock = registry.objects.write();
            lock.clear();
        }

        for register in inventory::iter::<RegistrationFunc> {
            #[cfg(not(feature = "multithread"))]
            (register.0)(&registry);

            #[cfg(feature = "multithread")]
            (register.0)(registry);
        }
    }
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Registry").finish()
    }
}

/// A builder for objects with dependencies. This can be created by using
/// [`Registry::with_deps`].
#[allow(clippy::single_char_lifetime_names)]
pub struct Builder<'a, T, Deps> {
    registry: &'a Registry,
    _marker: PhantomData<T>,
    _marker1: PhantomData<Deps>,
}

impl<T, Deps> Builder<'_, T, Deps>
where
    Deps: DepBuilder<T> + 'static,
    T: Registerable,
{
    /// Register a new transient object, with dependencies specified in
    /// `.with_deps`.
    ///
    /// The `ctor` parameter is a constructor function returning the newly
    /// constructed `T`. The constructor accepts a single argument `Deps` (a
    /// tuple implementing [`crate::dependency_builder::DepBuilder`]). It's
    /// best to destructure the tuple to accept each dependency separately.
    /// This constructor will be called for every `T` that is requested.
    ///
    /// # Example
    /// ```no_run
    /// # use ferrunix_core::{Registry, Singleton, Transient};
    /// # let registry = Registry::empty();
    /// # struct Template {
    /// #     template: &'static str,
    /// # }
    /// registry
    ///     .with_deps::<_, (Transient<u8>, Singleton<Template>)>()
    ///     .transient(|(num, template)| {
    ///         // access `num` and `template` here.
    ///         u16::from(*num)
    ///     });
    /// ```
    ///
    /// For single dependencies, the destructured tuple needs to end with a
    /// comma: `(dep,)`.
    pub fn transient(&self, ctor: fn(Deps) -> T) {
        self.registry.objects.write().insert(
            TypeId::of::<T>(),
            Object::Transient(Box::new(move |this| -> Option<BoxedAny> {
                #[allow(clippy::option_if_let_else)]
                match Deps::build(
                    this,
                    ctor,
                    dependency_builder::private::SealToken,
                ) {
                    Some(obj) => Some(Box::new(obj)),
                    None => None,
                }
            })),
        );
        self.registry.validation.write().insert(
            TypeId::of::<T>(),
            Box::new(|registry: &Registry| {
                let type_ids =
                    Deps::as_typeids(dependency_builder::private::SealToken);
                type_ids.iter().all(|el| {
                    if let Some(validator) = registry.validation.read().get(el)
                    {
                        return (validator)(registry);
                    }

                    false
                })
            }),
        );
    }

    /// Register a new singleton object, with dependencies specified in
    /// `.with_deps`.
    ///
    /// The `ctor` parameter is a constructor function returning the newly
    /// constructed `T`. The constructor accepts a single argument `Deps` (a
    /// tuple implementing [`crate::dependency_builder::DepBuilder`]). It's
    /// best to destructure the tuple to accept each dependency separately.
    /// This constructor will be called once, lazily, when the first
    /// instance of `T` is requested.
    ///
    /// # Example
    /// ```no_run
    /// # use ferrunix_core::{Registry, Singleton, Transient};
    /// # let registry = Registry::empty();
    /// # struct Template {
    /// #     template: &'static str,
    /// # }
    /// registry
    ///     .with_deps::<_, (Transient<u8>, Singleton<Template>)>()
    ///     .transient(|(num, template)| {
    ///         // access `num` and `template` here.
    ///         u16::from(*num)
    ///     });
    /// ```
    ///
    /// For single dependencies, the destructured tuple needs to end with a
    /// comma: `(dep,)`.
    pub fn singleton(&self, ctor: fn(Deps) -> T) {
        let getter = Box::new(
            move |this: &Registry, cell: &SingletonCell| -> Option<RefAny> {
                #[allow(clippy::option_if_let_else)]
                match Deps::build(
                    this,
                    ctor,
                    dependency_builder::private::SealToken,
                ) {
                    Some(obj) => {
                        let rc = cell.get_or_init(|| Ref::new(obj));
                        Some(Ref::clone(rc))
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
                let type_ids =
                    Deps::as_typeids(dependency_builder::private::SealToken);
                type_ids.iter().all(|el| {
                    if let Some(validator) = registry.validation.read().get(el)
                    {
                        return (validator)(registry);
                    }

                    false
                })
            }),
        );
    }
}

impl<T, Dep> std::fmt::Debug for Builder<'_, T, Dep> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Builder").finish()
    }
}
