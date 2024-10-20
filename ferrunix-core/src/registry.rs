//! Holds all registered types that can be injected or constructed.
#![allow(clippy::missing_docs_in_private_items)]

use std::any::TypeId;
use std::marker::PhantomData;

use crate::dependency_builder::{self, DepBuilder};
use crate::object_builder::Object;
use crate::types::{NonAsyncRwLock, Registerable, Validator};
use crate::{
    registration::RegistrationFunc, registration::DEFAULT_REGISTRY,
    types::HashMap, types::Ref, types::RwLock,
};

/// Registry for all types that can be constructed or otherwise injected.
pub struct Registry {
    objects: RwLock<HashMap<TypeId, Object>>,
    validation: NonAsyncRwLock<HashMap<TypeId, Validator>>,
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
            validation: NonAsyncRwLock::new(HashMap::new()),
        }
    }

    /// Create an empty registry, and add all autoregistered types into it.
    ///
    /// This is the constructor for the global registry that can be acquired
    /// with [`Registry::global`].
    #[cfg(any(doc, not(feature = "tokio")))]
    #[must_use]
    pub fn autoregistered() -> Self {
        let registry = Self::empty();
        for register in inventory::iter::<RegistrationFunc> {
            (register.0)(&registry);
        }

        registry
    }

    /// Create an empty registry, and add all autoregistered types into it.
    ///
    /// This is the constructor for the global registry that can be acquired
    /// with [`Registry::global`].
    #[cfg(any(doc, feature = "tokio"))]
    #[must_use]
    pub async fn autoregistered() -> Self {
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
    #[cfg(not(feature = "tokio"))]
    pub fn transient<T>(&self, ctor: fn() -> T)
    where
        T: Registerable,
    {
        use crate::object_builder::TransientBuilderImplNoDeps;

        let transient =
            Object::Transient(Box::new(TransientBuilderImplNoDeps::new(ctor)));
        {
            let mut lock = self.objects.write();
            lock.insert(TypeId::of::<T>(), transient);
        }

        let validator: Validator = Box::new(|_| true);
        {
            let mut lock = self.validation.write();
            lock.insert(TypeId::of::<T>(), validator);
        }
    }

    /// Register a new transient object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning the newly constructed `T`.
    ///     This constructor will be called for every `T` that is requested.
    #[cfg(feature = "tokio")]
    pub async fn transient<T>(
        &self,
        ctor: fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) where
        T: Registerable,
    {
        use crate::object_builder::AsyncTransientBuilderImplNoDeps;

        let transient = Object::AsyncTransient(Box::new(
            AsyncTransientBuilderImplNoDeps::new(ctor),
        ));

        {
            let mut lock = self.objects.write().await;
            lock.insert(TypeId::of::<T>(), transient);
        }

        let validator: Validator = Box::new(|_| true);
        {
            let mut lock = self.validation.write();
            lock.insert(TypeId::of::<T>(), validator);
        }
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
    #[cfg(not(feature = "tokio"))]
    pub fn singleton<T>(&self, ctor: fn() -> T)
    where
        T: Registerable,
    {
        use crate::object_builder::SingletonGetterNoDeps;

        let singleton =
            Object::Singleton(Box::new(SingletonGetterNoDeps::new(ctor)));

        {
            let mut lock = self.objects.write();
            lock.insert(TypeId::of::<T>(), singleton);
        }

        let validator: Validator = Box::new(|_| true);
        {
            let mut lock = self.validation.write();
            lock.insert(TypeId::of::<T>(), validator);
        }
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
    #[cfg(feature = "tokio")]
    pub async fn singleton<T>(
        &self,
        ctor: fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) where
        T: Registerable + Clone,
    {
        use crate::object_builder::AsyncSingletonNoDeps;

        let singleton =
            Object::AsyncSingleton(Box::new(AsyncSingletonNoDeps::new(ctor)));

        {
            let mut lock = self.objects.write().await;
            lock.insert(TypeId::of::<T>(), singleton);
        }

        let validator: Validator = Box::new(|_| true);
        {
            let mut lock = self.validation.write();
            lock.insert(TypeId::of::<T>(), validator);
        }
    }

    /// Retrieves a newly constructed `T` from this registry.
    ///
    /// Returns `None` if `T` wasn't registered or failed to construct.
    #[cfg(not(feature = "tokio"))]
    #[must_use]
    pub fn get_transient<T>(&self) -> Option<T>
    where
        T: Registerable,
    {
        let lock = self.objects.read();
        if let Some(Object::Transient(transient)) = lock.get(&TypeId::of::<T>())
        {
            let resolved = transient.make_transient(self)?;
            drop(lock);
            if let Ok(obj) = resolved.downcast::<T>() {
                return Some(*obj);
            }
        }

        None
    }

    /// Retrieves a newly constructed `T` from this registry.
    ///
    /// Returns `None` if `T` wasn't registered or failed to construct.
    #[cfg(feature = "tokio")]
    #[must_use]
    pub async fn get_transient<T>(&self) -> Option<T>
    where
        T: Registerable,
    {
        let lock = self.objects.read().await;
        if let Some(Object::AsyncTransient(ctor)) = lock.get(&TypeId::of::<T>())
        {
            let boxed = ctor.make_transient(self).await?;
            drop(lock);
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
    #[cfg(not(feature = "tokio"))]
    #[must_use]
    pub fn get_singleton<T>(&self) -> Option<Ref<T>>
    where
        T: Registerable,
    {
        let lock = self.objects.read();
        if let Some(Object::Singleton(singleton)) = lock.get(&TypeId::of::<T>())
        {
            let resolved = singleton.get_singleton(self)?;
            drop(lock);
            if let Ok(obj) = resolved.downcast::<T>() {
                return Some(obj);
            }
        }

        None
    }

    /// Retrieves the singleton `T` from this registry.
    ///
    /// Returns `None` if `T` wasn't registered or failed to construct. The
    /// singleton is a ref-counted pointer object (either `Arc` or `Rc`).
    #[cfg(feature = "tokio")]
    #[must_use]
    pub async fn get_singleton<T>(&self) -> Option<Ref<T>>
    where
        T: Registerable,
    {
        let lock = self.objects.read().await;
        if let Some(Object::AsyncSingleton(singleton)) =
            lock.get(&TypeId::of::<T>())
        {
            let resolved = singleton.get_singleton(self).await?;
            drop(lock);
            if let Ok(obj) = resolved.downcast::<T>() {
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
    #[cfg(all(not(feature = "tokio"), not(feature = "multithread")))]
    pub fn global() -> std::rc::Rc<Self> {
        DEFAULT_REGISTRY.with(|val| {
            let ret =
                val.get_or_init(|| std::rc::Rc::new(Self::autoregistered()));
            std::rc::Rc::clone(ret)
        })
    }

    /// Access the global registry.
    ///
    /// This registry contains the types that are marked for auto-registration
    /// via the derive macro.
    #[cfg(feature = "tokio")]
    pub async fn global() -> &'static Self {
        DEFAULT_REGISTRY.get_or_init(Self::autoregistered).await
    }

    /// Reset the global registry, removing all previously registered types, and
    /// re-running the auto-registration routines.
    ///
    /// # Safety
    /// Ensure that no other thread is currently using [`Registry::global()`].
    #[allow(unsafe_code)]
    #[cfg(not(feature = "tokio"))]
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

    /// Reset the global registry, removing all previously registered types, and
    /// re-running the auto-registration routines.
    ///
    /// # Safety
    /// Ensure that no other thread is currently using [`Registry::global()`].
    #[allow(unsafe_code)]
    #[cfg(feature = "tokio")]
    pub async unsafe fn reset_global() {
        let registry = Self::global().await;
        {
            let mut lock = registry.objects.write().await;
            lock.clear();
        }

        for register in inventory::iter::<RegistrationFunc> {
            #[cfg(not(feature = "multithread"))]
            (register.0)(&registry);
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
pub struct Builder<'reg, T, Deps> {
    registry: &'reg Registry,
    _marker: PhantomData<T>,
    _marker1: PhantomData<Deps>,
}

impl<
        'reg,
        T,
        #[cfg(not(feature = "tokio"))] Deps: DepBuilder<T> + 'static,
        #[cfg(feature = "tokio")] Deps: DepBuilder<T> + Sync + 'static,
    > Builder<'reg, T, Deps>
where
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
    #[cfg(not(feature = "tokio"))]
    pub fn transient(&self, ctor: fn(Deps) -> T) {
        use crate::object_builder::TransientBuilderImplWithDeps;

        let transient = Object::Transient(Box::new(
            TransientBuilderImplWithDeps::new(ctor),
        ));
        {
            let mut lock = self.registry.objects.write();
            lock.insert(TypeId::of::<T>(), transient);
        }

        let validator: Validator = Box::new(|registry: &Registry| {
            let type_ids =
                Deps::as_typeids(dependency_builder::private::SealToken);
            type_ids.iter().all(|el| {
                if let Some(validator) = registry.validation.read().get(el) {
                    return (validator)(registry);
                }

                false
            })
        });

        {
            let mut lock = self.registry.validation.write();
            lock.insert(TypeId::of::<T>(), validator);
        }
    }

    /// Register a new transient object, with dependencies specified in
    /// `.with_deps`.
    ///
    /// The `ctor` parameter is a constructor function returning the newly
    /// constructed `T`. The constructor accepts a single argument `Deps` (a
    /// tuple implementing [`crate::dependency_builder::DepBuilder`]). It's
    /// best to destructure the tuple to accept each dependency separately.
    /// This constructor will be called for every `T` that is requested.
    ///
    /// The `ctor` must return a boxed `dyn Future`.
    #[cfg(feature = "tokio")]
    pub async fn transient(
        &self,
        ctor: fn(
            Deps,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) {
        use crate::object_builder::AsyncTransientBuilderImplWithDeps;

        let transient = Object::AsyncTransient(Box::new(
            AsyncTransientBuilderImplWithDeps::new(ctor),
        ));
        {
            let mut lock = self.registry.objects.write().await;
            lock.insert(TypeId::of::<T>(), transient);
        }

        let validator: Validator = Box::new(|registry: &Registry| {
            let type_ids =
                Deps::as_typeids(dependency_builder::private::SealToken);
            type_ids.iter().all(|el| {
                if let Some(validator) = registry.validation.read().get(el) {
                    return (validator)(registry);
                }

                false
            })
        });

        {
            let mut lock = self.registry.validation.write();
            lock.insert(TypeId::of::<T>(), validator);
        }
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
    #[cfg(not(feature = "tokio"))]
    pub fn singleton(&self, ctor: fn(Deps) -> T) {
        use crate::object_builder::SingletonGetterWithDeps;

        let singleton =
            Object::Singleton(Box::new(SingletonGetterWithDeps::new(ctor)));
        {
            let mut lock = self.registry.objects.write();
            lock.insert(TypeId::of::<T>(), singleton);
        }

        let validator: Validator = Box::new(|registry: &Registry| {
            let type_ids =
                Deps::as_typeids(dependency_builder::private::SealToken);
            type_ids.iter().all(|el| {
                if let Some(validator) = registry.validation.read().get(el) {
                    return (validator)(registry);
                }

                false
            })
        });
        {
            let mut lock = self.registry.validation.write();
            lock.insert(TypeId::of::<T>(), validator);
        }
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
    /// The `ctor` must return a boxed `dyn Future`.
    #[cfg(feature = "tokio")]
    pub async fn singleton(
        &self,
        ctor: fn(
            Deps,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) {
        use crate::object_builder::AsyncSingletonWithDeps;

        let singleton =
            Object::AsyncSingleton(Box::new(AsyncSingletonWithDeps::new(ctor)));
        {
            let mut lock = self.registry.objects.write().await;
            lock.insert(TypeId::of::<T>(), singleton);
        }

        let validator: Validator = Box::new(|registry: &Registry| {
            let type_ids =
                Deps::as_typeids(dependency_builder::private::SealToken);
            type_ids.iter().all(|el| {
                if let Some(validator) = registry.validation.read().get(el) {
                    return (validator)(registry);
                }

                false
            })
        });
        {
            let mut lock = self.registry.validation.write();
            lock.insert(TypeId::of::<T>(), validator);
        }
    }
}

impl<T, Dep> std::fmt::Debug for Builder<'_, T, Dep> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Builder").finish()
    }
}
