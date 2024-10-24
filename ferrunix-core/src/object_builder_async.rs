//! Abstraction layer to build transient and singleton dependencies, asynchronously.
use crate::dependency_builder::DepBuilder;
use crate::types::{BoxedAny, Ref, RefAny, Registerable, RegisterableSingleton};
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
#[async_trait::async_trait]
pub(crate) trait AsyncTransientBuilder {
    /// Constructs a new object; it may use the [`Registry`] to construct any
    /// dependencies.
    ///
    /// <div class="warning">It must not use the global registry.</div>
    ///
    /// May return `None` if the dependencies couldn't be fulfilled.
    async fn make_transient(&self, registry: &Registry) -> Option<BoxedAny>;
}

/// Trait to build a new object with singleton lifetime.
///
/// This trait is implemented twice, once to build objects without dependencies, and
/// once to build objects with any number of dependencies:
///
///   * [`AsyncSingletonNoDeps`]
///   * [`AsyncSingletonWithDeps`]
#[async_trait::async_trait]
pub(crate) trait AsyncSingleton {
    /// Constructs a new object; it may use the [`Registry`] to construct any
    /// dependencies.
    ///
    /// <div class="warning">It must not use the global registry.</div>
    ///
    /// May return `None` if the dependencies couldn't be fulfilled.
    async fn get_singleton(&self, registry: &Registry) -> Option<RefAny>;
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                   TRANSIENT (no deps)                   ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct a new transient with no dependencies. Usually used through `dyn AsyncTransientBuilder`.
pub(crate) struct AsyncTransientBuilderImplNoDeps<T> {
    /// Constructor, returns a boxed future to `T`.
    ctor:
        fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
}

impl<T> AsyncTransientBuilderImplNoDeps<T> {
    /// Create a new [`AsyncTransientBuilder`] using `ctor` to create new objects.
    ///
    /// `ctor` should not have side-effects. It may be called multiple times.
    pub(crate) fn new(
        ctor: fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) -> Self {
        Self { ctor }
    }
}

#[async_trait::async_trait]
impl<T> AsyncTransientBuilder for AsyncTransientBuilderImplNoDeps<T>
where
    Self: Send + Sync,
    T: Registerable,
{
    async fn make_transient(&self, _: &Registry) -> Option<BoxedAny> {
        let obj = (self.ctor)().await;
        Option::<BoxedAny>::Some(Box::new(obj))
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
    ctor: fn(
        Deps,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
}

impl<T, Deps> AsyncTransientBuilderImplWithDeps<T, Deps> {
    /// Create a new [`AsyncTransientBuilder`] using `ctor` to create new objects.
    ///
    /// `ctor` should not have side-effects. It may be called multiple times.
    pub(crate) fn new(
        ctor: fn(
            Deps,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) -> Self {
        Self { ctor }
    }
}

#[async_trait::async_trait]
impl<T, Deps> AsyncTransientBuilder
    for AsyncTransientBuilderImplWithDeps<T, Deps>
where
    Self: Send,
    Deps: DepBuilder<T> + 'static,
    T: Registerable,
{
    async fn make_transient(&self, registry: &Registry) -> Option<BoxedAny> {
        #[allow(clippy::option_if_let_else)]
        match Deps::build(
            registry,
            self.ctor,
            crate::dependency_builder::private::SealToken,
        )
        .await
        {
            Some(obj) => Some(Box::new(obj)),
            None => None,
        }
    }
}

//          ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//          ┃                   SINGLETON (no deps)                   ┃
//          ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

/// Construct, and returns, a new singleton with no dependencies. Usually used through `dyn
/// AsyncSingleton`.
pub(crate) struct AsyncSingletonNoDeps<T> {
    /// Constructor, returns a boxed future to `T`.
    ctor:
        fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
    /// Cell containing the constructed `T`.
    cell: ::tokio::sync::OnceCell<Ref<T>>,
}

impl<T> AsyncSingletonNoDeps<T> {
    /// Create a new [`SingletonGetter`] using `ctor` to create new objects.
    /// Objects are stored internally in `cell`.
    ///
    /// `ctor` may contain side-effects. It's guaranteed to be only called once (for each thread).
    pub(crate) fn new(
        ctor: fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) -> Self {
        Self {
            ctor,
            cell: ::tokio::sync::OnceCell::new(),
        }
    }
}

#[async_trait::async_trait]
impl<T> AsyncSingleton for AsyncSingletonNoDeps<T>
where
    Self: Send,
    T: RegisterableSingleton,
{
    async fn get_singleton(&self, _registry: &Registry) -> Option<RefAny> {
        let rc = self
            .cell
            .get_or_init(move || async move {
                let obj = (self.ctor)().await;
                Ref::new(obj)
            })
            .await;
        let rc = Ref::clone(rc) as RefAny;
        Option::<RefAny>::Some(rc)
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
    ctor: fn(
        Deps,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
    /// Cell containing the constructed `T`.
    cell: ::tokio::sync::OnceCell<Ref<T>>,
}

impl<T, Deps> AsyncSingletonWithDeps<T, Deps> {
    /// Create a new [`SingletonGetter`] using `ctor` to create new objects.
    /// Objects are stored internally in `cell`.
    ///
    /// `ctor` may contain side-effects. It's guaranteed to be only called once (for each thread).
    pub(crate) fn new(
        ctor: fn(
            Deps,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) -> Self {
        Self {
            ctor,
            cell: ::tokio::sync::OnceCell::new(),
        }
    }
}

#[async_trait::async_trait]
impl<T, Deps> AsyncSingleton for AsyncSingletonWithDeps<T, Deps>
where
    Self: Send,
    Deps: DepBuilder<T> + 'static,
    T: RegisterableSingleton,
{
    async fn get_singleton(&self, registry: &Registry) -> Option<RefAny> {
        #[allow(clippy::option_if_let_else)]
        match Deps::build(
            registry,
            self.ctor,
            crate::dependency_builder::private::SealToken,
        )
        .await
        {
            Some(obj) => {
                let rc = self
                    .cell
                    .get_or_init(move || async move { Ref::new(obj) })
                    .await;
                let rc = Ref::clone(rc) as RefAny;
                Option::<RefAny>::Some(rc)
            }
            None => None,
        }
    }
}
