# Changelog — intervalsets-core

All notable changes to the `intervalsets-core` crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This crate ships in lockstep with `intervalsets`; both share the workspace
version and are released together via `cargo-release`. See the repo
[CONTRIBUTING.md](../CONTRIBUTING.md) for the versioning policy.

## [Unreleased]

### Added

- Optional `approx` feature with `AbsDiffEq` / `RelativeEq` / `UlpsEq` impls for `FiniteBound`, `FiniteInterval`, `HalfInterval`, `EnumInterval`, and `MaybeDisjoint` ([#215](https://github.com/gechelberger/intervalsets/pull/215)).
- `error::MathError` enum (`Range` / `Domain`) for value-level arithmetic failure, plus `From<Infallible> for MathError` and a new `Error::Math` variant. ([#240](https://github.com/gechelberger/intervalsets/pull/240))
- try_op impls for lib supported bound of set types and `Option<T>` wrapper. ([#240](https://github.com/gechelberger/intervalsets/pull/240))
- Validate 'FiniteBound' using 'Element::validate' with try_new and InvalidBoundLimit.

### Changed

- **Behavioral break:** Factory methods now reject `±INF` for `f32`/`f64`/`OrderedFloat<f*>`/`NotNan<f*>`. The fallible `try_*` variants return `Err(Error::InvalidBoundLimit)`; NaN handling is unchanged but now reports as `InvalidBoundLimit` rather than `TotalOrderError` for paths that funnel through the new chokepoint. `ConvertingFactory::Error` now requires `From<Error> + Debug` so factory-level convenience methods can propagate validation failures uniformly. All in-tree implementors already satisfy this.

### Deprecated

### Removed

### Fixed

### Security
