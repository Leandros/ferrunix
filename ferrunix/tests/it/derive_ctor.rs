use ferrunix::{Inject, Registry};

pub trait Adder {
    fn add(&self, lhs: u32, rhs: u32) -> u32;
}

#[derive(Inject)]
#[provides(transient = "dyn Adder", no_registration)]
pub struct MyAdder {}
impl Adder for MyAdder {
    fn add(&self, lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }
}

#[derive(Inject)]
#[provides(transient, no_registration, ctor = "new")]
pub struct DerivedCustomCtor {
    // These two are not injected.
    counter: u32,
    prefix: String,

    #[inject(transient)]
    adder: Box<dyn Adder>,
}

impl DerivedCustomCtor {
    pub fn new(adder: Box<dyn Adder>) -> Self {
        Self {
            counter: 1,
            prefix: "log-prefix: ".to_owned(),
            adder,
        }
    }
}

#[test]
fn custom_ctor() {
    let registry = Registry::empty();
    MyAdder::register(&registry);
    DerivedCustomCtor::register(&registry);

    let derived = registry.get_transient::<DerivedCustomCtor>().unwrap();
    assert_eq!(derived.counter, 1);
    assert_eq!(derived.prefix, "log-prefix: ");
    assert_eq!(derived.adder.add(1, 3), 4);
}
