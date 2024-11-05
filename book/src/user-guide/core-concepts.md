# Core Concepts

Fundamentally, [`ferrunix`] is a hash table, with the registered type as the
key, and the objects constructor (for transients) or value (for singletons) as
the value.

In Rust, this could be, very crudely, represented as a `HashMap`:

```rust,ignore
enum Provider {
    Transient(fn() -> Box<dyn Any>),
    Singleton(OnceCell<dyn Any>),
}

type Registry = HashMap<TypeId, Provider>;
```

Of course, reality is a bit more complicated.

The core concepts necessary to understand for efficiently using [`ferrunix`] are:

* [The Registry](./core-concepts-registry.md)
* [Lifetimes](./core-concepts-lifetimes.md)
* [Registration](./core-concepts-registration.md)
* [Retrieval](./core-concepts-retrieval.md)

All of which will be tacked in the next few pages.

[`ferrunix`]: https://github.com/Leandros/ferrunix
[`Registry`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/struct.Registry.html
