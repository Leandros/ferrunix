use crate::{Arc, Registry};

pub trait Dep {
    fn new(registry: &Registry) -> Self;
}

pub struct Transient<T> {
    inner: T,
}

impl<T: Send + Sync + 'static> Transient<T> {
    pub fn get(self) -> T {
        self.inner
    }
}

impl<T: Send + Sync + 'static> Dep for Transient<T> {
    fn new(registry: &Registry) -> Self {
        Self {
            inner: registry.get_transient::<T>().unwrap(),
        }
    }
}

pub struct Singleton<T> {
    inner: Arc<T>,
}

impl<T: Send + Sync + 'static> Singleton<T> {
    pub fn get(self) -> Arc<T> {
        self.inner
    }
}

impl<T: Send + Sync + 'static> Dep for Singleton<T> {
    fn new(registry: &Registry) -> Self {
        Self {
            inner: registry.get_singleton::<T>().unwrap(),
        }
    }
}
