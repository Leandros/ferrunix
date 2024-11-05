# Registry

The [`Registry`] is used for registering new objects, and retrieving previously
registered objects.

Each object has a specific lifetime set during registration, currently, the following two
lifetimes are supported:

- `transient`: when retrieved, a new object is created.
- `singleton`: when retrieved, a single instance is lazily created, and a
  reference is returned.

All functions of the registry are outlined in the [`Registry` reference guide]
or described in detail in [the documentation].

The next step is to more thoroughly understand [lifetimes].

[`Registry`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/struct.Registry.html
[`Registry` reference guide]: ../reference-guide/registry.md
[the documentation]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/struct.Registry.html
[lifetimes]: ./core-concepts-lifetimes.md
