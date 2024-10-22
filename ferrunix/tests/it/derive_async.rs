// #![cfg(not(miri))]

use ferrunix::{Inject, RegistrationFunc, Registry};

#[derive(Inject)]
#[provides(transient)]
struct Empty {}

// #[automatically_derived]
// impl Empty {
//     #[allow(clippy::use_self)]
//     pub(crate) fn register<'reg>(
//         registry: &'reg ::ferrunix::Registry,
//     ) -> std::pin::Pin<
//         std::boxed::Box<dyn std::future::Future<Output = ()> + Send + 'reg>,
//     >
//     where
//         Self: Sync + 'static,
//     {
//         Box::pin(async move {
//             registry
//                 .transient(|| Box::pin(async move { Self {} }))
//                 .await;
//         })
//     }
// }

ferrunix::autoregister!(RegistrationFunc::new(Empty::register));

#[tokio::test]
async fn simple_derive() {
    let registry = Registry::autoregistered().await;

    let _obj = registry.get_transient::<Empty>().await.unwrap();
}
