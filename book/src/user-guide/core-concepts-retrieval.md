# Retrieval

Each registered object needs to be retrieved with the same lifetime that it was
registered with.

For retrieval, the following functions are available on the [`Registry`]:

- [`Registry::get_transient::<T>()`]
- [`Registry::get_singleton::<T>()`]

The generic type parameter `T` is necessary to be specified, either on the
function call or the assignee. It's necessary to find which object to
construct.

This is the example from the previous section:

```rust
# #![allow(unused)]
# extern crate ferrunix;
use ferrunix::{Transient, Registry};

pub struct CurrentCurrency(&'static str);
pub struct Config { currency: CurrentCurrency }

fn main() {
    let registry = Registry::empty();
    registry.transient(|| CurrentCurrency("USD"));
    registry
        .with_deps::<_, (Transient<CurrentCurrency>,)>() // Trailing comma required!
        .transient(|(currency,)| {
            Config {
                currency: currency.get(),
            }
        });

    // Construct a new config and currency.
    let config = registry.get_transient::<Config>().unwrap(); // <-- (1)
    let currency: CurrentCurrency = registry.get_transient().unwrap(); // <-- (2)

    // Assert that our retrieved object is actually what we expect.
    assert_eq!(config.currency.0, "USD");
    assert_eq!(currency.0, "USD");
}
```

At `(1)` the [`Registry::get_transient::<T>()`] function is used to construct a
new `Config`. The generic type parameter `T` is directly specified at the
function call.

However, at `(2)`, the Rust compiler can infer the type `T` from the annotation
on the left, and therefore, it's not necessary to explicitly specify it.

[`Registry::get_transient::<T>()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.get_transient
[`Registry::get_singleton::<T>()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.get_singleton