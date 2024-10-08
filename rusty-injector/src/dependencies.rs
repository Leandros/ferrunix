use std::any::TypeId;

use crate::{Arc, Registry};

pub trait Dep {
    fn new(registry: &Registry) -> Self;
    fn type_id() -> TypeId;
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
            inner: registry
                .get_transient::<T>()
                .expect("transient dependency must only be constructed if it's fulfillable"),
        }
    }

    fn type_id() -> TypeId {
        TypeId::of::<T>()
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
            inner: registry
                .get_singleton::<T>()
                .expect("singleton dependency must only be constructed if it's fulfillable"),
        }
    }

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}
