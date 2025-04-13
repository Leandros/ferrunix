#![allow(clippy::no_effect)]
// use std::cell::RefCell;
// use std::ffi::c_void;

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
// #[cfg(feature = "multithread")]
// fn multithread_requires_no_send_types() {
// // struct NotSync {
// //     s: RefCell<String>,
// // }

// // struct NotSyncNotSend {
// //     s: *mut c_void,
// // }

//     // fn is_send<T: Send>() {}
//     // fn is_sync<T: Sync>() {}
//     // fn is_send_sync<T: Send + Sync>() {}

//     let registry = Registry::empty();

//     registry.transient(|| NotSync {
//         s: RefCell::new("Hello".to_owned()),
//     });

//     let _notsync = registry.transient::<NotSync>().unwrap();
// }

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
