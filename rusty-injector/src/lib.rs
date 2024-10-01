use std::sync::atomic::AtomicBool;

use once_cell::sync::OnceCell;

static DEFAULT_REGISTRY: OnceCell<RwLock<Registry>> = OnceCell::new();
static AUTOREGISTERED: AtomicBool = AtomicBool::new(false);

pub struct RegistrationFunc(pub fn(&mut Registry));

inventory::collect!(RegistrationFunc);

pub type RwLock<T> = parking_lot::RwLock<T>;
pub type MappedRwLockReadGuard<'a, T> = parking_lot::MappedRwLockReadGuard<'a, T>;
pub type MappedRwLockWriteGuard<'a, T> = parking_lot::MappedRwLockWriteGuard<'a, T>;
pub type RwLockReadGuard<'a, T> = parking_lot::RwLockReadGuard<'a, T>;
pub type RwLockWriteGuard<'a, T> = parking_lot::RwLockWriteGuard<'a, T>;

pub type Arc<T> = std::sync::Arc<T>;
pub type HashMap<K, V> = std::collections::HashMap<K, V>;

pub mod dependencies;
pub mod dependency_builder;
pub mod error;
pub mod lazy_singleton;
pub mod lazy_transient;
pub mod registry;

pub use dependencies::Singleton;
pub use dependencies::Transient;
pub use registry::Registry;

pub use inventory::submit as inventory_submit;

#[cfg(test)]
mod tests {
    use self::dependencies::{Singleton, Transient};

    use super::*;

    trait Env {
        fn get(&self, var: &str) -> Option<String>;
    }

    struct RealEnv {}
    impl Env for RealEnv {
        fn get(&self, var: &str) -> Option<String> {
            std::env::var(var).ok()
        }
    }

    struct MockEnv {}
    impl Env for MockEnv {
        fn get(&self, _var: &str) -> Option<String> {
            Some("TEST".to_owned())
        }
    }

    #[test]
    fn test_new() {
        let mut registry = Registry::new();
        registry.transient(|| 1_u8);

        let x = registry.get_transient::<u8>();
        assert_eq!(x, Some(1_u8));

        registry
            .with_deps::<_, (Transient<u8>,)>()
            .transient(|(i,)| {
                let i = i.get();
                u16::from(i) + 1_u16
            });

        let x1 = registry.get_transient::<u16>();
        assert_eq!(x1, Some(2_u16));

        registry
            .with_deps::<_, (Transient<u8>, Transient<u16>)>()
            .transient(|(i, j)| {
                let i = i.get();
                let j = j.get();
                u32::from(i) + u32::from(j) + 1_u32
            });

        let x2 = registry.get_transient::<u32>();
        assert_eq!(x2, Some(4_u32));

        registry.make_current().unwrap();

        Registry::current_mut().transient(|| -1_i8);
        let x3 = Registry::current().get_transient::<i8>();
        assert_eq!(x3, Some(-1_i8));

        Registry::current_mut().transient::<Box<dyn Env + Send + Sync>>(|| Box::new(MockEnv {}));
        let env = Registry::current().get_transient::<Box<dyn Env + Send + Sync>>();
        assert!(env.is_some());
        assert_eq!(env.unwrap().get("anything"), Some("TEST".to_owned()));
    }

    #[test]
    fn singletons_without_deps() {
        let mut registry = Registry::new();
        registry.transient(|| 1_u8);
        registry.transient(|| 1_u16);
        registry.transient(|| 1_u32);
        registry.singleton(8_i8);
        registry.singleton(16_i16);
        registry.singleton(32_i32);

        let x1 = registry.get_singleton::<i8>();
        assert_eq!(*x1.unwrap(), 8_i8);
        let x2 = registry.get_singleton::<i16>();
        assert_eq!(*x2.unwrap(), 16_i16);
        let x3 = registry.get_singleton::<i32>();
        assert_eq!(*x3.unwrap(), 32_i32);
    }

    #[test]
    fn singletons_with_deps() {
        let mut registry = Registry::new();
        registry.transient(|| 1_u8);
        registry.singleton(8_i8);

        registry
            .with_deps::<_, (Transient<u8>, Singleton<i8>)>()
            .singleton(|(i, j)| {
                let i = i.get();
                let j = j.get();
                i32::from(i) + i32::from(*j)
            });

        let x1 = registry.get_transient::<u8>();
        assert_eq!(x1.unwrap(), 1_u8);
        let x2 = registry.get_singleton::<i8>();
        assert_eq!(*x2.unwrap(), 8_i8);
        let x3 = registry.get_singleton::<i32>();
        assert_eq!(*x3.unwrap(), 9_i32);
    }
}
