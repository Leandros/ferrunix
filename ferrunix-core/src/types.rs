//! Type aliases.
#![allow(
    clippy::single_char_lifetime_names,
    clippy::missing_docs_in_private_items,
    dead_code
)]

pub(crate) type OnceCell<T> = once_cell::sync::OnceCell<T>;
pub(crate) type RwLock<T> = parking_lot::RwLock<T>;
pub(crate) type MappedRwLockReadGuard<'a, T> =
    parking_lot::MappedRwLockReadGuard<'a, T>;
pub(crate) type MappedRwLockWriteGuard<'a, T> =
    parking_lot::MappedRwLockWriteGuard<'a, T>;
pub(crate) type RwLockReadGuard<'a, T> = parking_lot::RwLockReadGuard<'a, T>;
pub(crate) type RwLockWriteGuard<'a, T> = parking_lot::RwLockWriteGuard<'a, T>;

pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V>;

pub type Ref<T> = std::sync::Arc<T>;
