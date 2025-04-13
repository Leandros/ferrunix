//! Abstraction layer to build transient and singleton dependencies.
use crate::dependency_builder::DepBuilder;
use crate::error::ResolveError;
use crate::types::{
    BoxedAny, OnceCell, Ref, RefAny, Registerable, RegisterableSingleton,
    RwLock, SingletonCtorFallibleDeps, SingletonCtorFallible, TransientCtorFallible,
    TransientCtorFallibleDeps,
};
use crate::Registry;

/// Trait to build a new object with transient lifetime.
///
/// This trait is implemented twice, once to build objects without dependencies, and
/// once to build objects with any number of dependencies:
///
///   * [`TransientBuilderImplNoDeps`]
///   * [`TransientBuilderImplWithDeps`]
pub(crate) trait TransientBuilder {
    /// Constructs a new object; it may use the [`Registry`] to construct any
    /// dependencies.
    ///
    /// <div class="warning">It must not use the global registry.</div>
    ///
    /// May return `None` if the dependencies couldn't be fulfilled.
    fn make_transient(
        &self,
        registry: &Registry,
    ) -> Result<BoxedAny, ResolveError>;
}

/// Trait to build a new object with singleton lifetime.
///
/// This trait is implemented twice, once to build objects without dependencies, and
/// once to build objects with any number of dependencies:
///
///   * [`SingletonGetterNoDeps`]
///   * [`SingletonGetterWithDeps`]
pub(crate) trait SingletonGetter {
    /// Constructs a new object; it may use the [`Registry`] to construct any
    /// dependencies.
    ///
    /// <div class="warning">It must not use the global registry.</div>
    ///
    /// May return `None` if the dependencies couldn't be fulfilled.
    fn get_singleton(
        &self,
        registry: &Registry,
    ) -> Result<RefAny, ResolveError>;
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                   TRANSIENT (no deps)                   ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct a new transient with no dependencies. Usually used through `dyn TransientBuilder`.
pub(crate) struct TransientBuilderImplNoDeps<T> {
    /// Constructor, returns a new `T`.
    ctor: Box<dyn TransientCtorFallible<T>>,
}

impl<T> TransientBuilderImplNoDeps<T> {
    /// Create a new [`TransientBuilder`] using `ctor` to create new objects.
    ///
    /// `ctor` should not have side-effects. It may be called multiple times.
    pub(crate) fn new(ctor: Box<dyn TransientCtorFallible<T>>) -> Self {
        Self { ctor }
    }
}

impl<T> TransientBuilder for TransientBuilderImplNoDeps<T>
where
    T: Registerable,
{
    fn make_transient(
        &self,
        _registry: &Registry,
    ) -> Result<BoxedAny, ResolveError> {
        let obj = (self.ctor)().map_err(ResolveError::Ctor)?;
        Ok(Box::new(obj))
    }
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                  TRANSIENT (with deps)                  ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct a new transient with any number of dependencies. Usually used through `dyn
/// TransientBuilder`.
///
/// The dependency tuple `Deps` must implement [`DepBuilder<T>`].
pub(crate) struct TransientBuilderImplWithDeps<T, Deps> {
    /// Constructor, returns a new `T`.
    ctor: Box<dyn TransientCtorFallibleDeps<T, Deps>>,
}

impl<T, Deps> TransientBuilderImplWithDeps<T, Deps> {
    /// Create a new [`TransientBuilder`] using `ctor` to create new objects.
    ///
    /// `ctor` should not have side-effects. It may be called multiple times.
    pub(crate) fn new(
        ctor: Box<dyn TransientCtorFallibleDeps<T, Deps>>,
    ) -> Self {
        Self { ctor }
    }
}

impl<T, Deps> TransientBuilder for TransientBuilderImplWithDeps<T, Deps>
where
    Deps: DepBuilder<T> + 'static,
    T: Registerable,
{
    fn make_transient(
        &self,
        registry: &Registry,
    ) -> Result<BoxedAny, ResolveError> {
        #[allow(clippy::option_if_let_else)]
        let obj = Deps::build(
            registry,
            &*self.ctor,
            crate::dependency_builder::private::SealToken,
        )?;

        Ok(Box::new(obj))
    }
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                   SINGLETON (no deps)                   ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct, and returns, a new singleton with no dependencies. Usually used through `dyn
/// SingletonGetter`.
pub(crate) struct SingletonGetterNoDeps<T> {
    /// Constructor, returns a new `T`.
    ctor: RwLock<Option<Box<dyn SingletonCtorFallible<T>>>>,
    /// Cell containing the constructed `T`.
    cell: OnceCell<Ref<T>>,
}

impl<T> SingletonGetterNoDeps<T> {
    /// Create a new [`SingletonGetter`] using `ctor` to create new objects.
    /// Objects are stored internally in `cell`.
    ///
    /// `ctor` may contain side-effects. It's guaranteed to be only called once (for each thread).
    pub(crate) fn new<F>(ctor: F) -> Self
    where
        F: SingletonCtorFallible<T>,
    {
        Self {
            ctor: RwLock::new(Some(Box::new(ctor))),
            cell: OnceCell::new(),
        }
    }
}

impl<T> SingletonGetter for SingletonGetterNoDeps<T>
where
    T: RegisterableSingleton,
{
    fn get_singleton(
        &self,
        _registry: &Registry,
    ) -> Result<RefAny, ResolveError> {
        let rc = self.cell.get_or_try_init(|| {
            let ctor = {
                let mut lock = self.ctor.write();
                lock.take().expect("to be called only once")
            };
            let obj = (ctor)().map_err(ResolveError::Ctor)?;
            Ok::<_, ResolveError>(Ref::new(obj))
        })?;
        let rc = Ref::clone(rc) as RefAny;
        Ok(rc)
    }
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                  SINGLETON (with deps)                  ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct a new singleton with any number of dependencies. Usually used through `dyn
/// SingletonGetter`.
///
/// The dependency tuple `Deps` must implement [`DepBuilder<T>`].
pub(crate) struct SingletonGetterWithDeps<T, Deps> {
    /// Constructor, returns a new `T`.
    ctor: RwLock<Option<Box<dyn SingletonCtorFallibleDeps<T, Deps>>>>,
    /// Cell containing the constructed `T`.
    cell: OnceCell<Ref<T>>,
}

impl<T, Deps> SingletonGetterWithDeps<T, Deps> {
    /// Create a new [`SingletonGetter`] using `ctor` to create new objects.
    /// Objects are stored internally in `cell`.
    ///
    /// `ctor` may contain side-effects. It's guaranteed to be only called once (for each thread).
    pub(crate) fn new<F>(ctor: F) -> Self
    where
        F: SingletonCtorFallibleDeps<T, Deps>,
    {
        Self {
            ctor: RwLock::new(Some(Box::new(ctor))),
            cell: OnceCell::new(),
        }
    }
}

impl<T, Deps> SingletonGetter for SingletonGetterWithDeps<T, Deps>
where
    Deps: DepBuilder<T> + 'static,
    T: RegisterableSingleton,
{
    fn get_singleton(
        &self,
        registry: &Registry,
    ) -> Result<RefAny, ResolveError> {
        let ctor = {
            let mut lock = self.ctor.write();
            lock.take().expect("to be called only once")
        };

        let obj = Deps::build_once(
            registry,
            ctor,
            crate::dependency_builder::private::SealToken,
        )?;

        let rc = self.cell.get_or_init(|| Ref::new(obj));
        let rc = Ref::clone(rc) as RefAny;
        Ok(rc)
    }
}
