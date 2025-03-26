use crate::attr::DeriveAttrInput;
use darling::FromDeriveInput;
use syn::{parse2, parse_quote, parse_str};

use super::*;

#[test]
fn test_get_ctor_for() {
    let run_test = |ty: &str, result: &str| {
        let inner = quote! { Self {} };
        let parsed: syn::Type = syn::parse_str(ty).unwrap();
        let ctor = get_ctor_for(&parsed, inner).unwrap();
        let result_from_test: syn::Expr = parse2(ctor).unwrap();
        let result_required: syn::Expr = parse_str(result).unwrap();
        assert_eq!(result_from_test, result_required);
    };

    run_test(
        "::std::boxed::Box<FooBar>",
        "::std::boxed::Box::new(Self {})",
    );
    run_test("std::boxed::Box<FooBar>", "::std::boxed::Box::new(Self {})");
    run_test("::ferrunix::Ref<FooBar>", "::ferrunix::Ref::new(Self {})");
    run_test("Ref<FooBar>", "::ferrunix::Ref::new(Self {})");
    run_test("Box<FooBar>", "::std::boxed::Box::new(Self {})");
}

#[test]
fn test_transform_type() {
    let run_test = |ty: &str, what: TransformType, result: &str| {
        let parsed: syn::Type = syn::parse_str(ty).unwrap();
        let result_from_test = transform_type(&parsed, what).unwrap();
        let result_required: syn::Type = parse_str(result).unwrap();
        assert_eq!(*result_from_test, result_required, "test failed");
    };

    run_test(
        "dyn Foo",
        TransformType::Transient,
        "::std::boxed::Box<dyn Foo>",
    );
    run_test(
        "dyn Foo + Send + Sync",
        TransformType::Transient,
        "::std::boxed::Box<dyn Foo + Send + Sync>",
    );
    run_test(
        "Box<dyn Foo + Send + Sync>",
        TransformType::Transient,
        "Box<dyn Foo + Send + Sync>",
    );
    run_test("Foo", TransformType::Transient, "Foo");

    run_test(
        "dyn Foo + Send + Sync",
        TransformType::Singleton,
        "::std::boxed::Box<dyn Foo + Send + Sync>",
    );
    run_test("Foo", TransformType::Singleton, "Foo");
    run_test(
        "::ferrunix::Ref<Foo>",
        TransformType::Singleton,
        "::ferrunix::Ref<Foo>",
    );
}

#[test]
fn test_strip_arc_rc_ref() {
    let run_test = |ty: &str, result: &str| {
        let parsed: syn::Type = syn::parse_str(ty).unwrap();
        let result_from_test = strip_arc_rc_ref(&parsed).unwrap();
        let result_required: syn::Type = parse_str(result).unwrap();
        assert_eq!(*result_from_test, result_required, "test failed");
    };

    run_test("::ferrunix::Ref<MySingleton>", "MySingleton");
    run_test("ferrunix::Ref<MySingleton>", "MySingleton");
    run_test("Ref<MySingleton>", "MySingleton");

    run_test("Arc<MySingleton>", "MySingleton");
    run_test("::std::sync::Arc<MySingleton>", "MySingleton");
    run_test("std::sync::Arc<MySingleton>", "MySingleton");

    run_test("Rc<MySingleton>", "MySingleton");
    run_test("::std::rc::Rc<MySingleton>", "MySingleton");
    run_test("std::rc::Rc<MySingleton>", "MySingleton");
}
