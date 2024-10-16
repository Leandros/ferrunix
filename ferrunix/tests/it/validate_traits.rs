#![allow(clippy::no_effect)]
use ferrunix::registry::Builder;
use ferrunix::{Registry, Singleton, Transient};

struct IsDebug<T: std::fmt::Debug>(Option<T>);

#[test]
fn all_public_types_are_debug() {
    IsDebug::<Transient<u32>>(None);
    IsDebug::<Singleton<u32>>(None);
    IsDebug::<Registry>(None);
    IsDebug::<Builder<'static, u32, ()>>(None);
}
