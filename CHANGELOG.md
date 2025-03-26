# Changelog

All notable changes of this project will be documented in this file.
This project is following [semantic versioning](http://semver.org), and the format
of the changelog is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Deprecated features will be kept for any following maintenance release, and
will be removed after two major releases.

## [unreleased]

### <!-- 0 -->🚀 Features

### <!-- 1 -->🐛 Bug Fixes

## [0.4.0] - 2025-03-26

### <!-- 0 -->🚀 Features
- Singleton created via `#[derive]` now support `dyn` types
  - **BREAKING:** `dyn` types are now automatically wrapped in a `Box`: `Box<dyn T>`,
    which requires all users to now retrieve singletons using the new type: `registry.get_singleton::<Box<dyn T>>`

### <!-- 1 -->🐛 Bug Fixes
- Remove unused tests
- Fix singleton behavior with derive macro

## [0.3.4] - 2025-03-19

### <!-- 0 -->🚀 Features

### <!-- 1 -->🐛 Bug Fixes
- Bump MSRV to `1.70.0` to reduce stringent tokio version requirement

## [0.3.3] - 2025-03-12

### <!-- 0 -->🚀 Features

### <!-- 1 -->🐛 Bug Fixes
- Fix building on MSRV (`1.67.1`)

## [0.3.2] - 2024-11-21

### <!-- 0 -->🚀 Features
- Add `no_registration` attribute to `provides` macro
- Add `ctor` attribute to `provides` macro

### <!-- 1 -->🐛 Bug Fixes
- Remove unecessary braces around constructor
- Fix singleton derive macro registration regression
- Fix incorrectly requiring singletons to be `Clone`
- Fix `#[derive(Inject)]` macro with `tokio` enabled

## [0.3.1] - 2024-11-05

### <!-- 1 -->🐛 Bug Fixes
- Publish `ferrunix` v0.3.1 which depends on an updated `ferrunix-macros`

## [0.3.0] - 2024-11-05

MSRV is bumped from `1.64.0` to `1.67.1`. MSRV with `tokio` enabled is `1.75.0`.

### <!-- 0 -->🚀 Features
- Add support for `async` constructors. Enabled with the `tokio` feature.
- Detect loops and missing dependency in the dependency graph, detected by
  `Registry::validate_all`/`Registry::validate_all_full`.
- Constructor of singletons is now `FnOnce`.
- Much less restrictive type bounds. Bounds for single-threaded registry is
  only `'static`, bounds for multi-threaded registry is `Send + 'static`,
  bounds for async are `Send + Sync + 'static`.
- Test all documentation code using newly introduced `doc-tests` sub-crate.
- Working `#[derive(Inject)]` macro.

### <!-- 1 -->🐛 Bug Fixes
- Results of validations are cached correctly.
- All features are now additive. Priority of features is as follows: `tokio` >
  `multithread` > `default`.
- Updated examples.

## [0.2.0] - 2024-10-16

### <!-- 0 -->🚀 Features
- Mostly working proc-macro
- Allow simplified trait object syntax
- Add `multithread` feature

### <!-- 1 -->🐛 Bug Fixes
- CI badge
- Specify version of dependency for publishing
- Specify ferrunix-macros version
- Required minimal versions
- Disable miri strict provenance
- Failing test
- Book upload
- Remaining lints
- Breaking minversion
- Disable `unsync` if not required
- Build on rust 1.64
- Ci failing
- Panic in cmd! proc-macro

[unreleased]: https://github.com/leandros/ferrunix/compare/v0.4.0..HEAD
[0.3.0]: https://github.com/leandros/ferrunix/compare/v0.2.0..v0.3.0
[0.3.1]: https://github.com/leandros/ferrunix/compare/v0.3.0..v0.3.1
[0.3.2]: https://github.com/leandros/ferrunix/compare/v0.3.1..v0.3.2
[0.3.3]: https://github.com/leandros/ferrunix/compare/v0.3.2..v0.3.3
[0.3.4]: https://github.com/leandros/ferrunix/compare/v0.3.3..v0.3.4
[0.4.0]: https://github.com/leandros/ferrunix/compare/v0.3.4..v0.4.0
