# First Steps

## Installation

Install [`ferrunix`] by adding the following to your `Cargo.toml`.

```toml
[dependencies]
ferrunix = "0.3"
```

## Setup

There are two fundamental ways of using [`ferrunix`]:

1. Manually, without any proc-macros
2. Using the `#[derive(Inject)]` proc-macro

Both options work together seamlessly and can be mixed and matched, even in the
same library or binary.

The `#[derive]` macro is designed to augment the manual registration without
being too intrusive. Therefore, it's fundamental to understand how [`ferrunix`]'s
`Registry` works.

```rust
# #![allow(unused)]
# extern crate ferrunix;
use ferrunix::Registry;

fn main() {
    // Create a new empty registry.
    let registry = Registry::empty();
}
```


[`ferrunix`]: https://github.com/Leandros/ferrunix

