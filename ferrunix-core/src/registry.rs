//! Holds all registered types that can be injected or constructed.
#![allow(clippy::multiple_inherent_impl)]

use std::any::TypeId;
use std::marker::PhantomData;

use crate::cycle_detection::{
    DependencyValidator, FullValidationError, ValidationError,
};
use crate::dependency_builder::DepBuilder;
use crate::error::{ImplErrors, ResolveError};
use crate::object_builder::Object;
use crate::types::{BoxErr, RefWeak, Registerable, RegisterableSingleton};
#[cfg(not(feature = "tokio"))]
use crate::types::{
    SingletonCtor, SingletonCtorDeps, SingletonCtorFallible,
    SingletonCtorFallibleDeps, TransientCtor, TransientCtorDeps,
    TransientCtorFallible, TransientCtorFallibleDeps,
};
use crate::{
    registration::RegistrationFunc, registration::DEFAULT_REGISTRY,
    types::HashMap, types::Ref, types::RwLock,
};

/// Registry for all types that can be constructed or otherwise injected.
pub struct Registry {
    /// Parent registry. None if it's the root registry.
    parent: Option<RefWeak<Registry>>,
    /// Internal hashtable of all registered objects.
    objects: RwLock<HashMap<TypeId, Object>>,
    /// Validation.
    validator: DependencyValidator,
}

#[allow(clippy::multiple_inherent_impl)]
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
            parent: None,
            objects: RwLock::new(HashMap::new()),
            validator: DependencyValidator::new(),
        }
    }

    /// Register a new transient or singleton with dependencies.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
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
    /// This is a potentially expensive call since it needs to go through the
    /// entire dependency tree for each registered type.
    ///
    /// Nontheless, it's recommended to call this before using the [`Registry`].
    ///
    /// # Errors
    /// Returns a [`ValidationError`] when the dependency graph is missing dependencies or
    /// has cycles.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn validate_all(&self) -> Result<(), ValidationError> {
        self.validator.validate_all()
    }

    /// Check whether all registered types have the required dependencies and returns a
    /// detailed error about what's missing or where a cycle was detected.
    ///
    /// This is a potentially expensive call since it needs to go through the
    /// entire dependency tree for each registered type.
    ///
    /// # Errors
    /// Returns a [`FullValidationError`] when the dependency graph is missing dependencies or has
    /// cycles.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn validate_all_full(&self) -> Result<(), FullValidationError> {
        self.validator.validate_all_full()
    }

    /// Check whether the type `T` is registered in this registry, and all
    /// dependencies of the type `T` are also registered.
    ///
    /// # Errors
    /// Returns a [`ValidationError`] when the dependency graph is missing dependencies or has cycles.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn validate<T>(&self) -> Result<(), ValidationError>
    where
        T: Registerable,
    {
        self.validator.validate::<T>()
    }

    /// Return a string of the dependency graph visualized using graphviz's `dot` language.
    ///
    /// # Errors
    /// Returns a [`ValidationError`] when the dependency graph is missing dependencies or has cycles.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn dotgraph(&self) -> Result<String, ValidationError> {
        self.validator.dotgraph()
    }

    /// Access the global registry.
    ///
    /// This registry contains the types that are marked for auto-registration
    /// via the derive macro.
    #[cfg(all(not(feature = "tokio"), not(feature = "multithread")))]
    #[cfg_attr(feature = "tracing", tracing::instrument)]
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
    #[cfg(all(feature = "multithread", not(feature = "tokio")))]
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn global() -> Ref<Self> {
        let ret = DEFAULT_REGISTRY.get_or_init(|| {
            let new = Self::autoregistered();
            Ref::new(new)
        });
        Ref::clone(ret)
    }

    /// Create a new child registry.
    ///
    /// When this registry is used to register and retrieve objects, the objects
    /// directly registered on the returned registry are preferred to the objects
    /// that have been previously registered with the parent registry.
    ///
    /// This allows for sub-registries to override objects from their parent.
    pub fn child(self: &Ref<Self>) -> Ref<Self> {
        Ref::new(Self {
            parent: Some(Ref::downgrade(self)),
            objects: RwLock::new(HashMap::new()),
            validator: DependencyValidator::new(),
        })
    }
}

