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

// #[test]
// fn is_send_sync() {
//     fn is_send<T: Send>() {}
//     fn is_sync<T: Sync>() {}
//     is_send::<Registry>();
//     is_sync::<Registry>();
//     is_send::<Transient<u32>>();
//     is_sync::<Transient<u32>>();
//     is_send::<Singleton<u32>>();
//     is_sync::<Singleton<u32>>();
//     is_send::<Builder<'static, u32, ()>>();
//     is_sync::<Builder<'static, u32, ()>>();
// }
