//! Abstraction layer to build transient and singleton dependencies, asynchronously.

use crate::dependency_builder::DepBuilder;
use crate::error::ResolveError;
use crate::types::{
    BoxFuture, BoxedAny, Ref, RefAny, Registerable, RegisterableSingleton,
    RwLock,
};
use crate::Registry;

/// Trait to build a new object with transient lifetime.
///
/// This trait is implemented twice, once to build objects without dependencies, and
/// once to build objects with any number of dependencies:
///
///   * [`AsyncTransientBuilderImplNoDeps`]
///   * [`AsyncTransientBuilderImplWithDeps`]
///
/// This is an `async` trait.
pub(crate) trait AsyncTransientBuilder {
    /// Constructs a new object; it may use the [`Registry`] to construct any
    /// dependencies.
    ///
    /// <div class="warning">It must not use the global registry.</div>
    ///
    /// May return `None` if the dependencies couldn't be fulfilled.
    fn make_transient<'this>(
        &'this self,
        registry: &'this Registry,
    ) -> BoxFuture<'this, Result<BoxedAny, ResolveError>>;
}

/// Trait to build a new object with singleton lifetime.
///
/// This trait is implemented twice, once to build objects without dependencies, and
/// once to build objects with any number of dependencies:
///
///   * [`AsyncSingletonNoDeps`]
///   * [`AsyncSingletonWithDeps`]
pub(crate) trait AsyncSingleton {
    /// Constructs a new object; it may use the [`Registry`] to construct any
    /// dependencies.
    ///
    /// <div class="warning">It must not use the global registry.</div>
    ///
    /// May return `None` if the dependencies couldn't be fulfilled.
    fn get_singleton<'this>(
        &'this self,
        registry: &'this Registry,
    ) -> BoxFuture<'this, Result<RefAny, ResolveError>>;
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                   TRANSIENT (no deps)                   ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct a new transient with no dependencies. Usually used through `dyn AsyncTransientBuilder`.
pub(crate) struct AsyncTransientBuilderImplNoDeps<T> {
    /// Constructor, returns a boxed future to `T`.
    ctor: Box<dyn crate::types::TransientCtorFallible<T> + Send + Sync>,
}

impl<T> AsyncTransientBuilderImplNoDeps<T> {
    /// Create a new [`AsyncTransientBuilder`] using `ctor` to create new objects.
    ///
    /// `ctor` should not have side-effects. It may be called multiple times.
    pub(crate) fn new(
        ctor: Box<dyn crate::types::TransientCtorFallible<T> + Send + Sync>,
    ) -> Self {
        Self { ctor }
    }
}

impl<T> AsyncTransientBuilder for AsyncTransientBuilderImplNoDeps<T>
where
    Self: Send,
    T: Registerable,
{
    fn make_transient<'this>(
        &'this self,
        _: &'this Registry,
    ) -> BoxFuture<'this, Result<BoxedAny, ResolveError>> {
        Box::pin(async move {
            let obj = (self.ctor)().await.map_err(ResolveError::Ctor)?;
            Ok::<BoxedAny, ResolveError>(Box::new(obj))
        })
    }
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                  TRANSIENT (with deps)                  ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct a new transient with any number of dependencies. Usually used through `dyn
/// TransientBuilder`.
///
/// The dependency tuple `Deps` must implement [`DepBuilder<T>`].
pub(crate) struct AsyncTransientBuilderImplWithDeps<T, Deps> {
    /// Constructor, returns a boxed future to `T`.
    ctor: Box<
        dyn crate::types::TransientCtorFallibleDeps<T, Deps>
            + Send
            + Sync
            + 'static,
    >,
}

impl<T, Deps> AsyncTransientBuilderImplWithDeps<T, Deps> {
    /// Create a new [`AsyncTransientBuilder`] using `ctor` to create new objects.
    ///
    /// `ctor` should not have side-effects. It may be called multiple times.
    pub(crate) fn new(
        ctor: Box<
            dyn crate::types::TransientCtorFallibleDeps<T, Deps>
                + Send
                + Sync
                + 'static,
        >,
    ) -> Self {
        Self { ctor }
    }
}

impl<T, Deps> AsyncTransientBuilder
    for AsyncTransientBuilderImplWithDeps<T, Deps>
