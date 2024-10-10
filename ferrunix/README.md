# Ferrunix

[![Build Status](https://github.com/leandros/ferrunix/actions/workflows/ci.yml/badge.svg)](https://github.com/leandros/ferrunix/actions)
[![Crates.io](https://img.shields.io/crates/v/ferrunix.svg)](https://crates.io/crates/ferrunix)
[![API reference](https://docs.rs/ferrunix/badge.svg)](https://docs.rs/ferrunix/)

A simple, idiomatic, and lightweight dependency injection framework for Rust.

```toml
[dependencies]
ferrunix = "0"
```

*Compiler support: requires rustc 1.64+*

## Example

```rust
use ferrunix::{Ref, Registry, Transient};
use example::{Logger, BillingService, SysLog}

#[derive(Debug)]
pub struct ExampleService {
    logger: Ref<dyn Logger>, // Logger is a singleton, and only instantiated once.
    billing: Box<dyn BillingService>, // The BillingService is constructed each time it's requested.
}

impl ExampleService {
    pub fn new(logger: Ref<dyn Logger>, billing: Box<dyn BillingService>) -> Self {
        Self { logger, billing }
    }

    pub fn do_work(&self) {
        // Omitted for brevity...
    }
}

fn main() {
    let registry = Registry::global();
    registry.singleton(|| SysLog::default()); // `SysLog` is a concrete type implementing `Logger`.
    registry
        .with_deps::<_, (Singleton<Ref<dyn Logger>>, Transient<Box<dyn BillingService>>,)>()
        .transient(|(logger, billing)| {
            ExampleService::new(*logger, *billing)
        });

    debug_assert!(registry.validate_all());

    let service = registry.get_transient::<ExampleService>().unwrap();
    service.do_work();
}
```

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
