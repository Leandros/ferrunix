# Registration


To register a new object, we need to tell the [`Registry`] which type we want
to use as a key, what dependencies it has, and how to construct it.

For the registration, we have four functions on the [`Registry`] that are of
interest:

- [`Registry::register_transient(...)`]
- [`Registry::register_singleton(...)`]
- [`Registry::with_deps::<_, (Ts, ...)>().register_transient(...)`]
- [`Registry::with_deps::<_, (Ts, ...)>().register_singleton(...)`]


## Without dependencies

The most straightforward object to register is a type without dependencies:

```rust
# #![allow(unused)]
# extern crate ferrunix;
use ferrunix::Registry;

pub struct CurrentCurrency(&'static str);

fn main() {
    // Create a new empty registry.
    let registry = Registry::empty();

    // Register `CurrentCurrency` as a singleton.
    registry.register_singleton(|| CurrentCurrency("USD"));

    // Retrieve `CurrentCurrency` from the registry.
    let currency = registry.singleton::<CurrentCurrency>().unwrap();

    // Assert that our retrieved object is actually what we expect.
    assert_eq!(currency.0, "USD");
}
```

Of course, this example is rather simple, but should highlight the pattern of
registering a new object:

1. The constructor is registered with `registry.register_singleton` or
   `registry.register_transient`.
2. A new object is retrieved using either `registry.singleton` or
   `registry.transient`.
3. The retrieved object is used.

The `CurrentCurrency` object in the example above is registered with a
`Singleton` lifetime. As a result, it needs to be retrieved with `singleton`.
Trying to retrieve it with `transient` will return `None`.

<div class="warning">
<b>Remember!</b>
<br />
<i>Retrieval lifetime must match registration lifetime!</i>
</div>


## With dependencies

Types that have dependencies need to state the dependencies they have, so that
the [`Registry`] can fulfill them before passing them to the constructor of the
type to be registered.

```rust
# #![allow(unused)]
# extern crate ferrunix;
use ferrunix::{Transient, Registry};

pub struct CurrentCurrency(&'static str);

pub struct Config {
    currency: CurrentCurrency,
}

fn main() {
    let registry = Registry::empty();
    // Register a type without dependencies.
    registry.register_transient(|| CurrentCurrency("USD"));

    // Register our `Config` types with a dependency.
    registry
        .with_deps::<_, (Transient<CurrentCurrency>,)>() // Trailing comma required!
        .register_transient(|(currency,)| {
            Config {
                currency: currency.get(),
            }
        });

    // Construct a new config.
    let config = registry.transient::<Config>().unwrap();

    // Assert that our retrieved object is actually what we expect.
    assert_eq!(config.currency.0, "USD");
}
```

This follows a very similar pattern as our previous example; however, the
registration of the type with dependencies is slightly different.

Let's examine this in a bit more detail:

```rust
# #![allow(unused)]
# extern crate ferrunix;
# use ferrunix::{Transient, Registry};
# pub struct CurrentCurrency(&'static str);
# pub struct Config {
#     currency: CurrentCurrency,
# }
# fn main() {
# let registry = Registry::empty();
registry
  .with_deps::<_, (Transient<CurrentCurrency>,)>() // <-- (1)
  .register_transient(|(currency,)| { // <-- (2)
      Config {
          currency: currency.get(), // <-- (3)
      }
  });
# }
```

At `(1)`, the `.with_deps` function has the following (simplified) call
signature: `fn with_deps<Ret, Deps>` with `Ret` being the type that's to be
registered, in our case `Config` and `Deps` a tuple type of our dependencies,
in our case `(CurrentCurrency,)`.

To indicate, that this is a transient dependency, the [`Transient<T>`] marker type
needs to be used. A similar marker also exists for singletons, it's called
[`Singleton<T>`].

At `(2)`, the constructor for the transient object `Config` is registered. It
takes a single argument, a tuple of dependencies, which we immediately
destructure into it's parts.

<div class="warning">
<b>Careful!</b>
<br />
<i>The trailing comma for the tuple at <code>(1)</code> and <code>(2)</code> is
necessary to indicate a single element tuple (and not a literal that's in
parentheses). This is only necessary for types with one dependency.</i>
</div>

At `(3)`, we'll use the [`Transient<T>`]'s `get()` function to consume it and
get the inner `CurrentCurrency` to construct the `Config`, which is returned
from the constructor function. The inner `CurrentCurrency` is constructed with
the previously registered constructor.

With the registration done, the last part to is [retrieval of constructed objects].

[`Registry`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/struct.Registry.html
[`Registry::register_transient(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.register_transient
[`Registry::register_singleton(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.register_singleton
[`Registry::with_deps::<_, (Ts, ...)>().register_transient(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Builder.html#method.register_transient
[`Registry::with_deps::<_, (Ts, ...)>().register_singleton(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Builder.html#method.register_singleton

[`Transient<T>`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/struct.Transient.html
[`Singleton<T>`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/struct.Singleton.html
[retrieval of constructed objects]: ./core-concepts-retrieval.md
