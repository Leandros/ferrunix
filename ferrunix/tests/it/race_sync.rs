//! Singleton data race tests.
#![cfg(all(not(feature = "tokio"), feature = "multithread"))]

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
                pub(super) fn register(registry: &ferrunix::Registry) {
                     registry.register_singleton(|| {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                         $base::default()
                     });
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
                pub(super) fn register(registry: &ferrunix::Registry) {
                    registry
                        .with_deps::<_, ($(
                            ferrunix::Singleton<$deps>,
                        )*)>()
                        .register_singleton(|($(
                            [<_ $deps>],
                        )*)| {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            $base {$(
                            [<_ $deps>]: [<_ $deps>].get(),
                        )*}});
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

#[test]
#[allow(clippy::too_many_lines)]
fn test_singleton_race() {
    use std::sync::Arc;

    let registry = Arc::new(Registry::empty());

    TySingleton0::register(&registry);
    TySingleton1::register(&registry);
    TySingleton2::register(&registry);
    TySingleton3::register(&registry);

    TySingleton00::register(&registry);
    TySingleton01::register(&registry);
    TySingleton02::register(&registry);
    TySingleton03::register(&registry);
    TySingleton04::register(&registry);
    TySingleton05::register(&registry);
    TySingleton06::register(&registry);
    TySingleton07::register(&registry);
    TySingleton08::register(&registry);
    TySingleton09::register(&registry);
    TySingleton10::register(&registry);

    let handle0 = std::thread::spawn({
        let registry = Arc::clone(&registry);
        move || {
            let singleton00 = registry.singleton::<TySingleton00>().unwrap();
            assert!(singleton00.is_true());
            let singleton01 = registry.singleton::<TySingleton01>().unwrap();
            assert!(singleton01.is_true());
            let singleton02 = registry.singleton::<TySingleton02>().unwrap();
            assert!(singleton02.is_true());
            let singleton03 = registry.singleton::<TySingleton03>().unwrap();
            assert!(singleton03.is_true());
            let singleton04 = registry.singleton::<TySingleton04>().unwrap();
            assert!(singleton04.is_true());
            let singleton05 = registry.singleton::<TySingleton05>().unwrap();
            assert!(singleton05.is_true());
            let singleton06 = registry.singleton::<TySingleton06>().unwrap();
            assert!(singleton06.is_true());
            let singleton07 = registry.singleton::<TySingleton07>().unwrap();
            assert!(singleton07.is_true());
            let singleton08 = registry.singleton::<TySingleton08>().unwrap();
            assert!(singleton08.is_true());
            let singleton09 = registry.singleton::<TySingleton09>().unwrap();
            assert!(singleton09.is_true());
            let singleton10 = registry.singleton::<TySingleton10>().unwrap();
            assert!(singleton10.is_true());

            let _singleton0 = registry.singleton::<TySingleton0>().unwrap();
            let _singleton1 = registry.singleton::<TySingleton1>().unwrap();
            let _singleton2 = registry.singleton::<TySingleton2>().unwrap();
            let _singleton3 = registry.singleton::<TySingleton3>().unwrap();
        }
    });

    let handle1 = std::thread::spawn({
        let registry = Arc::clone(&registry);
        move || {
            let singleton00 = registry.singleton::<TySingleton00>().unwrap();
            assert!(singleton00.is_true());
            let singleton01 = registry.singleton::<TySingleton01>().unwrap();
            assert!(singleton01.is_true());
            let singleton02 = registry.singleton::<TySingleton02>().unwrap();
            assert!(singleton02.is_true());
            let singleton03 = registry.singleton::<TySingleton03>().unwrap();
            assert!(singleton03.is_true());
            let singleton04 = registry.singleton::<TySingleton04>().unwrap();
            assert!(singleton04.is_true());
            let singleton05 = registry.singleton::<TySingleton05>().unwrap();
            assert!(singleton05.is_true());
            let singleton06 = registry.singleton::<TySingleton06>().unwrap();
            assert!(singleton06.is_true());
            let singleton07 = registry.singleton::<TySingleton07>().unwrap();
            assert!(singleton07.is_true());
            let singleton08 = registry.singleton::<TySingleton08>().unwrap();
            assert!(singleton08.is_true());
            let singleton09 = registry.singleton::<TySingleton09>().unwrap();
            assert!(singleton09.is_true());
            let singleton10 = registry.singleton::<TySingleton10>().unwrap();
            assert!(singleton10.is_true());

            let _singleton0 = registry.singleton::<TySingleton0>().unwrap();
            let _singleton1 = registry.singleton::<TySingleton1>().unwrap();
            let _singleton2 = registry.singleton::<TySingleton2>().unwrap();
            let _singleton3 = registry.singleton::<TySingleton3>().unwrap();
        }
    });

    let handle2 = std::thread::spawn({
        let registry = Arc::clone(&registry);
        move || {
            let singleton00 = registry.singleton::<TySingleton00>().unwrap();
            assert!(singleton00.is_true());
            let singleton01 = registry.singleton::<TySingleton01>().unwrap();
            assert!(singleton01.is_true());
            let singleton02 = registry.singleton::<TySingleton02>().unwrap();
            assert!(singleton02.is_true());
            let singleton03 = registry.singleton::<TySingleton03>().unwrap();
            assert!(singleton03.is_true());
            let singleton04 = registry.singleton::<TySingleton04>().unwrap();
            assert!(singleton04.is_true());
            let singleton05 = registry.singleton::<TySingleton05>().unwrap();
            assert!(singleton05.is_true());
            let singleton06 = registry.singleton::<TySingleton06>().unwrap();
            assert!(singleton06.is_true());
            let singleton07 = registry.singleton::<TySingleton07>().unwrap();
            assert!(singleton07.is_true());
            let singleton08 = registry.singleton::<TySingleton08>().unwrap();
            assert!(singleton08.is_true());
            let singleton09 = registry.singleton::<TySingleton09>().unwrap();
            assert!(singleton09.is_true());
            let singleton10 = registry.singleton::<TySingleton10>().unwrap();
            assert!(singleton10.is_true());

            let _singleton0 = registry.singleton::<TySingleton0>().unwrap();
            let _singleton1 = registry.singleton::<TySingleton1>().unwrap();
            let _singleton2 = registry.singleton::<TySingleton2>().unwrap();
            let _singleton3 = registry.singleton::<TySingleton3>().unwrap();
        }
    });

    let handle3 = std::thread::spawn({
        let registry = Arc::clone(&registry);
        move || {
            let singleton00 = registry.singleton::<TySingleton00>().unwrap();
            assert!(singleton00.is_true());
            let singleton01 = registry.singleton::<TySingleton01>().unwrap();
            assert!(singleton01.is_true());
            let singleton02 = registry.singleton::<TySingleton02>().unwrap();
            assert!(singleton02.is_true());
            let singleton03 = registry.singleton::<TySingleton03>().unwrap();
            assert!(singleton03.is_true());
            let singleton04 = registry.singleton::<TySingleton04>().unwrap();
            assert!(singleton04.is_true());
            let singleton05 = registry.singleton::<TySingleton05>().unwrap();
            assert!(singleton05.is_true());
            let singleton06 = registry.singleton::<TySingleton06>().unwrap();
            assert!(singleton06.is_true());
            let singleton07 = registry.singleton::<TySingleton07>().unwrap();
            assert!(singleton07.is_true());
            let singleton08 = registry.singleton::<TySingleton08>().unwrap();
            assert!(singleton08.is_true());
            let singleton09 = registry.singleton::<TySingleton09>().unwrap();
            assert!(singleton09.is_true());
            let singleton10 = registry.singleton::<TySingleton10>().unwrap();
            assert!(singleton10.is_true());

            let _singleton0 = registry.singleton::<TySingleton0>().unwrap();
            let _singleton1 = registry.singleton::<TySingleton1>().unwrap();
            let _singleton2 = registry.singleton::<TySingleton2>().unwrap();
            let _singleton3 = registry.singleton::<TySingleton3>().unwrap();
        }
    });

    handle0.join().unwrap();
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
}
