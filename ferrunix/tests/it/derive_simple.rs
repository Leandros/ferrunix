#![cfg(not(miri))]
use std::sync::RwLock;

#[allow(unused)]
use ferrunix::{Inject, Registry};

#[derive(Inject)]
#[provides(transient)]
struct Empty {}

trait Logger: Send + Sync {
    fn log(&self, msg: &'static str);
}

#[derive(Inject)]
#[provides(transient = "dyn Logger")]
struct StdoutLog {}
impl Logger for StdoutLog {
    fn log(&self, msg: &'static str) {
        println!("{msg}");
    }
}

trait ColorLogger: Send + Sync {
    fn log_colored(&self, msg: &'static str);
}

#[derive(Inject)]
#[provides(singleton = "dyn ColorLogger")]
struct ColoredStdoutLog {
    #[inject(transient)]
    log: Box<dyn Logger>,
    cache: RwLock<String>,
}

impl ColorLogger for ColoredStdoutLog {
    fn log_colored(&self, msg: &'static str) {
        self.cache.write().expect("non poisened").push_str(msg);
        self.log.log(msg);
    }
}

#[derive(Inject)]
#[provides(transient = "StringTemplate")]
struct StringTemplate {
    #[inject(ctor = r#""The Magic Number is ""#)]
    raw: &'static str,
}

#[derive(Inject)]
#[provides(transient)]
struct TemplateMaker {
    #[inject(transient)]
    template: StringTemplate,
    #[inject(ctor = "5")]
    number: u32,
}

#[test]
fn inject_stringtemplate() {
    let registry = Registry::empty();

    // We register all types manually, to avoid having types from other tests
    // registered here and failing our tests for no reason.
    Empty::register(&registry);
    StdoutLog::register(&registry);
    ColoredStdoutLog::register(&registry);
    StringTemplate::register(&registry);
    TemplateMaker::register(&registry);

    registry.validate_all_full().unwrap();

    let logger = registry.get_singleton::<Box<dyn ColorLogger>>().unwrap();
    logger.log_colored("hello");

    let stringtemplate = registry.get_transient::<StringTemplate>().unwrap();
    assert_eq!(stringtemplate.raw, "The Magic Number is ");

    let maker = registry.get_transient::<TemplateMaker>().unwrap();
    assert_eq!(maker.template.raw, "The Magic Number is ");
    assert_eq!(maker.number, 5);
}
