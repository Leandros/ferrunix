#![allow(clippy::unwrap_used, dead_code)]

use ferrunix::{Registry, Singleton, Transient};

#[test]
fn simple_registry_concrete_types() {
    let registry = Registry::empty();
    registry.transient(|| 1_u8);
    registry.singleton(|| String::from("Hello, World"));

    registry
        .with_deps::<_, (Transient<u8>,)>()
        .transient(|(i,)| {
            let i = i.get();
            u16::from(i) + 1_u16
        });

    registry
        .with_deps::<_, (Transient<u8>, Transient<u16>)>()
        .transient(|(i, j)| {
            let i = i.get();
            let j = j.get();
            u32::from(i) + u32::from(j) + 1_u32
        });

    registry.validate_all().unwrap();

    let x = registry.get_transient::<u8>();
    assert_eq!(x, Some(1_u8));

    let x1 = registry.get_transient::<u16>();
    assert_eq!(x1, Some(2_u16));

    let x2 = registry.get_transient::<u32>();
    assert_eq!(x2, Some(4_u32));

    let s1 = registry.get_singleton::<String>().unwrap();
    assert_eq!(&*s1, &"Hello, World".to_owned());
}

#[test]
fn singletons_without_deps() {
    let registry = Registry::empty();
    registry.transient(|| 1_u8);
    registry.transient(|| 1_u16);
    registry.transient(|| 1_u32);
    registry.singleton(|| 8_i8);
    registry.singleton(|| 16_i16);
    registry.singleton(|| 32_i32);

    registry.validate_all().unwrap();

    let x1 = registry.get_singleton::<i8>();
    assert_eq!(*x1.unwrap(), 8_i8);
    let x2 = registry.get_singleton::<i16>();
    assert_eq!(*x2.unwrap(), 16_i16);
    let x3 = registry.get_singleton::<i32>();
    assert_eq!(*x3.unwrap(), 32_i32);
}

#[test]
fn singletons_with_deps() {
    let registry = Registry::empty();
    registry.transient(|| 1_u8);
    registry.singleton(|| 8_i8);

    registry
        .with_deps::<_, (Transient<u8>, Singleton<i8>)>()
        .singleton(|(i, j)| {
            let i = i.get();
            let j = j.get();
            i32::from(i) + i32::from(*j)
        });

    registry.validate_all().unwrap();

    let x1 = registry.get_transient::<u8>();
    assert_eq!(x1.unwrap(), 1_u8);
    let x2 = registry.get_singleton::<i8>();
    assert_eq!(*x2.unwrap(), 8_i8);
    let x3 = registry.get_singleton::<i32>();
    assert_eq!(*x3.unwrap(), 9_i32);
}

#[test]
fn validate_failure_missing_dependencies() {
    let registry = Registry::empty();

    registry
        .with_deps::<_, (Transient<u8>,)>()
        .transient(|(i,)| {
            let i = i.get();
            u16::from(i) + 1_u16
        });

    registry
        .with_deps::<_, (Transient<u8>, Transient<u16>)>()
        .transient(|(i, j)| {
            let i = i.get();
            let j = j.get();
            u32::from(i) + u32::from(j) + 1_u32
        });

    assert!(
        registry.validate_all().is_err(),
        "should fail due to missing u8 dependency"
    );

    let x1 = registry.get_transient::<u16>();
    assert_eq!(x1, None);

    let x2 = registry.get_transient::<u32>();
    assert_eq!(x2, None);

    let s1 = registry.get_singleton::<String>();
    assert_eq!(s1, None);
}

#[test]
#[should_panic]
#[allow(clippy::should_panic_without_expect)]
fn panic_when_registered_twice_diff() {
    let registry = Registry::empty();
    registry.transient(|| 1_u8);
    registry.singleton(|| 1_u8);
}

#[test]
#[should_panic]
#[allow(clippy::should_panic_without_expect)]
fn panic_when_registered_twice_transient() {
    let registry = Registry::empty();
    registry.transient(|| 1_u8);
    registry.transient(|| 1_u8);
}

#[test]
#[should_panic]
#[allow(clippy::should_panic_without_expect)]
fn panic_when_registered_twice_singleton() {
    let registry = Registry::empty();
    registry.singleton(|| 1_u8);
    registry.singleton(|| 1_u8);
}

#[derive(Debug)]
struct NotClone {
    inner: String,
}

#[test]
fn register_not_clone() {
    let registry = Registry::empty();
    registry.transient(|| NotClone {
        inner: String::new(),
    });

    let _not_clone = registry.get_transient::<NotClone>().unwrap();
}

struct TupleWithStatic(&'static str);

#[test]
fn register_static_lifetime() {
    let registry = Registry::empty();
    registry.transient(|| TupleWithStatic("TEST"));
}