#[cfg(not(feature = "tokio"))]
impl Registry {
    /// Register a new transient object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning the newly constructed `T`.
    ///     This constructor will be called for every `T` that is requested.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub fn register_transient<T, C>(&self, ctor: C)
    where
        T: Registerable,
        C: TransientCtor<T> + Copy + 'static,
    {
        self.try_register_transient::<T, _>(move || -> Result<T, BoxErr> {
            Ok((ctor)())
        });
    }

    /// Register a new transient object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning a `Result<T, E>`.
    ///     This constructor will be called for every `T` that is requested.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub fn try_register_transient<T, C>(&self, ctor: C)
    where
        T: Registerable,
        C: TransientCtorFallible<T> + Copy + 'static,
    {
        use crate::object_builder::TransientBuilderImplNoDeps;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "registering transient ({})",
            std::any::type_name::<T>()
        );

        let closure = Box::new(move || -> Result<T, BoxErr> { (ctor)() });
        let transient = Object::Transient(Box::new(
            TransientBuilderImplNoDeps::new(closure),
        ));

        self.insert_or_panic::<T>(transient);
        self.validator.add_transient_no_deps::<T>();
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
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub fn register_singleton<T, F>(&self, ctor: F)
    where
        T: RegisterableSingleton,
        F: SingletonCtor<T>,
    {
        self.try_register_singleton(move || -> Result<T, BoxErr> {
            Ok((ctor)())
        });
    }

    /// Register a new singleton object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning a `Result<T, E>`.
    ///     This constructor will be called once, lazily, when the first
    ///     instance of `T` is requested.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub fn try_register_singleton<T, F>(&self, ctor: F)
    where
        T: RegisterableSingleton,
        F: SingletonCtorFallible<T>,
    {
        use crate::object_builder::SingletonGetterNoDeps;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "registering singleton ({})",
            std::any::type_name::<T>()
        );

        let singleton =
            Object::Singleton(Box::new(SingletonGetterNoDeps::new(ctor)));

        self.insert_or_panic::<T>(singleton);
        self.validator.add_singleton_no_deps::<T>();
    }

    /// Retrieves a newly constructed `T` from this registry.
    ///
    /// # Errors
    /// Returns an error if the registered constructor fails, or the type fails
    /// to resolve (i.e is not registered).
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn transient<T>(&self) -> Result<T, ResolveError>
    where
        T: Registerable,
    {
        let lock = self.objects.read();
        if let Some(Object::Transient(transient)) = lock.get(&TypeId::of::<T>())
        {
            let resolved = transient.make_transient(self)?;
            drop(lock);
            let ret = resolved.downcast::<T>().map_err(|_self| {
                ResolveError::Impl(ImplErrors::TypeMismatch)
            })?;
            return Ok(*ret);
        }

        #[allow(clippy::redundant_closure_for_method_calls)]
        self.parent
            .as_ref()
            .and_then(|weakref| weakref.upgrade())
            .map_or_else(
                || {
                    Err(ResolveError::TypeMissing {
                        typename: std::any::type_name::<T>(),
                    })
                },
                |parent| parent.transient::<T>(),
            )
    }

    /// Retrieves the singleton `T` from this registry.
    ///
    /// The return singleton is a ref-counted pointer object (either `Arc` or `Rc`).
    ///
    /// # Errors
    /// Returns an error if the registered constructor fails, or the type fails
    /// to resolve (i.e is not registered).
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn singleton<T>(&self) -> Result<Ref<T>, ResolveError>
    where
        T: RegisterableSingleton,
    {
        let lock = self.objects.read();
        if let Some(Object::Singleton(singleton)) = lock.get(&TypeId::of::<T>())
        {
            let resolved = singleton.get_singleton(self)?;
            drop(lock);
            let obj = resolved.downcast::<T>().map_err(|_self| {
                ResolveError::Impl(ImplErrors::TypeMismatch)
            })?;
            return Ok(obj);
        }

        #[allow(clippy::redundant_closure_for_method_calls)]
        self.parent
            .as_ref()
            .and_then(|weakref| weakref.upgrade())
            .map_or_else(
                || {
                    Err(ResolveError::TypeMissing {
                        typename: std::any::type_name::<T>(),
                    })
                },
                |parent| parent.singleton::<T>(),
            )
    }

    /// Reset the global registry, removing all previously registered types, and
    /// re-running the auto-registration routines.
    ///
    /// # Safety
    /// Ensure that no other thread is currently using [`Registry::global()`].
    #[allow(unsafe_code)]
    #[cfg_attr(feature = "tracing", tracing::instrument)]
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
            (register.0)(&registry);
        }
    }

    /// Create an empty registry, and add all autoregistered types into it.
    ///
    /// This is the constructor for the global registry that can be acquired
    /// with [`Registry::global`].
    #[must_use]
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn autoregistered() -> Self {
        let registry = Self::empty();
        for register in inventory::iter::<RegistrationFunc> {
            (register.0)(&registry);
        }

        registry
    }

    /// Inserts a new object into the objecs hashtable.
    ///
    /// This acquires an exclusive lock on `self.objects`.
    ///
    /// # Panics
    /// If the key already exists (=> the type was previously registered).
    #[inline]
    fn insert_or_panic<T: 'static>(&self, value: Object) {
        let mut lock = self.objects.write();
        let entry = lock.entry(TypeId::of::<T>());
        match entry {
            #[allow(clippy::panic)]
            hashbrown::hash_map::Entry::Occupied(_) => panic!(
                "Type '{}' ({:?}) is already registered",
                std::any::type_name::<T>(),
                TypeId::of::<T>()
            ),
            hashbrown::hash_map::Entry::Vacant(view) => {
                view.insert(value);
            }
        }
    }
}

