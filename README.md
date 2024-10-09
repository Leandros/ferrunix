# Ferrunix

[![Build Status](https://github.com/leandros/ferrunix/actions/workflows/ci.yml/badge.svg)](https://github.com/leandros/ferrunix/actions)
[![Crates.io](https://img.shields.io/crates/v/ferrunix.svg)](https://crates.io/crates/ferrunix)
[![API reference](https://docs.rs/ferrunix/badge.svg)](https://docs.rs/ferrunix/)

A simple, idiomatic, and lightweight dependency injection framework for Rust.

```toml
[dependencies]
ferrunix = "0"
```

*Compiler support: requires rustc 1.62+*

## Example

```rust
use ferrunix::{Registry, Transient};

#[derive(Default, Debug)]
pub struct Logger {}

impl Logger {
    pub fn info(&self, message: &str) {
        println!("INFO: {message}");
    }
}

#[derive(Debug)]
pub struct Worker {
    logger: Box<Logger>,
}

impl Worker {
    pub fn new(logger: Box<Logger>) -> Self {
        Self { logger }
    }

    pub fn do_work(&self) {
        self.logger.info("doing something ...");
    }
}

fn main() {
    let mut registry = Registry::empty();
    registry.transient(|| Logger::default());
    registry
        .with_deps::<_, (Transient<Logger>,)>()
        .transient(|(logger,)| Worker::new(logger));

    let worker = registry.get_transient::<Worker>().unwrap();
    worker.do_work();
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
request) you agree with the [individual contributor license
agreement](Contributors-License-Agreement.md). Make sure to read and understand
it.
</sub>
