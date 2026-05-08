# Changelog — intervalsets

All notable changes to the `intervalsets` crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This crate ships in lockstep with `intervalsets-core`; both share the workspace
version and are released together via `cargo-release`. See the repo
[CONTRIBUTING.md](../CONTRIBUTING.md) for the versioning policy.

## [Unreleased]

### Added

- Optional `approx` feature with `AbsDiffEq` / `RelativeEq` / `UlpsEq` impls for `Interval` and `IntervalSet` ([#215](https://github.com/gechelberger/intervalsets/pull/215)).
- `error::Error::InvalidBoundLimit` variant lifted from `intervalsets-core` for the new `Element::validate` rejection path.

### Changed

- **Behavioral break (inherited from `intervalsets-core`):** `Interval` / `IntervalSet` factory paths now reject `±INF` and `NaN` at construction for `f32`/`f64`/`OrderedFloat<f*>`/`NotNan<f*>`. Previously `±INF` silently produced a non-canonical interval. Code that constructed intervals from arbitrary floats should filter via `is_finite()` before construction or handle `Err(Error::InvalidBoundLimit)` from the `try_*` variants.

### Deprecated

### Removed

### Fixed

### Security