#[cfg(feature = "tokio")]
impl Registry {
    /// Create an empty registry, and add all autoregistered types into it.
    ///
    /// This is the constructor for the global registry that can be acquired
    /// with [`Registry::global`].
    ///
    /// # Panics
    /// If any of the constructors panic.
    #[must_use]
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn autoregistered() -> Self {
        use std::sync::Arc;

        let registry = Arc::new(Self::empty());

        let mut set = tokio::task::JoinSet::new();
        for register in inventory::iter::<RegistrationFunc> {
            let registry = Arc::clone(&registry);
            set.spawn(async move {
                let inner_registry = registry;
                (register.0)(&inner_registry).await;
            });
        }

        #[allow(clippy::panic)]
        while let Some(res) = set.join_next().await {
            match res {
                Ok(_) => {}
                Err(err) if err.is_panic() => {
                    std::panic::resume_unwind(err.into_panic())
                }
                Err(err) => panic!("{err}"),
            }
        }

        assert_eq!(
            Arc::strong_count(&registry), 1,
            "all of the tasks in the `JoinSet` should've joined, dropping their \
            Arc's. some task is still holding an Arc");
        Arc::try_unwrap(registry).expect("all tasks above are joined")
    }

    /// Register a new singleton object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning the newly constructed `T`. This constructor
    ///     will be called once, lazily, when the first instance of `T` is requested.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub async fn register_singleton<T, F>(&self, ctor: F)
    where
        T: RegisterableSingleton,
        F: crate::types::SingletonCtor<T> + Copy,
    {
        let wrapped = move || {
            Box::pin(async move {
                let obj = (ctor)().await;
                Ok::<_, BoxErr>(obj)
            })
                as crate::types::BoxFuture<'static, Result<T, BoxErr>>
        };

        self.try_register_singleton::<T, _>(wrapped).await;
    }

    /// Try registering a new singleton object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning the newly constructed `Result<T>`. This
    ///     constructor will be called once, lazily, when the first instance of `T` is requested.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub async fn try_register_singleton<T, F>(&self, ctor: F)
    where
        T: RegisterableSingleton,
        F: crate::types::SingletonCtorFallible<T>,
    {
        use crate::object_builder::AsyncSingletonNoDeps;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "registering singleton ({})",
            std::any::type_name::<T>()
        );