where
    Self: Send,
    Deps: DepBuilder<T> + Send + 'static,
    T: Registerable,
{
    fn make_transient<'this>(
        &'this self,
        registry: &'this Registry,
    ) -> BoxFuture<'this, Result<BoxedAny, ResolveError>> {
        Box::pin(async move {
            let obj = Deps::build(
                registry,
                &*self.ctor,
                crate::dependency_builder::private::SealToken,
            )
            .await?;

            Ok::<BoxedAny, ResolveError>(Box::new(obj))
        })
    }
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                   SINGLETON (no deps)                   ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct, and returns, a new singleton with no dependencies. Usually used through `dyn
/// AsyncSingleton`.
pub(crate) struct AsyncSingletonNoDeps<T> {
    /// Constructor, returns a boxed future to `T`.
    ctor: RwLock<
        Option<Box<dyn crate::types::SingletonCtorFallible<T> + Send + Sync>>,
    >,
    /// Cell containing the constructed `T`.
    cell: ::tokio::sync::OnceCell<Ref<T>>,
}

impl<T> AsyncSingletonNoDeps<T> {
    /// Create a new [`SingletonGetter`] using `ctor` to create new objects.
    /// Objects are stored internally in `cell`.
    ///
    /// `ctor` may contain side-effects. It's guaranteed to be only called once (for each thread).
    pub(crate) fn new<F>(ctor: F) -> Self
    where
        F: crate::types::SingletonCtorFallible<T> + Send + Sync + 'static,
    {
        Self {
            ctor: RwLock::new(Some(Box::new(ctor))),
            cell: ::tokio::sync::OnceCell::new(),
        }
    }
}

impl<T> AsyncSingleton for AsyncSingletonNoDeps<T>
where
    Self: Send,
    T: RegisterableSingleton,
{
    fn get_singleton<'this>(
        &'this self,
        _registry: &'this Registry,
    ) -> BoxFuture<'this, Result<RefAny, ResolveError>> {
        Box::pin(async move {
            let rc = self
                .cell
                .get_or_try_init(move || async move {
                    let ctor = {
                        let mut lock = self.ctor.write().await;
                        lock.take().expect("to be called only once")
                    };
                    let obj = (ctor)().await.map_err(ResolveError::Ctor)?;
                    Ok::<_, ResolveError>(Ref::new(obj))
                })
                .await?;
            let rc = Ref::clone(rc) as RefAny;
            Ok(rc)
        })
    }
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                  SINGLETON (with deps)                  ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct a new singleton with any number of dependencies. Usually used through `dyn
/// AsyncSingleton`.
///
/// The dependency tuple `Deps` must implement [`DepBuilder<T>`].
pub(crate) struct AsyncSingletonWithDeps<T, Deps> {
    /// Constructor, returns a boxed future to `T`.
    ctor: RwLock<
        Option<
            Box<
                dyn crate::types::SingletonCtorFallibleDeps<T, Deps>
                    + Send
                    + Sync,
            >,
        >,
    >,
    /// Cell containing the constructed `T`.
    cell: ::tokio::sync::OnceCell<Ref<T>>,
}

impl<T, Deps> AsyncSingletonWithDeps<T, Deps> {
    /// Create a new [`SingletonGetter`] using `ctor` to create new objects.
    /// Objects are stored internally in `cell`.
    ///
    /// `ctor` may contain side-effects. It's guaranteed to be only called once (for each thread).
    pub(crate) fn new<F>(ctor: F) -> Self
    where
        F: crate::types::SingletonCtorFallibleDeps<T, Deps>
            + Send
            + Sync
            + 'static,
    {
        Self {
            ctor: RwLock::new(Some(Box::new(ctor))),
            cell: ::tokio::sync::OnceCell::new(),
        }
    }
}

impl<T, Deps> AsyncSingleton for AsyncSingletonWithDeps<T, Deps>
where
    Self: Send,
    Deps: DepBuilder<T> + Send + 'static,
    T: RegisterableSingleton,
{
    fn get_singleton<'this>(
        &'this self,
        registry: &'this Registry,
    ) -> BoxFuture<'this, Result<RefAny, ResolveError>> {
        Box::pin(async move {
            let ctor = {
                let mut lock = self.ctor.write().await;
                lock.take().expect("to be called only once")
            };

            let obj = Deps::build_once(
                registry,
                ctor,
                crate::dependency_builder::private::SealToken,
            )
            .await?;

            let rc = self
                .cell
                .get_or_init(move || async move { Ref::new(obj) })
                .await;
            let rc = Ref::clone(rc) as RefAny;
            Ok(rc)
        })
    }
}
