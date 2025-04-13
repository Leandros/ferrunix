use ferrunix::Registry;

#[derive(Debug, Default, PartialEq, PartialOrd, Hash)]
struct ExampleStruct {
    num: u32,
    some_string: String,
}

trait NotObjectSafe {
    fn returns(&self) -> Self; // ERROR: Self in return type
}

impl NotObjectSafe for ExampleStruct {
    fn returns(&self) -> Self {
        Self {
            num: 1,
            some_string: "2".to_owned(),
        }
    }
}

#[test]
fn can_store_objecs_impl_non_object_safe_traits() {
    let registry = Registry::empty();
    registry.register_transient(|| ExampleStruct {
        num: 0,
        some_string: "1".to_owned(),
    });

    let example = registry.get_transient::<ExampleStruct>().unwrap();
    let ret = example.returns();
    assert_eq!(ret.num, 1);
    assert_eq!(ret.some_string, "2");
}
