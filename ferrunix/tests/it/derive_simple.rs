#[allow(unused)]
use ferrunix::{Inject, Registry};

#[derive(Inject)]
#[provides(transient)]
struct Empty {}

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

// #[automatically_derived]
// impl StringTemplate {
//     #[allow(clippy::use_self)]
//     pub(crate) fn register(registry: &::ferrunix::Registry) {
//         registry.transient::<StringTemplate>(|| StringTemplate {
//             template: "The Magic Number is ",
//         });
//     }
// }

// ::ferrunix::autoregister!(::ferrunix::RegistrationFunc::new(
//     StringTemplate::register
// ));

#[test]
fn inject_stringtemplate() {
    let registry = Registry::autoregistered();
    assert!(registry.validate_all());

    let stringtemplate = registry.get_transient::<StringTemplate>().unwrap();
    assert_eq!(stringtemplate.raw, "The Magic Number is ");

    let maker = registry.get_transient::<TemplateMaker>().unwrap();
    assert_eq!(maker.template.raw, "The Magic Number is ");
    assert_eq!(maker.number, 5);
}
