use crate::error::ResolveError;
use crate::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, Registry, RwLock,
    RwLockReadGuard, RwLockWriteGuard,
};

#[derive(Debug)]
pub struct LazyTransient<T> {
    inner: RwLock<Option<T>>,
}

impl<T> Default for LazyTransient<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T> LazyTransient<T>
where
    T: Send + Sync + 'static,
{
    pub fn resolved() -> Self {
        Self::resolved_with(Registry::global())
    }

    pub fn resolved_with(registry: &Registry) -> Self {
        registry
            .get_transient::<T>()
            .map(|inner| Self {
                inner: RwLock::new(Some(inner)),
            })
            .expect("dependency or transient not registered")
    }

    pub fn resolve(&self) -> Result<(), ResolveError> {
        self.resolve_with(Registry::global())
    }

    pub fn resolve_with(
        &self,
        registry: &Registry,
    ) -> Result<(), ResolveError> {
        match self.inner.try_write() {
            Some(mut lockguard) => match registry.get_transient::<T>() {
                Some(obj) => {
                    *lockguard = Some(obj);
                    Ok(())
                }

                None => Err(ResolveError::DependenciesMissing),
            },

            None => Err(ResolveError::LockAcquire),
        }
    }

    pub fn get(&self) -> MappedRwLockReadGuard<'_, T> {
        if self.inner.read().is_none() {
            self.resolve().expect("Deref for LazyTransient<T>");
        }

        RwLockReadGuard::map(self.inner.read(), |el| {
            el.as_ref().expect(
                "value guaranteed due to resolve above. a panic here is a bug",
            )
        })
    }

    pub fn get_mut(&mut self) -> MappedRwLockWriteGuard<'_, T> {
        if self.inner.read().is_none() {
            self.resolve().expect("Deref for LazyTransient<T>");
        }

        RwLockWriteGuard::map(self.inner.write(), |el| {
            el.as_mut().expect(
                "value guaranteed due to resolve above. a panic here is a bug",
            )
        })
    }
}
