# Registry

## Creation

- [`Registry::empty()`]
- [`Registry::global()`]

## Registration

- [`Registry::transient(...)`]
- [`Registry::singleton(...)`]
- [`Registry::with_deps::<_, (Ts, ...)>().transient(...)`]
- [`Registry::with_deps::<_, (Ts, ...)>().singleton(...)`]

## Retrieval

- [`Registry::get_transient::<T>()`]
- [`Registry::get_singleton::<T>()`]

## Validation

- [`Registry::validate::<T>()`]
- [`Registry::validate_all()`]
- [`Registry::validate_all_full()`]


[`Registry::empty()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.empty
[`Registry::global()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.global
[`Registry::transient(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.transient
[`Registry::singleton(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.singleton
[`Registry::with_deps::<_, (Ts, ...)>().transient(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Builder.html#method.transient
[`Registry::with_deps::<_, (Ts, ...)>().singleton(...)`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Builder.html#method.singleton
[`Registry::get_transient::<T>()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.get_transient
[`Registry::get_singleton::<T>()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.get_singleton
[`Registry::validate::<T>()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.validate
[`Registry::validate_all()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.validate_all
[`Registry::validate_all_full()`]: https://leandros.github.io/ferrunix/docs-multithread/ferrunix/registry/struct.Registry.html#method.validate_all_full
