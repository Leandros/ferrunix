use once_cell::sync::OnceCell;

static DEFAULT_REGISTRY: OnceCell<Registry> = OnceCell::new();

pub struct RegistrationFunc(pub fn(&Registry));

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
