# Changelog — intervalsets

All notable changes to the `intervalsets` crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This crate ships in lockstep with `intervalsets-core`; both share the workspace
version and are released together via `cargo-release`. See the repo
[CONTRIBUTING.md](../CONTRIBUTING.md) for the versioning policy.

## [Unreleased]

### Added

- `ops::Bisect<T>` impls for `Interval<T>` and `IntervalSet<T>` (trait + `Bisection<T, S>` re-exported from `intervalsets-core`). Same closure-driven API as the core impls — `bisect_by(closed, |s| s.width().finite())` for width-balance, `bisect_by(closed, |s| s.cardinality().finite())` for population, or any monotone-in-cut measure. `Interval` extracts bounds via its `SetBounds` impl; `IntervalSet` routes through its inherent `hull()` accessor. Both return `None` for empty / unbounded inputs.
- `core::num::Saturating<T>` is now a valid storage type (inherited from `intervalsets-core`). All four `Try*` ops are uniformly `Error = Infallible` — saturation is defined behavior on the bounded integer lattice, not overflow. `core::num::Wrapping<T>` is intentionally not supported (ordering-flipping overflow violates the bound-pair invariant). See the `intervalsets-core` changelog for the full surface, `TryDiv` saturation rule, and orphan-rule limitation around `IntoFiniteInterval`.
- `cast::{Cast, LossyCast, TryCast}` impls for `Interval` and `IntervalSet` (re-exported via prelude). `Interval` delegates to its inner `EnumInterval`. `IntervalSet::cast` routes through `try_new` (strict; widenings preserve invariants); `IntervalSet::lossy_cast` routes through `new` (repairing — narrowed intervals that collapse onto the same range merge); `IntervalSet::try_cast` routes through `try_new` and surfaces cast-induced invariant violations as `Error::InvalidIntervalSet`. Coverage extends to `ordered-float`'s `OrderedFloat<T>` and `NotNan<T>` storage types via the impls landed in `intervalsets-core`.
- `ops::Midpoint<T>` impls for `Interval<T>` and `IntervalSet<T>` (trait re-exported from `intervalsets_core::ops`). Hull-midpoint semantics: `(inf(S) + sup(S)) / 2`. For a single connected `Interval` this is the interval's midpoint; for a multi-piece `IntervalSet` it is the midpoint of the convex hull and **may lie in a gap** between components — see the trait docs. Empty / half-bounded / unbounded inputs return `Err(Error::Math(MathError::Domain))`. Wrapper impls use `type Error = crate::error::Error` so the wrapper's umbrella error type carries the result.
- Re-export of `numeric::Midpointable` (the storage-type trait, renamed from `numeric::Midpoint` in core) and `measure::Widthable`. The set-level `Width`/`Cardinality` impls for `Interval` / `IntervalSet` use `type Error = MathError`.
- `Width::try_width` surfaces representation overflow (e.g. `[i32::MIN, i32::MAX]` widening, `f64::MIN..f64::MAX` overflow to `±INF`) as `Err(MathError::Range)`. The infallible `width()` panics on overflow per its docstring.
- `IntervalSet::try_cardinality` and `IntervalSet::try_width` summations now use `TryAdd`-based folds and surface mid-fold overflow as `Err(MathError::Range)`.
- `Cardinality` now supported on continuous element types (`f32`, `f64`, `Decimal`, `BigDecimal`, `OrderedFloat<F>`, `NotNan<F>`) — inherited from `intervalsets-core`. Singleton intervals have cardinality `1`; non-degenerate continuous intervals have cardinality `Extent::Infinite`; empty stays `0`.
- `Cardinality` now supported on all fixed-point storage types (`FixedI{8,16,32,64,128}<Frac>`, `FixedU{8,16,32,64,128}<Frac>`) — inherited from `intervalsets-core`. Cardinality is the bit-rep diff + 1 via `to_bits()`; 128-bit full-range cardinalities surface as `Err(MathError::Range)`.
- Optional `approx` feature with `AbsDiffEq` / `RelativeEq` / `UlpsEq` impls for `Interval` and `IntervalSet` ([#215](https://github.com/gechelberger/intervalsets/pull/215)).
- `error::Error::InvalidBoundLimit` variant lifted from `intervalsets-core` for the new `Element::validate` rejection path.
- `error::Error::Math(MathError)` variant (with `From<MathError> for Error`) so set-level `try_*` math can surface value-level arithmetic failures. `MathError` is re-exported from `intervalsets_core::error`.

### Changed

- **Breaking (inherited from `intervalsets-core`):** `measure::Count` renamed to `measure::Cardinality`; methods `count()` / `try_count()` renamed to `cardinality()` / `try_cardinality()`. Trait conflicted with `Iterator::count` in the prelude. `Countable` is unchanged. Migration: `s.count()` → `s.cardinality()`, `s.try_count()` → `s.try_cardinality()`, `use ...::Count` → `use ...::Cardinality`.
- **Behavioral break (inherited from `intervalsets-core`):** `Interval` / `IntervalSet` factory paths now reject `±INF` and `NaN` at construction for `f32`/`f64`/`OrderedFloat<f*>`/`NotNan<f*>`. Previously `±INF` silently produced a non-canonical interval. Code that constructed intervals from arbitrary floats should filter via `is_finite()` before construction or handle `Err(Error::InvalidBoundLimit)` from the `try_*` variants.
- **Behavioral break (inherited from `intervalsets-core`):** Set-level math on `Interval` / `IntervalSet` is re-bound from `T: Add<Output = T>` (etc.) to `T: TryAdd<Output = T>` (etc.) via the wrapped `EnumInterval`. `try_add` / `try_sub` / `try_mul` / `try_div` now propagate value-level overflow / non-finite / divide-by-zero into `Err(Error::Math(MathError::Range | Domain))`. Infix `+` / `-` / `*` / `/` is panicking sugar over `try_op().unwrap()` and **may panic in release** when the underlying `try_*` would have returned `Err` — including integer overflow, signed `iN::MIN / -1`, and float `INF` / `NaN` results. The previous `T: Ord` requirement on infix is dropped; the bound now requires that the wrapped `EnumInterval`'s `Try*` impl is available with `Error: Debug`. `Zero` / `One` impls on `Interval` / `IntervalSet` pick up the new `Self: Add<Self, Output = Self>` / `Self: Mul<Self, Output = Self>` chain instead of the previous `T: Ord` requirement.
- **Behavioral break (inherited from `intervalsets-core`):** `Width::Output` for primitive integer `T` widens to `u128`. `Interval::<i32>::closed(0, 10).width().finite()` now returns `10u128` instead of `10i32`; cast at the boundary if the narrower type is wanted. The widening fixes a debug-panic / release-wrap on `[i32::MIN, i32::MAX]`-shaped intervals.
- **Behavioral break (inherited from `intervalsets-core`):** `Width` / `IntervalSet::Width` bounds switch from `for<'a> &'a T: Sub<Output = Out>` and `Out: Add<Out, Output = Out>` onto `T: Widthable` and `Out: TryAdd<Out, Output = Out>`. Existing in-tree types and downstream users via `default_width_impl!` are unaffected; custom `T`s implementing `Width` via the old `Sub` bound need a `Widthable` impl.
- **Behavioral break (inherited from `intervalsets-core`):** `HalfInterval::Cardinality::Output` is now `T::Output` (was `()`). The wrapper's `Interval::cardinality()` was already `T::Output`-typed; the alignment closes the discrepancy when calling `.cardinality()` on a half-bounded `EnumInterval` directly.
- **Behavioral break:** `ops::IntoFinite` → `ops::IntoFiniteInterval` and `into_finite()` → `into_finite_interval()` (inherited from `intervalsets-core`). The `IntervalSet<T>` impl additionally changes its `Output` from `Self` to `Interval<T>` and its semantics from per-subinterval clamping to "hull of the set, then truncate to type extents" — the trait name promised a single finite interval but the previous impl returned a multi-piece `IntervalSet`. Migration: callers wanting the previous per-piece behavior should collect manually — `set.into_iter().map(IntoFiniteInterval::into_finite_interval).filter(MaybeEmpty::is_inhabited).collect::<IntervalSet<_>>()`. The new impl consumes via `OrdBoundPair::from(self)`, so no `T: Clone` bound.
- **Behavioral break (inherited from `intervalsets-core`):** `measure::Measurement<T>` → `measure::Extent<T>` (now living in its own `measure/extent.rs` submodule). Variant shape (`Finite(T)` / `Infinite`) unchanged. The inherent-method surface is trimmed — `finite`, `is_finite`, `is_infinite`, `try_binop_map` are kept; `expect_finite`, `finite_or`, `map`, `flat_map` are dropped in favor of routing through the new `From<Extent<T>> ↔ Option<T>` impls (use `Option::from(x).expect(msg)`, etc.). `TryAdd` and `Display` impls are now provided on `Extent<T>`. Migration: rename `Measurement` → `Extent` at the call site and substitute the `Option` combinators for the dropped methods.

### Deprecated

### Removed

- **Removed** `IFactory<T, C>` and `ISFactory<T, C>` (the parameterized factory marker types), along with the underlying `Converter` machinery in `intervalsets-core`. Migration: construct wrapped types directly at the call site — `Interval::closed(NotNan::new(0.0).unwrap(), NotNan::new(10.0).unwrap())` or `Interval::closed(OrderedFloat::from(0.0), OrderedFloat::from(10.0))`.
- **Removed** the `Error::TotalOrderError(TotalOrderError)` variant from the umbrella `Error`. `From<TotalOrderError> for Error` now collapses to `Error::InvalidBoundLimit`. The `TotalOrderError` struct re-export is unchanged. Migration: replace `Err(Error::TotalOrderError(_))` matchers with `Err(Error::InvalidBoundLimit)`.
- **Removed** `Extent::Sub` impl (inherited from core). The previous impl returned `Infinite` whenever either operand was `Infinite`, which is mathematically wrong.
- **Removed** `Extent::expect_finite`, `Extent::finite_or`, `Extent::map`, `Extent::flat_map` inherent methods (inherited from core). Migration: use `Option::from(extent)` and the std `Option` combinators.

### Fixed

- `IntervalSet::try_width` no longer panics in debug / wraps in release on integer intervals; the per-step `Sub` overflow on full-range integer intervals is gone (Output widens to `u128`), and the summation step uses `TryAdd` so a sum that exceeds `Out` surfaces as `Err(MathError::Range)`.
- `IntervalSet::try_cardinality` summation is now panic-free at every step (previously only the per-interval cardinality was checked; the fold-step `Add` panicked on overflow in debug / wrapped in release).

### Security
