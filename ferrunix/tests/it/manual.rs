#![allow(clippy::unwrap_used, dead_code)]

use ferrunix::{Registry, Singleton, Transient};

#[test]
fn simple_registry_concrete_types() {
    let registry = Registry::empty();
    // todo!("make `transient` accept a non-fallible function, which is internally converted to a fallible function");
    registry.register_transient(|| 1_u8);
    registry.register_singleton(|| String::from("Hello, World"));

    registry
        .with_deps::<_, (Transient<u8>,)>()
        .register_transient(|(i,)| {
            let i = i.get();
            u16::from(i) + 1_u16
        });

    registry
        .with_deps::<_, (Transient<u8>, Transient<u16>)>()
        .register_transient(|(i, j)| {
            let i = i.get();
            let j = j.get();
            u32::from(i) + u32::from(j) + 1_u32
        });

    registry.validate_all().unwrap();

    let x = registry.get_transient::<u8>();
    assert_eq!(x.unwrap(), 1_u8);

    let x1 = registry.get_transient::<u16>();
    assert_eq!(x1.unwrap(), 2_u16);

    let x2 = registry.get_transient::<u32>();
    assert_eq!(x2.unwrap(), 4_u32);

    let s1 = registry.get_singleton::<String>().unwrap();
    assert_eq!(&*s1, &"Hello, World".to_owned());
}

#[test]
fn singletons_without_deps() {
    let registry = Registry::empty();
    registry.register_transient(|| 1_u8);
    registry.register_transient(|| 1_u16);
    registry.register_transient(|| 1_u32);
    registry.register_singleton(|| 8_i8);
    registry.register_singleton(|| 16_i16);
    registry.register_singleton(|| 32_i32);

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
    registry.register_transient(|| 1_u8);
    registry.register_singleton(|| 8_i8);

    registry
        .with_deps::<_, (Transient<u8>, Singleton<i8>)>()
        .register_singleton(|(i, j)| {
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
        .register_transient(|(i,)| {
            let i = i.get();
            u16::from(i) + 1_u16
        });

    registry
        .with_deps::<_, (Transient<u8>, Transient<u16>)>()
        .register_transient(|(i, j)| {
            let i = i.get();
            let j = j.get();
            u32::from(i) + u32::from(j) + 1_u32
        });

    assert!(
        registry.validate_all().is_err(),
        "should fail due to missing u8 dependency"
    );

    let x1 = registry.get_transient::<u16>();
    assert_eq!(x1.is_err(), true);

    let x2 = registry.get_transient::<u32>();
    assert_eq!(x2.is_err(), true);

    let s1 = registry.get_singleton::<String>();
    assert_eq!(s1.is_err(), true);
}

#[test]
fn test_fallible_transient() {
    let registry = Registry::empty();
    registry.try_register_transient(|| Ok(1_u8));
    registry
        .with_deps::<_, (Transient<u8>,)>()
        .try_register_transient(|(first,)| Ok((*first + 15) as u16));
    registry.register_transient(|| 1_u32);
    registry.validate_all_full().unwrap();

    let x = registry.get_transient::<u8>();
    assert_eq!(x.unwrap(), 1_u8);
    let x1 = registry.get_transient::<u16>();
    assert_eq!(x1.unwrap(), 16_u16);
    let x2 = registry.get_transient::<u32>();
    assert_eq!(x2.unwrap(), 1_u32);
}

#[test]
fn test_fallible_singleton() {
    let registry = Registry::empty();
    registry.try_register_singleton(|| Ok(1_u8));
    registry
        .with_deps::<_, (Singleton<u8>,)>()
        .try_register_singleton(|(first,)| Ok((**first + 15) as u16));
    registry.register_singleton(|| 1_u32);
    registry.validate_all_full().unwrap();

    let x = registry.get_singleton::<u8>();
    assert_eq!(*x.unwrap(), 1_u8);
    let x1 = registry.get_singleton::<u16>();
    assert_eq!(*x1.unwrap(), 16_u16);
    let x2 = registry.get_singleton::<u32>();
    assert_eq!(*x2.unwrap(), 1_u32);
}

#[test]
#[should_panic]
#[allow(clippy::should_panic_without_expect)]
fn panic_when_registered_twice_diff() {
    let registry = Registry::empty();
    registry.register_transient(|| 1_u8);
    registry.register_singleton(|| 1_u8);
}

#[test]
#[should_panic]
#[allow(clippy::should_panic_without_expect)]
fn panic_when_registered_twice_transient() {
    let registry = Registry::empty();
    registry.register_transient(|| 1_u8);
    registry.register_transient(|| 1_u8);
}

#[test]
#[should_panic]
#[allow(clippy::should_panic_without_expect)]
fn panic_when_registered_twice_singleton() {
    let registry = Registry::empty();
    registry.register_singleton(|| 1_u8);
    registry.register_singleton(|| 1_u8);
}

#[derive(Debug)]
struct NotClone {
    inner: String,
}

#[test]
fn register_not_clone() {
    let registry = Registry::empty();
    registry.register_transient(|| NotClone {
        inner: String::new(),
    });

    let _not_clone = registry.get_transient::<NotClone>().unwrap();
}

struct TupleWithStatic(&'static str);

#[test]
fn register_static_lifetime() {
    let registry = Registry::empty();
    registry.register_transient(|| TupleWithStatic("TEST"));
}
