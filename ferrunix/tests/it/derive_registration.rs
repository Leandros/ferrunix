#![allow(dead_code)]
use ferrunix::{Inject, Registry};

#[derive(Inject)]
#[provides(singleton, no_registration)]
pub struct NotRegistered {}

#[test]
#[cfg(not(feature = "tokio"))]
fn no_not_registered_type() {
    let global = Registry::autoregistered();
    let not_registered = global.get_singleton::<NotRegistered>();
    assert!(not_registered.is_err());
}

#[tokio::test]
#[cfg(feature = "tokio")]
async fn no_not_registered_type() {
    let global = Registry::autoregistered().await;
    let not_registered = global.get_singleton::<NotRegistered>().await;
    assert!(not_registered.is_err());
}
