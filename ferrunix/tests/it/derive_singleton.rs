//! Tests for creating singletons using the derive macro syntax.

use ferrunix::{Inject, Ref, Registry};

#[derive(Inject)]
#[provides(singleton = "dyn Logger + Send + Sync")]
pub struct MyLogger {}

impl Logger for MyLogger {
    fn info(&self, message: &str) {
        println!("INFO: {message}");
    }
}

pub trait Logger {
    fn info(&self, message: &str);
}

#[derive(Inject)]
#[provides(singleton)]
pub struct MyContext {
    #[inject(singleton)]
    logger: Ref<Box<dyn Logger + Send + Sync>>,
}

#[test]
fn derive_singleton() {
    let registry = Registry::empty();

    // We register all types manually, to avoid having types from other tests
    // registered here and failing our tests for no reason.
    MyLogger::register(&registry);
    MyContext::register(&registry);

    registry.validate_all_full().unwrap();

    let logger = registry
        .get_singleton::<Box<dyn Logger + Send + Sync>>()
        .unwrap();
    logger.info("Hello!");

    let ctx = registry.get_singleton::<MyContext>().unwrap();
    ctx.logger.info("Hello!");
}
