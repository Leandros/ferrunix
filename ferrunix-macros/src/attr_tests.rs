#![allow(
    clippy::similar_names,
    clippy::assertions_on_result_states,
    clippy::needless_raw_strings,
    clippy::needless_raw_string_hashes
)]
use quote::format_ident;

use super::*;

#[test]
fn attrs_fields_success() {
    let input = r#"
#[derive(Inject)]
#[provides(transient = "dyn FooTrait")]
pub struct Foo {
    #[inject(default)]
    bar: u8,
    #[inject(ctor = "-1")]
    baz: i64,
    #[inject(transient)]
    my_transient: Box<dyn BarTrait>,
    #[inject(singleton)]
    my_singleton: Arc<dyn BazTrait>,
    #[inject(transient = true)]
    my_transient_long: Box<dyn BarTrait>,
    #[inject(singleton = true)]
    my_singleton_long: Arc<dyn BazTrait>,
}"#;
    let parsed = syn::parse_str(input).unwrap();
    let receiver = DeriveAttrInput::from_derive_input(&parsed).unwrap();

    let fields = receiver.data.take_struct().unwrap();
    let get_field = |name: &str| -> &DeriveField {
        fields
            .fields
            .iter()
            .find(|el| el.ident().unwrap() == &format_ident!("{name}"))
            .unwrap()
    };
    let bar = get_field("bar");
    let baz = get_field("baz");
    let my_transient = get_field("my_transient");
    let my_singleton = get_field("my_singleton");
    let my_transient_long = get_field("my_transient_long");
    let my_singleton_long = get_field("my_singleton_long");

    assert!(bar.default);
    assert!(bar.ctor.is_none());

    assert!(!baz.default);
    assert_eq!(&*baz.ctor.clone().unwrap(), &"-1".to_owned());

    assert!(!my_transient.default);
    assert!(my_transient.ctor().is_none());
    assert!(my_transient.transient);
    assert!(!my_transient.singleton);

    assert!(!my_transient_long.default);
    assert!(my_transient_long.ctor().is_none());
    assert!(my_transient_long.transient);
    assert!(!my_transient_long.singleton);

    assert!(!my_singleton.default);
    assert!(my_singleton.ctor().is_none());
    assert!(!my_singleton.transient);
    assert!(my_singleton.singleton);

    assert!(!my_singleton_long.default);
    assert!(my_singleton_long.ctor().is_none());
    assert!(!my_singleton_long.transient);
    assert!(my_singleton_long.singleton);
}

#[test]
fn attr_transient_explicit() {
    let input = r#"
#[derive(Inject)]
#[provides(transient = "Box<Foo>")]
pub struct Foo {
    counter: u8,
}"#;
    let parsed = syn::parse_str(input).unwrap();
    let receiver = DeriveAttrInput::from_derive_input(&parsed);
    let receiver = receiver.unwrap();
    let transient = receiver.transient().unwrap();
    let ty: syn::Type = syn::parse2(quote!(Box<Foo>)).unwrap();
    assert_eq!(transient.as_ref(), &ty);
    assert_eq!(receiver.singleton(), None);
}

#[test]
fn attr_transient_default() {
    let input = r#"
#[derive(Inject)]
#[provides(transient)]
pub struct Foo {
    counter: u8,
}"#;
    let parsed = syn::parse_str(input).unwrap();
    let receiver = DeriveAttrInput::from_derive_input(&parsed);
    let receiver = receiver.unwrap();
    let transient = receiver.transient().unwrap();
    let ty: syn::Type = syn::parse2(quote!(Self)).unwrap();
    assert_eq!(transient.as_ref(), &ty);
    assert_eq!(receiver.singleton(), None);
}

#[test]
fn attr_singleton_explicit() {
    let input = r#"
#[derive(Inject)]
#[provides(singleton = "Foo")]
pub struct Foo {
    counter: u8,
}"#;
    let parsed = syn::parse_str(input).unwrap();
    let receiver = DeriveAttrInput::from_derive_input(&parsed);
    let receiver = receiver.unwrap();
    let singleton = receiver.singleton().unwrap();
    let ty: syn::Type = syn::parse2(quote!(Foo)).unwrap();
    assert_eq!(singleton.as_ref(), &ty);
    assert_eq!(receiver.transient(), None);
}

#[test]
fn attr_singleton_default() {
    let input = r#"
#[derive(Inject)]
#[provides(singleton)]
pub struct Foo {
    counter: u8,
}"#;
    let parsed = syn::parse_str(input).unwrap();
    let receiver = DeriveAttrInput::from_derive_input(&parsed);
    let receiver = receiver.unwrap();
    let singleton = receiver.singleton().unwrap();
    let ty: syn::Type = syn::parse2(quote!(Self)).unwrap();
    assert_eq!(singleton.as_ref(), &ty);
    assert_eq!(receiver.transient(), None);
}

#[test]
fn attr_singleton_no_autoregistry() {
    let input = r#"
#[derive(Inject)]
#[provides(singleton, no_registration)]
pub struct Foo {
}"#;
    let parsed = syn::parse_str(input).unwrap();
    let receiver = DeriveAttrInput::from_derive_input(&parsed);
    let receiver = receiver.unwrap();
    assert!(receiver.no_registration());
}

#[test]
fn attr_singleton_custom_ctor() {
    let input = r#"
#[derive(Inject)]
#[provides(singleton, ctor = "new")]
pub struct Foo {
}"#;
    let parsed = syn::parse_str(input).unwrap();
    let receiver = DeriveAttrInput::from_derive_input(&parsed);
    let receiver = receiver.unwrap();
    let ctor = receiver.custom_ctor().unwrap();
    assert_eq!(*ctor.as_ident(), format_ident!("new"));
}
