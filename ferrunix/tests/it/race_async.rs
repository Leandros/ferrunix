//! Singleton data race tests.
#![cfg(all(feature = "tokio", feature = "multithread"))]

use ferrunix::Registry;

macro_rules! make_singleton_type {
    ($base:ident) => {
        paste::paste! {
            #[allow(non_snake_case)]
            #[derive(Debug, Default)]
            pub(super) struct $base {}

            impl $base {
                #[allow(unused)]
                pub fn is_true(&self) -> bool {
                    true
                }

                #[allow(non_snake_case)]
                pub(super) async fn register(registry: &ferrunix::Registry) {
                     registry.register_singleton(|| Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                         $base::default()
                     })).await;
                }
            }
        }
    };

    ($base:ident, $($deps:ident),*) => {
        paste::paste! {
            #[allow(non_snake_case)]
            #[derive(Debug, Default)]
            pub(super) struct $base {
                $(
                    pub(super) [<_ $deps>]: ferrunix::Ref<$deps>,
                )*
            }

            impl $base {
                #[allow(non_snake_case)]
                pub(super) async fn register(registry: &ferrunix::Registry) {
                    registry
                        .with_deps::<_, ($(
                            ferrunix::Singleton<$deps>,
                        )*)>()
                        .register_singleton(|($(
                            [<_ $deps>],
                        )*)| Box::pin(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            $base {$(
                            [<_ $deps>]: [<_ $deps>].get(),
                        )*}})).await;
                }
            }
        }
    };
}

