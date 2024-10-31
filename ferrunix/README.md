<div align="center">
  <h1>Ferrunix</h1>
  <p>
    <strong>A simple, idiomatic, and lightweight <a href="https://en.wikipedia.org/wiki/Dependency_injection">dependency injection</a> framework for Rust.</strong>
  </p>
  <p>

[![Build Status](https://github.com/leandros/ferrunix/actions/workflows/ci.yml/badge.svg)](https://github.com/leandros/ferrunix/actions)
[![Crates.io](https://img.shields.io/crates/v/ferrunix.svg)](https://crates.io/crates/ferrunix)
[![API reference](https://docs.rs/ferrunix/badge.svg)](https://docs.rs/ferrunix/)
![MSRV](https://img.shields.io/crates/msrv/ferrunix)
![License](https://img.shields.io/crates/l/ferrunix.svg)

  </p>
</div>

```toml
[dependencies]
ferrunix = "0.3"
```

*Compiler support: requires rustc 1.67.1+*

**Check out the [User Guide](https://leandros.github.io/ferrunix/book).**


## Documentation

Due to how the various features affect the public API of the library, the
documentation is provided for each major feature separately.

|    Feature Flags    | Link to Documentation |
| ------------------- | --------------------- |
| `none`              | [link to docs](https://leandros.github.io/ferrunix/docs-default/ferrunix/)     |
| `multithread`       | [link to docs](https://leandros.github.io/ferrunix/docs-multithread/ferrunix/) |
| `tokio`             | [link to docs](https://leandros.github.io/ferrunix/docs-multithread/ferrunix/) |


## Features

- Can register and inject any type (incl. generics, types must be `Send` +
    `Sync` if the `multithread` feature is enabled).
- Simple and elegant Rust API, making the derive macro purely optional.
- Different dependency lifetimes:
    - Singleton: Only a single instance of the object is created.
    - Transient: A new instance is created for every request.
- Derive macro (`#[derive(Inject)]`) to simplify registration.
- Automatic registration of types.
- One global registry; with support for multipiple sub-registries.

## Cargo Feature Flags

Ferrunix has the following features to enable further functionality.
Features enabled by default are marked with `*`.

- `multithread`: Enables support for accessing the registry from multiple
    threads. This adds a bound that all registered types must be `Send`.
- `derive` (`*`): Enables support for the `#[derive(Inject)]` macro.
- `tokio`: Enables support for `async` constructors. Bumps the MSRV up to
    `1.75.0` because some of the internal traits require
    [RPITIT](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html#whats-stabilizing).
- `tracing`: Enables support for [tracing](https://docs.rs/tracing/latest/tracing/index.html) and annotates all public functions with
    [`tracing::instrument`](https://docs.rs/tracing/latest/tracing/attr.instrument.html).

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
By contributing to this project (for example, through submitting a pull
request) you agree with the <a
href="https://github.com/Leandros/ferrunix/blob/master/Contributors-License-Agreement.md">individual
contributor license agreement</a>. Make sure to read and understand it.
</sub>
