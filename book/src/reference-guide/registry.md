# Registry

## Creation

- [`Registry::empty()`]
- [`Registry::global()`]

## Registration

- [`Registry::register_transient(...)`]
- [`Registry::register_singleton(...)`]
- [`Registry::with_deps::<_, (Ts, ...)>().register_transient(...)`]
- [`Registry::with_deps::<_, (Ts, ...)>().register_singleton(...)`]

## Retrieval

- [`Registry::transient::<T>()`]
- [`Registry::singleton::<T>()`]

## Validation

- [`Registry::validate::<T>()`]
- [`Registry::validate_all()`]
- [`Registry::validate_all_full()`]


[`Registry::empty()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.empty
[`Registry::global()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.global
[`Registry::register_transient(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.register_transient
[`Registry::register_singleton(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.register_singleton
[`Registry::with_deps::<_, (Ts, ...)>().register_transient(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Builder.html#method.register_transient
[`Registry::with_deps::<_, (Ts, ...)>().register_singleton(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Builder.html#method.register_singleton
[`Registry::transient::<T>()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.transient
[`Registry::singleton::<T>()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.singleton
[`Registry::validate::<T>()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.validate
[`Registry::validate_all()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.validate_all
[`Registry::validate_all_full()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.validate_all_full
