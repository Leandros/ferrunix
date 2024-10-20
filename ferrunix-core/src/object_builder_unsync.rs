use crate::dependency_builder::DepBuilder;
use crate::types::{BoxedAny, OnceCell, Ref, RefAny, Registerable};
use crate::Registry;

pub(crate) trait TransientBuilder {
    fn make_transient(&self, registry: &Registry) -> Option<BoxedAny>;
}

pub(crate) trait SingletonGetter {
    fn get_singleton(&self, registry: &Registry) -> Option<RefAny>;
}

// --- TRANSIENT (no deps)
// ----------------------------------------------------------------------------
pub(crate) struct TransientBuilderImplNoDeps<T> {
    ctor: fn() -> T,
}

impl<T> TransientBuilderImplNoDeps<T> {
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

// --- TRANSIENT (with deps)
// ----------------------------------------------------------------------------
pub(crate) struct TransientBuilderImplWithDeps<T, Deps> {
    ctor: fn(Deps) -> T,
}

impl<T, Deps> TransientBuilderImplWithDeps<T, Deps> {
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

// --- SINGLETON (no deps)
// ----------------------------------------------------------------------------
pub(crate) struct SingletonGetterNoDeps<T> {
    ctor: fn() -> T,
    cell: OnceCell<Ref<T>>,
}

impl<T> SingletonGetterNoDeps<T> {
    pub(crate) fn new(ctor: fn() -> T) -> Self {
        Self {
            ctor,
            cell: OnceCell::new(),
        }
    }
}

impl<T> SingletonGetter for SingletonGetterNoDeps<T>
where
    T: Registerable,
{
    fn get_singleton(&self, _registry: &Registry) -> Option<RefAny> {
        let rc = self.cell.get_or_init(|| Ref::new((self.ctor)()));
        let rc = Ref::clone(rc) as RefAny;
        Some(rc)
    }
}

// --- SINGLETON (with deps)
// ----------------------------------------------------------------------------
pub(crate) struct SingletonGetterWithDeps<T, Deps> {
    ctor: fn(Deps) -> T,
    cell: OnceCell<Ref<T>>,
}

impl<T, Deps> SingletonGetterWithDeps<T, Deps> {
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
    T: Registerable,
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