        let singleton =
            Object::AsyncSingleton(Box::new(AsyncSingletonNoDeps::new(ctor)));

        self.insert_or_panic::<T>(singleton).await;
        self.validator.add_singleton_no_deps::<T>();
    }

    /// Register a new transient object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning the newly constructed `T`. This constructor
    ///     will be called for every `T` that is requested.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub async fn register_transient<T, F>(&self, ctor: F)
    where
        T: Registerable,
        F: crate::types::TransientCtor<T> + Copy,
    {
        let wrapped = move || {
            Box::pin(async move {
                let obj = (ctor)().await;
                Ok::<T, BoxErr>(obj)
            })
        }
            as crate::types::BoxFuture<'static, Result<T, BoxErr>>;

        self.try_register_transient::<T, _>(wrapped).await;
    }

    /// Try registering a new transient object, without dependencies.
    ///
    /// To register a type with dependencies, use the builder returned from
    /// [`Registry::with_deps`].
    ///
    /// # Parameters
    ///   * `ctor`: A constructor function returning the newly constructed `Result<T>`. This
    ///     constructor will be called for every `T` that is requested.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub async fn try_register_transient<T, F>(&self, ctor: F)
    where
        T: Registerable,
        F: crate::types::TransientCtorFallible<T>,
    {
        use crate::object_builder::AsyncTransientBuilderImplNoDeps;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "registering transient ({})",
            std::any::type_name::<T>()
        );

        let transient = Object::AsyncTransient(Box::new(
            AsyncTransientBuilderImplNoDeps::new(Box::new(ctor)
                as Box<
                    dyn crate::types::TransientCtorFallible<T> + Send + Sync,
                >),
        ));

        self.insert_or_panic::<T>(transient).await;
        self.validator.add_transient_no_deps::<T>();
    }

    /// Retrieves a newly constructed `T` from this registry.
    ///
    /// Returns `None` if `T` wasn't registered or failed to construct.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn transient<T>(&self) -> Result<T, ResolveError>
    where
        T: Registerable,
    {
        let lock = self.objects.read().await;
        if let Some(Object::AsyncTransient(ctor)) = lock.get(&TypeId::of::<T>())
        {
            let boxed = ctor.make_transient(self).await?;
            drop(lock);
            let obj = boxed.downcast::<T>().map_err(|_self| {
                ResolveError::Impl(ImplErrors::TypeMismatch)
            })?;
            return Ok(*obj);
        }

        #[allow(clippy::redundant_closure_for_method_calls)]
        if let Some(parent) =
            self.parent.as_ref().and_then(|weakref| weakref.upgrade())
        {
            Box::pin(async move { parent.transient::<T>().await }).await
        } else {
            Err(ResolveError::TypeMissing {
                typename: std::any::type_name::<T>(),
            })
        }
    }

    /// Retrieves the singleton `T` from this registry.
    ///
    /// Returns `None` if `T` wasn't registered or failed to construct. The
    /// singleton is a ref-counted pointer object (either `Arc` or `Rc`).
    ///
    /// # Errors
    /// Returns an error if the singleton fails to construct.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn singleton<T>(&self) -> Result<Ref<T>, ResolveError>
    where
        T: RegisterableSingleton,
    {
        let lock = self.objects.read().await;
        if let Some(Object::AsyncSingleton(singleton)) =
            lock.get(&TypeId::of::<T>())
        {
            let resolved = singleton.get_singleton(self).await?;
            drop(lock);
            let obj = resolved.downcast::<T>().map_err(|_self| {
                ResolveError::Impl(ImplErrors::TypeMismatch)
            })?;
            return Ok(obj);
        }

        #[allow(clippy::redundant_closure_for_method_calls)]
        if let Some(parent) =
            self.parent.as_ref().and_then(|weakref| weakref.upgrade())
        {
            Box::pin(async move { parent.singleton::<T>().await }).await
        } else {
            Err(ResolveError::TypeMissing {
                typename: std::any::type_name::<T>(),
            })
        }
    }

    /// Access the global registry.
    ///
    /// This registry contains the types that are marked for auto-registration
    /// via the derive macro.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn global() -> Ref<Self> {
        let ret = DEFAULT_REGISTRY
            .get_or_init(|| async move {
                let new = Self::autoregistered().await;
                Ref::new(new)
            })
            .await;
        Ref::clone(ret)
    }

    /// Reset the global registry, removing all previously registered types, and
    /// re-running the auto-registration routines.
    ///
    /// # Safety
    /// Ensure that no other thread is currently using [`Registry::global()`].
    #[allow(unsafe_code)]
    pub async unsafe fn reset_global() {
        // Purposefully not annotated with `tracing::instrument` because it mangles the order of
        // `async` and `unsafe`, resulting in a compiler error.
        let registry = Self::global().await;
        {
            let mut lock = registry.objects.write().await;
            lock.clear();
        }

        for register in inventory::iter::<RegistrationFunc> {
            (register.0)(&registry).await;
        }
    }

    /// Inserts a new object into the objecs hashtable.
    ///
    /// This acquires an exclusive lock on `self.objects`.
    ///
    /// # Panics
    /// If the key already exists (=> the type was previously registered).
    #[inline]
    async fn insert_or_panic<T: 'static>(&self, value: Object) {
        let mut lock = self.objects.write().await;
        let entry = lock.entry(TypeId::of::<T>());
        match entry {
            #[allow(clippy::panic)]
            hashbrown::hash_map::Entry::Occupied(_) => panic!(
                "Type '{}' ({:?}) is already registered",
                std::any::type_name::<T>(),
                TypeId::of::<T>()
            ),
            hashbrown::hash_map::Entry::Vacant(view) => {
                view.insert(value);
            }
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
    /// Reference to parent registry.
    registry: &'reg Registry,
    /// Marker for `T`.
    _marker: PhantomData<T>,
    /// Marker for `Deps`.
    _marker1: PhantomData<Deps>,
}

