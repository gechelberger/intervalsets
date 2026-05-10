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
- Validate `FiniteBound` using `Element::validate` with `try_new` and `InvalidBoundLimit`.
- New `TryFiniteFactory` and `TryHalfBoundedFactory` traits hold the fallible `try_*` constructors; `FiniteFactory` and `HalfBoundedFactory` are now panicking-only and blanket-implemented over their `Try*` siblings, so types only impl the fallible half.
- `TryAdd` / `TrySub` / `TryMul` / `TryDiv` impls on `FiniteBound<T>` (the previous `Add` / `Sub` / `Mul` impls become panicking sugar over the new `try_*` siblings; `TryDiv` and the matching `Div` sugar are net-new — `FiniteBound` had no division before).

### Changed

- **Behavioral break:** Factory methods now reject `±INF` for `f32`/`f64`/`OrderedFloat<f*>`/`NotNan<f*>`. The fallible `try_*` variants return `Err(Error::InvalidBoundLimit)`; NaN handling is unchanged but now reports as `InvalidBoundLimit` rather than `TotalOrderError` for paths that funnel through the new chokepoint. `ConvertingFactory::Error` now requires `From<Error>` so factory-level convenience methods can propagate validation failures uniformly. All in-tree implementors already satisfy this.
- Implementors of the factory traits now implement `TryFiniteFactory` / `TryHalfBoundedFactory` (the fallible halves) and pick up `FiniteFactory` / `HalfBoundedFactory` for free via blanket impl. External users with custom factory types must rename `fn finite`/`fn half_bounded` overrides to `fn try_finite`/`fn try_half_bounded` and drop the panicking method bodies.
- The factory traits dropped their second type parameter: `FiniteFactory<T, C = Identity>` is now `FiniteFactory<T>`, and likewise for `EmptyFactory`, `HalfBoundedFactory`, `UnboundedFactory`, `TryFiniteFactory`, `TryHalfBoundedFactory`. Implementors drop the trailing `, Identity` parameter from their impls.
- The shared base trait that declares `Output` / `Error` was renamed `ConvertingFactory` → `Factory` (the "Converting" prefix referred to the now-removed `Converter` trait).
- **Behavioral break:** Set-level math (`TryAdd` / `TrySub` / `TryMul` / `TryDiv` for `FiniteInterval` / `HalfInterval` / `EnumInterval` and the heterogeneous combinations) is re-bound from `T: Add<Output = T>` (etc.) to `T: TryAdd<Output = T>` (etc.). Bound math now propagates value-level overflow / non-finite / divide-by-zero through `T::TryAdd::Error` into `Error::Math(MathError)`. For primitive integers this means overflow surfaces as `Err(MathError::Range)` instead of silently wrapping in release; for primitive floats, `INF` / `NaN` results surface as `Err(MathError::Domain)`. The `iN::MIN / -1` panic that previously slipped through `Div` for signed integers in release mode is now caught here and reported as `Err(MathError::Range)`.
- **Behavioral break:** Infix `+` / `-` / `*` / `/` on set types (`FiniteInterval` / `HalfInterval` / `EnumInterval`) is now explicit panicking sugar over `try_op().unwrap()`. The previous `T: Ord`-backed "provably infallible" framing is dropped; the bound becomes `Self: TryAdd<Output = Self>` plus `<Self as TryAdd>::Error: Debug` (analogous for sub/mul/div). Infix **may panic in release** when the corresponding `try_*` returns `Err`. The panic site is part of the documented contract.
- `FiniteBound<T>` infix `Add` / `Sub` / `Mul` re-bound the same way: the bound flips to `T: TryAdd<Output = T>` (etc.) and the impl bodies become `try_op().unwrap()`. The output type tightens from `FiniteBound<<T as Add>::Output>` to `FiniteBound<T>` because `T: TryAdd<Output = T>` locks the inner output. `Zero` / `One` / `ConstZero` / `ConstOne` impls on `FiniteBound` and on `FiniteInterval` / `EnumInterval` pick up the new `Try*` bound chain transitively.
- Tier 3 documentation in `ops/mod.rs` is split into **Tier 3a** (`try_*`: total, panic-free in release, canary-verified) and **Tier 3b** (infix sugar: may panic in release and debug). The `ops/math/mod.rs` "Panicking and fallible forms" section is rewritten to drop the `T: Ord`-makes-unwrap-safe claim and frame infix as honest panicking sugar.

### Deprecated

### Removed

- **Removed** the `Converter` trait, the `Identity` converter, the `EIFactory<T, C>` type-level factory, and `ConvertingFactory::try_convert`. The trait was a tutorial-quality nicety for end users wanting to construct `OrderedFloat`/`NotNan`-wrapped intervals from raw `f32`/`f64` values; nothing internal used it. Migration: wrap the value directly at the call site — `EnumInterval::closed(NotNan::new(0.0).unwrap(), NotNan::new(10.0).unwrap())` or define a project-local helper. `OrderedFloat::from(value: T)` exists for the infallible case.
- **Removed** the `Error::TotalOrderError(TotalOrderError)` variant from the umbrella enum. `From<TotalOrderError> for Error` now collapses to `Error::InvalidBoundLimit` — same destination as `Element::validate` rejection, since both gates fire on the same root cause (a bound's value isn't a usable limit). The `TotalOrderError` struct itself is unchanged and remains the precise return type of `TryCmp::try_cmp`. Migration: replace `Err(Error::TotalOrderError(_))` matchers with `Err(Error::InvalidBoundLimit)`.

### Fixed

### Security
