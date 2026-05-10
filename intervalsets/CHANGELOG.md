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
- `error::Error::Math(MathError)` variant (with `From<MathError> for Error`) so set-level `try_*` math can surface value-level arithmetic failures. `MathError` is re-exported from `intervalsets_core::error`.

### Changed

- **Behavioral break (inherited from `intervalsets-core`):** `Interval` / `IntervalSet` factory paths now reject `±INF` and `NaN` at construction for `f32`/`f64`/`OrderedFloat<f*>`/`NotNan<f*>`. Previously `±INF` silently produced a non-canonical interval. Code that constructed intervals from arbitrary floats should filter via `is_finite()` before construction or handle `Err(Error::InvalidBoundLimit)` from the `try_*` variants.
- **Behavioral break (inherited from `intervalsets-core`):** Set-level math on `Interval` / `IntervalSet` is re-bound from `T: Add<Output = T>` (etc.) to `T: TryAdd<Output = T>` (etc.) via the wrapped `EnumInterval`. `try_add` / `try_sub` / `try_mul` / `try_div` now propagate value-level overflow / non-finite / divide-by-zero into `Err(Error::Math(MathError::Range | Domain))`. Infix `+` / `-` / `*` / `/` is panicking sugar over `try_op().unwrap()` and **may panic in release** when the underlying `try_*` would have returned `Err` — including integer overflow, signed `iN::MIN / -1`, and float `INF` / `NaN` results. The previous `T: Ord` requirement on infix is dropped; the bound now requires that the wrapped `EnumInterval`'s `Try*` impl is available with `Error: Debug`. `Zero` / `One` impls on `Interval` / `IntervalSet` pick up the new `Self: Add<Self, Output = Self>` / `Self: Mul<Self, Output = Self>` chain instead of the previous `T: Ord` requirement.

### Deprecated

### Removed

- **Removed** `IFactory<T, C>` and `ISFactory<T, C>` (the parameterized factory marker types), along with the underlying `Converter` machinery in `intervalsets-core`. Migration: construct wrapped types directly at the call site — `Interval::closed(NotNan::new(0.0).unwrap(), NotNan::new(10.0).unwrap())` or `Interval::closed(OrderedFloat::from(0.0), OrderedFloat::from(10.0))`.
- **Removed** the `Error::TotalOrderError(TotalOrderError)` variant from the umbrella `Error`. `From<TotalOrderError> for Error` now collapses to `Error::InvalidBoundLimit`. The `TotalOrderError` struct re-export is unchanged. Migration: replace `Err(Error::TotalOrderError(_))` matchers with `Err(Error::InvalidBoundLimit)`.

### Fixed

### Security