make_singleton_type!(TySingleton0);
make_singleton_type!(TySingleton1, TySingleton0);
make_singleton_type!(TySingleton2, TySingleton0, TySingleton1);
make_singleton_type!(TySingleton3, TySingleton0, TySingleton1, TySingleton2);
make_singleton_type!(TySingleton00);
make_singleton_type!(TySingleton01);
make_singleton_type!(TySingleton02);
make_singleton_type!(TySingleton03);
make_singleton_type!(TySingleton04);
make_singleton_type!(TySingleton05);
make_singleton_type!(TySingleton06);
make_singleton_type!(TySingleton07);
make_singleton_type!(TySingleton08);
make_singleton_type!(TySingleton09);
make_singleton_type!(TySingleton10);

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(clippy::too_many_lines)]
async fn test_singleton_race() {
    use std::sync::Arc;

    let registry = Arc::new(Registry::empty());

    TySingleton0::register(&registry).await;
    TySingleton1::register(&registry).await;
    TySingleton2::register(&registry).await;
    TySingleton3::register(&registry).await;

    TySingleton00::register(&registry).await;
    TySingleton01::register(&registry).await;
    TySingleton02::register(&registry).await;
    TySingleton03::register(&registry).await;
    TySingleton04::register(&registry).await;
    TySingleton05::register(&registry).await;
    TySingleton06::register(&registry).await;
    TySingleton07::register(&registry).await;
    TySingleton08::register(&registry).await;
    TySingleton09::register(&registry).await;
    TySingleton10::register(&registry).await;

    let handle0 = tokio::spawn({
        let registry = Arc::clone(&registry);
        async move {
            let singleton00 = registry.singleton::<TySingleton00>().await.unwrap();
            assert!(singleton00.is_true());
            let singleton01 = registry.singleton::<TySingleton01>().await.unwrap();
            assert!(singleton01.is_true());
            let singleton02 = registry.singleton::<TySingleton02>().await.unwrap();
            assert!(singleton02.is_true());
            let singleton03 = registry.singleton::<TySingleton03>().await.unwrap();
            assert!(singleton03.is_true());
            let singleton04 = registry.singleton::<TySingleton04>().await.unwrap();
            assert!(singleton04.is_true());
            let singleton05 = registry.singleton::<TySingleton05>().await.unwrap();
            assert!(singleton05.is_true());
            let singleton06 = registry.singleton::<TySingleton06>().await.unwrap();
            assert!(singleton06.is_true());
            let singleton07 = registry.singleton::<TySingleton07>().await.unwrap();
            assert!(singleton07.is_true());
            let singleton08 = registry.singleton::<TySingleton08>().await.unwrap();
            assert!(singleton08.is_true());
            let singleton09 = registry.singleton::<TySingleton09>().await.unwrap();
            assert!(singleton09.is_true());
            let singleton10 = registry.singleton::<TySingleton10>().await.unwrap();
            assert!(singleton10.is_true());

            let _singleton0 = registry.singleton::<TySingleton0>().await.unwrap();
            let _singleton1 = registry.singleton::<TySingleton1>().await.unwrap();
            let _singleton2 = registry.singleton::<TySingleton2>().await.unwrap();
            let _singleton3 = registry.singleton::<TySingleton3>().await.unwrap();
        }
    });

    let handle1 = tokio::spawn({
        let registry = Arc::clone(&registry);
        async move {
            let singleton00 = registry.singleton::<TySingleton00>().await.unwrap();
            assert!(singleton00.is_true());
            let singleton01 = registry.singleton::<TySingleton01>().await.unwrap();
            assert!(singleton01.is_true());
            let singleton02 = registry.singleton::<TySingleton02>().await.unwrap();
            assert!(singleton02.is_true());
            let singleton03 = registry.singleton::<TySingleton03>().await.unwrap();
            assert!(singleton03.is_true());
            let singleton04 = registry.singleton::<TySingleton04>().await.unwrap();
            assert!(singleton04.is_true());
            let singleton05 = registry.singleton::<TySingleton05>().await.unwrap();
            assert!(singleton05.is_true());
            let singleton06 = registry.singleton::<TySingleton06>().await.unwrap();
            assert!(singleton06.is_true());
            let singleton07 = registry.singleton::<TySingleton07>().await.unwrap();
            assert!(singleton07.is_true());
            let singleton08 = registry.singleton::<TySingleton08>().await.unwrap();
            assert!(singleton08.is_true());
            let singleton09 = registry.singleton::<TySingleton09>().await.unwrap();
            assert!(singleton09.is_true());
            let singleton10 = registry.singleton::<TySingleton10>().await.unwrap();
            assert!(singleton10.is_true());

            let _singleton0 = registry.singleton::<TySingleton0>().await.unwrap();
            let _singleton1 = registry.singleton::<TySingleton1>().await.unwrap();
            let _singleton2 = registry.singleton::<TySingleton2>().await.unwrap();
            let _singleton3 = registry.singleton::<TySingleton3>().await.unwrap();
        }
    });

    let handle2 = tokio::spawn({
        let registry = Arc::clone(&registry);
        async move {
            let singleton00 = registry.singleton::<TySingleton00>().await.unwrap();
            assert!(singleton00.is_true());
            let singleton01 = registry.singleton::<TySingleton01>().await.unwrap();
            assert!(singleton01.is_true());
            let singleton02 = registry.singleton::<TySingleton02>().await.unwrap();
            assert!(singleton02.is_true());
            let singleton03 = registry.singleton::<TySingleton03>().await.unwrap();
            assert!(singleton03.is_true());
            let singleton04 = registry.singleton::<TySingleton04>().await.unwrap();
            assert!(singleton04.is_true());
            let singleton05 = registry.singleton::<TySingleton05>().await.unwrap();
            assert!(singleton05.is_true());
            let singleton06 = registry.singleton::<TySingleton06>().await.unwrap();
            assert!(singleton06.is_true());
            let singleton07 = registry.singleton::<TySingleton07>().await.unwrap();
            assert!(singleton07.is_true());
            let singleton08 = registry.singleton::<TySingleton08>().await.unwrap();
            assert!(singleton08.is_true());
            let singleton09 = registry.singleton::<TySingleton09>().await.unwrap();
            assert!(singleton09.is_true());
            let singleton10 = registry.singleton::<TySingleton10>().await.unwrap();
            assert!(singleton10.is_true());

            let _singleton0 = registry.singleton::<TySingleton0>().await.unwrap();
            let _singleton1 = registry.singleton::<TySingleton1>().await.unwrap();
            let _singleton2 = registry.singleton::<TySingleton2>().await.unwrap();
            let _singleton3 = registry.singleton::<TySingleton3>().await.unwrap();
        }
    });

    let handle3 = tokio::spawn({
        let registry = Arc::clone(&registry);
        async move {
            let singleton00 = registry.singleton::<TySingleton00>().await.unwrap();
            assert!(singleton00.is_true());
            let singleton01 = registry.singleton::<TySingleton01>().await.unwrap();
            assert!(singleton01.is_true());
            let singleton02 = registry.singleton::<TySingleton02>().await.unwrap();
            assert!(singleton02.is_true());
            let singleton03 = registry.singleton::<TySingleton03>().await.unwrap();
            assert!(singleton03.is_true());
            let singleton04 = registry.singleton::<TySingleton04>().await.unwrap();
            assert!(singleton04.is_true());
            let singleton05 = registry.singleton::<TySingleton05>().await.unwrap();
            assert!(singleton05.is_true());
            let singleton06 = registry.singleton::<TySingleton06>().await.unwrap();
            assert!(singleton06.is_true());
            let singleton07 = registry.singleton::<TySingleton07>().await.unwrap();
            assert!(singleton07.is_true());
            let singleton08 = registry.singleton::<TySingleton08>().await.unwrap();
            assert!(singleton08.is_true());
            let singleton09 = registry.singleton::<TySingleton09>().await.unwrap();
            assert!(singleton09.is_true());
            let singleton10 = registry.singleton::<TySingleton10>().await.unwrap();
            assert!(singleton10.is_true());

            let _singleton0 = registry.singleton::<TySingleton0>().await.unwrap();
            let _singleton1 = registry.singleton::<TySingleton1>().await.unwrap();
            let _singleton2 = registry.singleton::<TySingleton2>().await.unwrap();
            let _singleton3 = registry.singleton::<TySingleton3>().await.unwrap();
        }
    });

    handle0.await.unwrap();
    handle1.await.unwrap();
    handle2.await.unwrap();
    handle3.await.unwrap();
}
