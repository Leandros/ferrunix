#![cfg(not(feature = "tokio"))]
#![allow(unused)]

use ferrunix::{Registry, Transient};

mod broken {
    pub(crate) struct TypeZero {
        pub(crate) dep0: Box<Dep0>,
    }

    pub(crate) struct Dep0 {
        pub(crate) dep1: Box<Dep1>,
    }

    pub(crate) struct Dep1 {
        pub(crate) dep2: Box<Dep2>,
    }

    pub(crate) struct Dep2 {
        pub(crate) dep0: Box<Dep0>,
    }

    pub(crate) struct Dep3 {
        pub(crate) dep_missing: Box<DepMissing>,
    }

    pub(crate) struct DepMissing {}
}

mod fine {
    pub(crate) struct TypeZero {
        pub(crate) dep0: Box<Dep0>,
    }

    pub(crate) struct Dep0 {
        pub(crate) dep1: Box<Dep1>,
    }

    pub(crate) struct Dep1 {
        pub(crate) dep2: Box<Dep2>,
    }

    pub(crate) struct Dep2 {}
}

#[test]
fn detect_cycle() {
    use broken::*;

    let registry = Registry::empty();
    registry
        .with_deps::<_, (Transient<Dep0>,)>()
        .register_transient(|(dep0,)| TypeZero {
            dep0: Box::new(dep0.get()),
        });

    registry
        .with_deps::<_, (Transient<Dep1>,)>()
        .register_transient(|(dep1,)| Dep0 {
            dep1: Box::new(dep1.get()),
        });

    registry
        .with_deps::<_, (Transient<Dep2>,)>()
        .register_transient(|(dep2,)| Dep1 {
            dep2: Box::new(dep2.get()),
        });

    registry
        .with_deps::<_, (Transient<Dep0>,)>()
        .register_transient(|(dep0,)| Dep2 {
            dep0: Box::new(dep0.get()),
        });

    assert!(registry.validate::<TypeZero>().is_err());
    assert!(registry.validate_all().is_err());
    assert!(registry.validate_all_full().is_err());
}

#[test]
fn detect_missing() {
    use broken::*;

    let registry = Registry::empty();
    registry
        .with_deps::<_, (Transient<Dep0>,)>()
        .register_transient(|(dep0,)| TypeZero {
            dep0: Box::new(dep0.get()),
        });

    registry
        .with_deps::<_, (Transient<Dep1>,)>()
        .register_transient(|(dep1,)| Dep0 {
            dep1: Box::new(dep1.get()),
        });

    registry
        .with_deps::<_, (Transient<Dep2>,)>()
        .register_transient(|(dep2,)| Dep1 {
            dep2: Box::new(dep2.get()),
        });

    registry
        .with_deps::<_, (Transient<DepMissing>,)>()
        .register_transient(|(dep_missing,)| Dep3 {
            dep_missing: Box::new(dep_missing.get()),
        });

    assert!(registry.validate::<TypeZero>().is_err());
    assert!(registry.validate_all().is_err());
    assert!(registry.validate_all_full().is_err());
}

#[test]
fn all_fine() {
    use fine::*;

    let registry = Registry::empty();
    registry
        .with_deps::<_, (Transient<Dep0>,)>()
        .register_transient(|(dep0,)| TypeZero {
            dep0: Box::new(dep0.get()),
        });

    registry
        .with_deps::<_, (Transient<Dep1>,)>()
        .register_transient(|(dep1,)| Dep0 {
            dep1: Box::new(dep1.get()),
        });

    registry
        .with_deps::<_, (Transient<Dep2>,)>()
        .register_transient(|(dep2,)| Dep1 {
            dep2: Box::new(dep2.get()),
        });

    registry.register_transient(|| Dep2 {});

    registry.validate::<TypeZero>().unwrap();
    registry.validate_all().unwrap();
    registry.validate_all_full().unwrap();
}
