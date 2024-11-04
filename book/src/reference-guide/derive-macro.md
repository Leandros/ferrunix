# Derive Macro

The `Inject` derive macro supports the two following attributes:

- `#[provides]`: Customizing the object registration.
- `#[inject]`: Customizing how an injected member is created.

```rust,ignore
#[derive(Inject)]
#[provides(PROPERTY...)]
struct Transient {
    #[inject(PROPERTY...)]
    field: UserType,
}
```

## `provides` Properties

- `transient [= "<TYPE-SIGNATURE>"]`
    - The object is provided as a transient registered with `<TYPE-SIGNATURE>` as key.
      If the signature is omitted, the concrete type is used as a key.
- `singleton [= "<TYPE-SIGNATURE>"]`
    - The object is provided as a singleton registered with `<TYPE-SIGNATURE>` as key.
      If the signature is omitted, the concrete type is used as a key.

## `inject` Properties

- `default`
    - Construct the field using the `Default` implementation.
- `ctor = "<RUST-CODE>"`
    - Construct the field using the provided Rust code.
- `transient [= true]`
    - Construct the field as a transient by retrieving it from the `Registry`.
- `singleton [= true]`
    - Construct the field as a singleton by retrieving it from the `Registry`.

## Full Example

```rust
# #![allow(unused)]
# extern crate ferrunix;
use ferrunix::Inject;

pub trait Logger {}

#[derive(Inject)]
#[provides(transient = "dyn Logger")]
//                   ^^^^^^^^^^^^^^
// The explicit type can be omitted, if it matches the concrete type.
pub struct MyLogger {}

impl Logger for MyLogger {}

#[derive(Inject)]
#[provides(singleton)]
pub struct MyConfig {
    #[inject(default)]
    // Use the `Default::default` impl for construction.
    counter: u32,

    #[inject(ctor = r#""log-prefix: ".to_owned()"#)]
    //              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    // The constructor must be valid Rust code. For strings, two sets of quotes
    // are required.
    prefix: String,

    #[inject(transient /* = true */)]
    //                 ^^^^^^^^^^^^
    // The long form with `= true` is optional.
    logger: Box<dyn Logger>,
}

fn main() {
    let registry = ferrunix::Registry::empty();
    MyLogger::register(&registry);
    MyConfig::register(&registry);
}
```
