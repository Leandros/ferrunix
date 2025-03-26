//! Tests for creating singletons using the derive macro syntax.

use ferrunix::{Inject, Ref, Registry};

#[derive(Inject)]
#[provides(singleton = "dyn Logger")]
pub struct MyLogger {}

// impl MyLogger {
//     fn register(registry: &Registry) {
//         registry.singleton::<Box<dyn Logger>, _>(|| Box::new(MyLogger {}));
//     }
// }

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
    logger: Ref<Box<dyn Logger>>,
}

// impl MyContext {
//     fn register(registry: &Registry) {
//         registry
//             .with_deps::<Self, (Singleton<Box<dyn Logger>>,)>()
//             .singleton(|(logger,)| MyContext {
//                 logger: logger.get(),
//             });
//     }
// }

#[test]
fn derive_singleton() {
    let registry = Registry::empty();

    MyLogger::register(&registry);
    MyContext::register(&registry);

    registry.validate_all_full().unwrap();

    let logger = registry.get_singleton::<Box<dyn Logger>>().unwrap();
    logger.info("Hello!");

    let ctx = registry.get_singleton::<MyContext>().unwrap();
    ctx.logger.info("Hello!");
}
