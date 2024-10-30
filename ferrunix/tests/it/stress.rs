#![cfg(all(feature = "multithread", not(feature = "tokio")))]
use std::sync::Arc;

use ferrunix::Registry;

macro_rules! make_type {
    ($base:ident) => {
        paste! {
            #[allow(non_snake_case)]
            #[derive(Debug, Default)]
            pub(super) struct $base {}

            impl $base {
                #[allow(non_snake_case)]
                pub(super) fn register(registry: &ferrunix::Registry) {
                     registry.transient(|| $base::default());
                }
            }
        }
    };

    ($base:ident, $($deps:ident),*) => {
        paste! {
            #[allow(non_snake_case)]
            #[derive(Debug, Default)]
            pub(super) struct $base {
                $(
                    pub(super) [<_ $deps>]: Box<$deps>,
                )*
            }

            impl $base {
                #[allow(non_snake_case)]
                pub(super) fn register(registry: &ferrunix::Registry) {
                    registry
                        .with_deps::<_, ($(
                            ferrunix::Transient<$deps>,
                        )*)>()
                        .transient(|($(
                            [<_ $deps>],
                        )*)| $base {$(
                            [<_ $deps>]: Box::new([<_ $deps>].get()),
                        )*});
                }
            }
        }
    };
}

macro_rules! make_many_types {
    ($modname:ident) => {
        mod $modname {
            use paste::paste;

            make_type!(Config);
            make_type!(TypeZero, Dep0);
            make_type!(Dep0, Dep1);
            make_type!(Dep1, Dep2);
            make_type!(Dep2, Dep3, TypeNoDeps0, TypeNoDeps1);
            make_type!(Dep3, Dep4);
            make_type!(Dep4, Dep5, Config);
            make_type!(Dep5, Dep6, Config);
            make_type!(Dep6, Dep7);
            make_type!(Dep7, Dep8);
            make_type!(Dep8, Dep9);
            make_type!(Dep9);

            make_type!(TypeNoDeps0);
            make_type!(TypeNoDeps1);
            make_type!(TypeNoDeps2);
            make_type!(TypeNoDeps3);
            make_type!(TypeNoDeps4);
            make_type!(TypeNoDeps5);
            make_type!(TypeNoDeps6);
            make_type!(TypeNoDeps7);
            make_type!(TypeNoDeps8);
            make_type!(TypeNoDeps9);
            make_type!(TypeNoDeps10);

            make_type!(
                TypeManyDeps0,
                TypeNoDeps0,
                TypeNoDeps1,
                TypeNoDeps2,
                TypeNoDeps3,
                TypeNoDeps4,
                TypeNoDeps5
            );

            make_type!(
                TypeManyDeps1,
                TypeNoDeps6,
                TypeNoDeps7,
                TypeNoDeps8,
                TypeNoDeps9,
                TypeNoDeps10
            );

            make_type!(TypeSingleDep0, Config);
            make_type!(TypeSingleDep1, Config);
            make_type!(TypeSingleDep2, Config);
            make_type!(TypeSingleDep3, Config);
            make_type!(TypeSingleDep4, Config);
            make_type!(TypeSingleDep5, Config);
            make_type!(TypeSingleDep6, Config);
            make_type!(TypeSingleDep7, Config);
            make_type!(TypeSingleDep8, Config);
            make_type!(TypeSingleDep9, Config);
            make_type!(TypeSingleDep10, Config);
        }
    };
}

macro_rules! register_all_types {
    ($modname:ident, $reg:ident) => {
        $modname::Config::register(&$reg);
        $modname::TypeZero::register(&$reg);
        $modname::Dep0::register(&$reg);
        $modname::Dep1::register(&$reg);
        $modname::Dep2::register(&$reg);
        $modname::Dep3::register(&$reg);
        $modname::Dep4::register(&$reg);
        $modname::Dep5::register(&$reg);
        $modname::Dep6::register(&$reg);
        $modname::Dep7::register(&$reg);
        $modname::Dep8::register(&$reg);
        $modname::Dep9::register(&$reg);

        $modname::TypeNoDeps0::register(&$reg);
        $modname::TypeNoDeps1::register(&$reg);
        $modname::TypeNoDeps2::register(&$reg);
        $modname::TypeNoDeps3::register(&$reg);
        $modname::TypeNoDeps4::register(&$reg);
        $modname::TypeNoDeps5::register(&$reg);
        $modname::TypeNoDeps6::register(&$reg);
        $modname::TypeNoDeps7::register(&$reg);
        $modname::TypeNoDeps8::register(&$reg);
        $modname::TypeNoDeps9::register(&$reg);
        $modname::TypeNoDeps10::register(&$reg);

        $modname::TypeManyDeps0::register(&$reg);
        $modname::TypeManyDeps1::register(&$reg);

        $modname::TypeSingleDep0::register(&$reg);
        $modname::TypeSingleDep1::register(&$reg);
        $modname::TypeSingleDep2::register(&$reg);
        $modname::TypeSingleDep3::register(&$reg);
        $modname::TypeSingleDep4::register(&$reg);
        $modname::TypeSingleDep5::register(&$reg);
        $modname::TypeSingleDep6::register(&$reg);
        $modname::TypeSingleDep7::register(&$reg);
        $modname::TypeSingleDep8::register(&$reg);
        $modname::TypeSingleDep9::register(&$reg);
        $modname::TypeSingleDep10::register(&$reg);

        // Error ignored, because it might fail when some other thread is in
        // between adding types.
        #[allow(clippy::let_underscore_must_use)]
        let _ = $reg.validate_all();
    };
}

make_many_types!(manytypes0);
make_many_types!(manytypes1);
make_many_types!(manytypes2);
make_many_types!(manytypes3);
make_many_types!(manytypes4);
make_many_types!(manytypes5);
make_many_types!(manytypes6);
make_many_types!(manytypes7);
make_many_types!(manytypes8);
make_many_types!(manytypes9);

#[test]
fn stress_registration() {
    let registry = Arc::new(Registry::empty());

    let handle0 = {
        let registry = Arc::clone(&registry);
        std::thread::spawn(move || {
            register_all_types!(manytypes0, registry);
            register_all_types!(manytypes1, registry);
            register_all_types!(manytypes2, registry);
        })
    };
    let handle1 = {
        let registry = Arc::clone(&registry);
        std::thread::spawn(move || {
            register_all_types!(manytypes3, registry);
            register_all_types!(manytypes4, registry);
            register_all_types!(manytypes5, registry);
        })
    };
    let handle2 = {
        let registry = Arc::clone(&registry);
        std::thread::spawn(move || {
            register_all_types!(manytypes6, registry);
            register_all_types!(manytypes7, registry);
            register_all_types!(manytypes8, registry);
        })
    };
    let handle3 = {
        let registry = Arc::clone(&registry);
        std::thread::spawn(move || {
            register_all_types!(manytypes9, registry);
        })
    };

    handle0.join().unwrap();
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();

    registry.validate_all_full().unwrap();
    registry.validate_all().unwrap();
    // println!("{}", registry.dotgraph().unwrap());
}