impl<
        T,
        #[cfg(not(feature = "tokio"))] Deps: DepBuilder<T> + 'static,
        #[cfg(feature = "tokio")] Deps: DepBuilder<T> + Send + Sync + 'static,
    > Builder<'_, T, Deps>
where
    T: Registerable,
{
    /// Register a new transient object, with dependencies specified in [`Registry::with_deps`].
    ///
    /// The `ctor` parameter is a constructor function returning the newly constructed `T`. The
    /// constructor accepts a single argument `Deps` (a tuple implementing
    /// [`crate::dependency_builder::DepBuilder`]). It's best to destructure the tuple to accept
    /// each dependency separately. This constructor will be called for every `T` that is
    /// requested.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use ferrunix_core::{Registry, Singleton, Transient};
    /// # let registry = Registry::empty();
    /// # struct Template {
    /// #     template: &'static str,
    /// # }
    /// registry
    ///     .with_deps::<_, (Transient<u8>, Singleton<Template>)>()
    ///     .register_transient(|(num, template)| {
    ///         // access `num` and `template` here.
    ///         u16::from(*num)
    ///     });
    /// ```
    ///
    /// For single dependencies, the destructured tuple needs to end with a
    /// comma: `(dep,)`.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg(not(feature = "tokio"))]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub fn register_transient<C>(&self, ctor: C)
    where
        C: TransientCtorDeps<T, Deps> + Copy + 'static,
    {
        self.try_register_transient::<_>(move |deps| -> Result<T, BoxErr> {
            Ok((ctor)(deps))
        });
    }

    /// Register a new transient object, with dependencies specified in [`Registry::with_deps`].
    ///
    /// The `ctor` parameter is a constructor function returning a `Result<T, E>`. The
    /// constructor accepts a single argument `Deps` (a tuple implementing
    /// [`crate::dependency_builder::DepBuilder`]). It's best to destructure the tuple to accept
    /// each dependency separately. This constructor will be called for every `T` that is
    /// requested.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use ferrunix_core::{Registry, Singleton, Transient};
    /// # let registry = Registry::empty();
    /// # struct Template {
    /// #     template: &'static str,
    /// # }
    /// registry
    ///     .with_deps::<_, (Transient<u8>, Singleton<Template>)>()
    ///     .register_transient(|(num, template)| {
    ///         // access `num` and `template` here.
    ///         u16::from(*num)
    ///     });
    /// ```
    ///
    /// For single dependencies, the destructured tuple needs to end with a
    /// comma: `(dep,)`.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg(not(feature = "tokio"))]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub fn try_register_transient<C>(&self, ctor: C)
    where
        C: TransientCtorFallibleDeps<T, Deps> + Copy + 'static,
    {
        use crate::object_builder::TransientBuilderImplWithDeps;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "registering transient (with dependencies) ({})",
            std::any::type_name::<T>()
        );

        let closure =
            Box::new(move |deps| -> Result<T, BoxErr> { (ctor)(deps) });
        let transient = Object::Transient(Box::new(
            TransientBuilderImplWithDeps::new(closure),
        ));

        self.registry.insert_or_panic::<T>(transient);
        self.registry.validator.add_transient_deps::<T, Deps>();
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
    /// The `ctor` is an `async` closure.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg(feature = "tokio")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub async fn register_transient<F>(&self, ctor: F)
    where
        F: crate::types::TransientCtorDeps<T, Deps> + Copy,
    {
        self.try_register_transient(move |deps| {
            Box::pin(async move {
                let obj = (ctor)(deps).await;
                Ok::<T, BoxErr>(obj)
            })
        })
        .await;
    }

    /// Try registering a new transient object, with dependencies specified in
    /// `.with_deps`.
    ///
    /// The `ctor` parameter is a constructor function returning a `Result<T>` of the newly
    /// constructed `T`. The constructor accepts a single argument `Deps` (a tuple implementing
    /// [`crate::dependency_builder::DepBuilder`]). It's best to destructure the tuple to accept
    /// each dependency separately. This constructor will be called for every `T` that is
    /// requested.
    ///
    /// The `ctor` is an `async` closure.
    ///
    /// # Panics
    /// When the type has been registered already.
    #[cfg(feature = "tokio")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub async fn try_register_transient<F>(&self, ctor: F)
    where
        F: crate::types::TransientCtorFallibleDeps<T, Deps>,
    {
        use crate::object_builder::AsyncTransientBuilderImplWithDeps;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "registering transient (with dependencies) ({})",
            std::any::type_name::<T>()
        );

        let closure = Box::new(ctor)
            as Box<
                dyn crate::types::TransientCtorFallibleDeps<T, Deps>
                    + Send
                    + Sync,
            >;

        let transient = Object::AsyncTransient(Box::new(
            AsyncTransientBuilderImplWithDeps::new(closure),
        ));

        self.registry.insert_or_panic::<T>(transient).await;
        self.registry.validator.add_transient_deps::<T, Deps>();
    }
}

