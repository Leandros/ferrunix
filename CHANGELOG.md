# Changelog

All notable changes of this project will be documented in this file.
This project is following [semantic versioning](http://semver.org), and the format
of the changelog is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Deprecated features will be kept for any following maintenance release, and
will be removed after two major releases.

## [0.3.0] - UNRELEASED

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
- Ci badge
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


