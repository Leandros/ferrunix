<div align="center">
  <h1>Ferrunix</h1>
  <p>
    <strong>A simple, idiomatic, and lightweight <a href="https://en.wikipedia.org/wiki/Dependency_injection">dependency injection</a> framework for Rust.</strong>
  </p>
  <p>

[![Build Status](https://github.com/leandros/ferrunix/actions/workflows/ci.yml/badge.svg)](https://github.com/leandros/ferrunix/actions)
[![Crates.io](https://img.shields.io/crates/v/ferrunix.svg)](https://crates.io/crates/ferrunix)
[![API reference](https://docs.rs/ferrunix/badge.svg)](https://docs.rs/ferrunix/)
![License](https://img.shields.io/crates/l/ferrunix.svg)

  </p>
</div>

```toml
[dependencies]
ferrunix = "0"
```

*Compiler support: requires rustc 1.64+*

## [Changelog](https://github.com/Leandros/ferrunix/releases)

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

## Usage

Add the dependency to your `Cargo.toml`:

```bash
cargo add ferrunix
```

Register your types with the [`Registry`](https://docs.rs/ferrunix/latest/ferrunix/):

```rust
use ferrunix::{Ref, Registry, Transient};
use example::{Logger, BillingService, SysLog}

#[derive(Debug, Default)]
pub struct ExampleService {}

impl ExampleService {
    pub fn do_work(&self) {
        // Omitted for brevity...
    }
}

fn main() {
    let registry = Registry::global();
    registry.transient(|| ExampleService::default());
    // Register more types here ...

    debug_assert!(registry.validate_all());

    let service = registry.get_transient::<ExampleService>().unwrap();
    service.do_work();
}
```

## Cargo Feature Flags

Ferrunix has the following features to enable further functionality.
Features enabled by default are marked with `*`.

- `multithread`: Enables support for accessing the registry from multiple
    threads. This adds a bound that all registered types must be `Send` and
    `Sync`.
- `derive` (`*`): Enables support for the `#[derive(Inject)]` macro.
- `tokio`: Enables support for `async` constructors. Bumps the MSRV up to
    `1.75.0` because some of the internal traits require
    [RPITIT](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html#whats-stabilizing).

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
