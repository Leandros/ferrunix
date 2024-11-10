#![allow(dead_code)]
use ferrunix::{Inject, Registry};

#[derive(Inject)]
#[provides(singleton, no_registration)]
pub struct NotRegistered {}

#[test]
fn no_not_registered_type() {
    let global = Registry::autoregistered();
    let not_registered = global.get_singleton::<NotRegistered>();
    assert!(not_registered.is_none());
}