impl<
        T,
        #[cfg(not(feature = "tokio"))] Deps: DepBuilder<T> + 'static,
        #[cfg(feature = "tokio")] Deps: DepBuilder<T> + Send + Sync + 'static,
    > Builder<'_, T, Deps>
where
    T: RegisterableSingleton,
{
    /// Register a new singleton object, with dependencies specified by
    /// [`Registry::with_deps`].
    ///
    /// The `ctor` parameter is a constructor function returning the newly constructed `T`. The
    /// constructor accepts a single argument `Deps` (a tuple implementing
    /// [`crate::dependency_builder::DepBuilder`]). It's best to destructure the tuple to accept
    /// each dependency separately. This constructor will be called once, lazily, when the first
    /// instance of `T` is requested.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use ferrunix_core::{Registry, Singleton, Transient};
    /// # let registry = Registry::empty();
    /// # struct Template {
    /// #     template: &'static str,
    /// # }
    /// registry
    ///     .with_deps::<_, (Transient<u8>, Singleton<Template>)>()
    ///     .register_transient(|(num, template)| {
    ///         // access `num` and `template` here.
    ///         u16::from(*num)
    ///     });
    /// ```
    ///
    /// For single dependencies, the destructured tuple needs to end with a
    /// comma: `(dep,)`.
    #[cfg(not(feature = "tokio"))]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub fn register_singleton<F>(&self, ctor: F)
    where
        F: SingletonCtorDeps<T, Deps>,
    {
        self.try_register_singleton::<_>(move |deps| -> Result<T, BoxErr> {
            Ok((ctor)(deps))
        });
    }

    /// Register a new singleton object, with dependencies specified by
    /// [`Registry::with_deps`].
    ///
    /// The `ctor` parameter is a constructor function returning a `Result<T, E>`. The constructor
    /// accepts a single argument `Deps` (a tuple implementing
    /// [`crate::dependency_builder::DepBuilder`]). It's best to destructure the tuple to accept
    /// each dependency separately. This constructor will be called once, lazily, when the first
    /// instance of `T` is requested.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use ferrunix_core::{Registry, Singleton, Transient};
    /// # let registry = Registry::empty();
    /// # struct Template {
    /// #     template: &'static str,
    /// # }
    /// registry
    ///     .with_deps::<_, (Transient<u8>, Singleton<Template>)>()
    ///     .register_transient(|(num, template)| {
    ///         // access `num` and `template` here.
    ///         u16::from(*num)
    ///     });
    /// ```
    ///
    /// For single dependencies, the destructured tuple needs to end with a comma: `(dep,)`.
    #[cfg(not(feature = "tokio"))]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub fn try_register_singleton<F>(&self, ctor: F)
    where
        F: SingletonCtorFallibleDeps<T, Deps>,
    {
        use crate::object_builder::SingletonGetterWithDeps;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "registering singleton (with dependencies) ({})",
            std::any::type_name::<T>()
        );

        let singleton =
            Object::Singleton(Box::new(SingletonGetterWithDeps::new(ctor)));

        self.registry.insert_or_panic::<T>(singleton);
        self.registry.validator.add_singleton_deps::<T, Deps>();
    }

    /// Register a new singleton object, with dependencies specified in
    /// [`Registry::with_deps`].
    ///
    /// The `ctor` parameter is a constructor function returning the newly constructed `T`. The
    /// constructor accepts a single argument `Deps` (a tuple implementing
    /// [`crate::dependency_builder::DepBuilder`]). It's best to destructure the tuple to accept
    /// each dependency separately. This constructor will be called once, lazily, when the first
    /// instance of `T` is requested.
    ///
    /// The `ctor` is an `async` closure.
    #[cfg(feature = "tokio")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub async fn register_singleton<F>(&self, ctor: F)
    where
        F: crate::types::SingletonCtorDeps<T, Deps>,
    {
        self.try_register_singleton(move |deps| {
            Box::pin(async move {
                let obj = (ctor)(deps).await;
                Ok::<_, BoxErr>(obj)
            })
        })
        .await;
    }

    /// Register a new singleton object, with dependencies specified in
    /// [`Registry::with_deps`].
    ///
    /// The `ctor` parameter is a constructor function returning a `Result<T>` with the newly
    /// constructed `T`. The constructor accepts a single argument `Deps` (a tuple implementing
    /// [`crate::dependency_builder::DepBuilder`]). It's best to destructure the tuple to accept
    /// each dependency separately. This constructor will be called once, lazily, when the first
    /// instance of `T` is requested.
    ///
    /// The `ctor` is an `async` closure.
    #[cfg(feature = "tokio")]
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(ctor)))]
    pub async fn try_register_singleton<F>(&self, ctor: F)
    where
        F: crate::types::SingletonCtorFallibleDeps<T, Deps>,
    {
        use crate::object_builder::AsyncSingletonWithDeps;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "registering singleton (with dependencies) ({})",
            std::any::type_name::<T>()
        );

        let closure = Box::new(ctor)
            as Box<
                dyn crate::types::SingletonCtorFallibleDeps<T, Deps>
                    + Send
                    + Sync,
            >;

        let singleton = Object::AsyncSingleton(Box::new(
            AsyncSingletonWithDeps::new(closure),
        ));

        self.registry.insert_or_panic::<T>(singleton).await;
        self.registry.validator.add_singleton_deps::<T, Deps>();
    }
}

impl<T, Dep> std::fmt::Debug for Builder<'_, T, Dep> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Builder").finish()
    }
}
