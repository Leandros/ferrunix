# Feature Flags

Features enabled by default are marked with `*`.

- `multithread`: Enables support for accessing the registry from multiple
    threads. This adds a bound that all registered types must be `Send`.
- `derive` (`*`): Enables support for the `#[derive(Inject)]` macro.
- `tokio`: Enables support for `async` constructors. Bumps the MSRV up to
    `1.75.0` because some of the internal traits require
    [RPITIT](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html#whats-stabilizing).
- `tracing`: Enables support for [tracing](https://docs.rs/tracing/latest/tracing/index.html) and annotates all public functions with
    [`tracing::instrument`](https://docs.rs/tracing/latest/tracing/attr.instrument.html).
