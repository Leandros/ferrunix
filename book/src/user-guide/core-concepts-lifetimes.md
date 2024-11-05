# Lifetimes

### Transient

Transients are constructed every time when requested. For transients, the
following container types are supported:

- `T`: No container.
- `Box<T>`: A boxed type, where `T` can be a `dyn T`.

### Singleton

Singletons are lazily constructed on the first access, and only a single
instance can be created over the programs lifetime. When requested, a reference
to the object is returned. The following container types are supported:

- `Rc<T>`: When `multithread` and `tokio` feature are disabled.
- `Arc<T>`: When the `multithread` or `tokio` is enabled.

The library offers a [`Ref`] type alias, which is aliasing the correct
container, based on the enabled features.

The next step is to understand [registration].

[`Ref`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/type.Ref.html
[registration]: ./core-concepts-registration.md
