use crate::dependency_builder::DepBuilder;
use crate::types::{BoxedAny, Ref, RefAny, Registerable};
use crate::Registry;

#[async_trait::async_trait]
pub(crate) trait AsyncTransientBuilder {
    async fn make_transient(&self, registry: &Registry) -> Option<BoxedAny>;
}

#[async_trait::async_trait]
pub(crate) trait AsyncSingleton {
    async fn get_singleton(&self, registry: &Registry) -> Option<RefAny>;
}

pub(crate) struct AsyncTransientBuilderImplNoDeps<T, Deps> {
    ctor:
        fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
    _marker: std::marker::PhantomData<Deps>,
}

impl<T> AsyncTransientBuilderImplNoDeps<T, ()> {
    pub(crate) fn new(
        ctor: fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = T> + Send>,
        >,
    ) -> Self {
        Self {
            ctor,
            _marker: std::marker::PhantomData::<()>,
        }
    }
}

#[async_trait::async_trait]
impl<T, Deps> AsyncTransientBuilder for AsyncTransientBuilderImplNoDeps<T, Deps>
where
    Self: Send + Sync,
    Deps: DepBuilder<T> + 'static,
    T: Registerable,
{
    async fn make_transient(&self, _: &Registry) -> Option<BoxedAny> {
        let obj = (self.ctor)().await;
        Option::<BoxedAny>::Some(Box::new(obj))
    }
}

pub(crate) struct AsyncTransientBuilderImplWithDeps<T, Deps> {
    ctor: fn(
        Deps,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
}

impl<T, Deps> AsyncTransientBuilderImplWithDeps<T, Deps> {
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

pub(crate) struct AsyncSingletonNoDeps<T> {
    ctor:
        fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
    cell: ::tokio::sync::OnceCell<Ref<T>>,
}

impl<T> AsyncSingletonNoDeps<T> {
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
    T: Registerable,
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

pub(crate) struct AsyncSingletonWithDeps<T, Deps> {
    ctor: fn(
        Deps,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
    cell: ::tokio::sync::OnceCell<Ref<T>>,
}

impl<T, Deps> AsyncSingletonWithDeps<T, Deps> {
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
    T: Registerable,
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
