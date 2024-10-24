//! Abstraction layer to build transient and singleton dependencies.
use crate::dependency_builder::DepBuilder;
use crate::types::{BoxedAny, OnceCell, Ref, RefAny, Registerable, RegisterableSingleton};
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
    fn make_transient(&self, registry: &Registry) -> Option<BoxedAny>;
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
    fn get_singleton(&self, registry: &Registry) -> Option<RefAny>;
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                   TRANSIENT (no deps)                   ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct a new transient with no dependencies. Usually used through `dyn TransientBuilder`.
pub(crate) struct TransientBuilderImplNoDeps<T> {
    /// Constructor, returns a new `T`.
    ctor: fn() -> T,
}

impl<T> TransientBuilderImplNoDeps<T> {
    /// Create a new [`TransientBuilder`] using `ctor` to create new objects.
    ///
    /// `ctor` should not have side-effects. It may be called multiple times.
    pub(crate) fn new(ctor: fn() -> T) -> Self {
        Self { ctor }
    }
}

impl<T> TransientBuilder for TransientBuilderImplNoDeps<T>
where
    T: Registerable,
{
    fn make_transient(&self, _registry: &Registry) -> Option<BoxedAny> {
        let obj = (self.ctor)();
        Some(Box::new(obj))
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
    ctor: fn(Deps) -> T,
}

impl<T, Deps> TransientBuilderImplWithDeps<T, Deps> {
    /// Create a new [`TransientBuilder`] using `ctor` to create new objects.
    ///
    /// `ctor` should not have side-effects. It may be called multiple times.
    pub(crate) fn new(ctor: fn(Deps) -> T) -> Self {
        Self { ctor }
    }
}

impl<T, Deps> TransientBuilder for TransientBuilderImplWithDeps<T, Deps>
where
    Deps: DepBuilder<T> + 'static,
    T: Registerable,
{
    fn make_transient(&self, registry: &Registry) -> Option<BoxedAny> {
        #[allow(clippy::option_if_let_else)]
        match Deps::build(
            registry,
            self.ctor,
            crate::dependency_builder::private::SealToken,
        ) {
            Some(obj) => Some(Box::new(obj)),
            None => None,
        }
    }
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                   SINGLETON (no deps)                   ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct, and returns, a new singleton with no dependencies. Usually used through `dyn
/// SingletonGetter`.
pub(crate) struct SingletonGetterNoDeps<T> {
    /// Constructor, returns a new `T`.
    ctor: fn() -> T,
    /// Cell containing the constructed `T`.
    cell: OnceCell<Ref<T>>,
}

impl<T> SingletonGetterNoDeps<T> {
    /// Create a new [`SingletonGetter`] using `ctor` to create new objects.
    /// Objects are stored internally in `cell`.
    ///
    /// `ctor` may contain side-effects. It's guaranteed to be only called once (for each thread).
    pub(crate) fn new(ctor: fn() -> T) -> Self {
        Self {
            ctor,
            cell: OnceCell::new(),
        }
    }
}

impl<T> SingletonGetter for SingletonGetterNoDeps<T>
where
    T: RegisterableSingleton,
{
    fn get_singleton(&self, _registry: &Registry) -> Option<RefAny> {
        let rc = self.cell.get_or_init(|| Ref::new((self.ctor)()));
        let rc = Ref::clone(rc) as RefAny;
        Some(rc)
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
    ctor: fn(Deps) -> T,
    /// Cell containing the constructed `T`.
    cell: OnceCell<Ref<T>>,
}

impl<T, Deps> SingletonGetterWithDeps<T, Deps> {
    /// Create a new [`SingletonGetter`] using `ctor` to create new objects.
    /// Objects are stored internally in `cell`.
    ///
    /// `ctor` may contain side-effects. It's guaranteed to be only called once (for each thread).
    pub(crate) fn new(ctor: fn(Deps) -> T) -> Self {
        Self {
            ctor,
            cell: OnceCell::new(),
        }
    }
}

impl<T, Deps> SingletonGetter for SingletonGetterWithDeps<T, Deps>
where
    Deps: DepBuilder<T> + 'static,
    T: RegisterableSingleton,
{
    fn get_singleton(&self, registry: &Registry) -> Option<RefAny> {
        #[allow(clippy::option_if_let_else)]
        match Deps::build(
            registry,
            self.ctor,
            crate::dependency_builder::private::SealToken,
        ) {
            Some(obj) => {
                let rc = self.cell.get_or_init(|| Ref::new(obj));
                let rc = Ref::clone(rc) as RefAny;
                Some(rc)
            }
            None => None,
        }
    }
}
