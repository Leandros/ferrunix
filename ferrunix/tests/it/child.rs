#![allow(clippy::unwrap_used, clippy::panic, dead_code)]

use ferrunix::{Ref, Registry};

#[test]
#[cfg(not(feature = "tokio"))]
fn simple_child() {
    let registry = Ref::new(Registry::empty());
    registry.register_transient(|| 5_u8);
    registry.register_singleton(|| 5_u16);
    registry.validate_all_full().unwrap();

    let child1 = registry.child();
    child1.register_transient(|| 0_u8);
    child1.validate_all_full().unwrap();

    let child2 = registry.child();
    child2.register_singleton(|| 0_u32);
    child2.validate_all_full().unwrap();

    assert_eq!(registry.transient::<u8>().unwrap(), 5_u8);
    assert_eq!(*registry.singleton::<u16>().unwrap(), 5_u16);

    assert_eq!(child1.transient::<u8>().unwrap(), 0_u8);
    assert_eq!(*child1.singleton::<u16>().unwrap(), 5_u16);

    assert_eq!(child2.transient::<u8>().unwrap(), 5_u8);
    assert_eq!(*child2.singleton::<u16>().unwrap(), 5_u16);
    assert_eq!(*child2.singleton::<u32>().unwrap(), 0_u32);
}

#[tokio::test]
#[cfg(feature = "tokio")]
async fn simple_child_tokio() {
    let registry = Ref::new(Registry::empty());
    registry
        .register_transient(|| Box::pin(async move { 5_u8 }))
        .await;
    registry
        .register_singleton(|| Box::pin(async move { 5_u16 }))
        .await;
    registry.validate_all_full().unwrap();

    let child1 = registry.child();
    child1
        .register_transient(|| Box::pin(async move { 0_u8 }))
        .await;
    child1.validate_all_full().unwrap();

    let child2 = registry.child();
    child2
        .register_singleton(|| Box::pin(async move { 0_u32 }))
        .await;
    child2.validate_all_full().unwrap();

    assert_eq!(registry.transient::<u8>().await.unwrap(), 5_u8);
    assert_eq!(*registry.singleton::<u16>().await.unwrap(), 5_u16);

    assert_eq!(child1.transient::<u8>().await.unwrap(), 0_u8);
    assert_eq!(*child1.singleton::<u16>().await.unwrap(), 5_u16);

    assert_eq!(child2.transient::<u8>().await.unwrap(), 5_u8);
    assert_eq!(*child2.singleton::<u16>().await.unwrap(), 5_u16);
    assert_eq!(*child2.singleton::<u32>().await.unwrap(), 0_u32);
}
