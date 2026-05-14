# Measure API Redesign

**Status:** implemented on branch `refactor/measure`. The shipped design diverges from the spec below on one point: the `Element` trait does not carry a `KindOps<T>` default-impl dispatch helper. Each T's impl provides `try_adjacent` and `try_measure_finite` directly, with the `DiscreteKind` / `ContinuousKind` markers used only for type-level kind narrowing. See the implementation notes at the bottom of this doc.
**Date:** 2026-05-13
**Revises:** the prior 2026-05-10 version (which split Cardinality + Width into two parallel kind-gated traits); this version unifies them into a single `Measure` trait.
**Drives:** the api/domain merge/revert decision — outcome: Kind machinery built from scratch in this PR, no `KindOps` dispatch.

---

## Problem statement

The current measure module has three latent issues:

1. **Width on integers is a mathematical category error.** `width([10, 100]: i32) = 90` today, but `[10, 100] ∩ ℤ` is a countable set with Lebesgue measure 0. The docstring carries a caveat about "discrete normalization" rather than rejecting the call outright.

2. **Width silently violates its `Measurement` contract.** `Width` bounds on `Sub`, which means float overflow produces `Measurement::Finite(f32::INFINITY)` — a lie. `Measurement::Infinite` is supposed to mean "set is unbounded," not "arithmetic overflowed." Three distinct states are wearing two masks.

3. **`Count` collides with `Iterator::count`.** Method chains like `a.complement().count()` are ambiguous between the iterator combinator and the cardinality measure. README drafting surfaced this; a rename to `Cardinality` was parked.

A fourth issue is structural: `Width` and `Countable` are parallel per-T traits that don't acknowledge the deeper structural distinction the crate already encodes (discrete vs continuous via the api/domain `Element::Kind` machinery). This redesign collapses Width and Cardinality into a single `Measure` trait whose output is the natural measure of T (cardinality for discrete, Lebesgue width for continuous).

---

## Decision summary

```
Element { type Kind; type Measure; try_adjacent_or_none; try_measure_finite }
  ├─ DiscreteElement: Element<Kind=DiscreteKind>           (pure Kind marker)
  └─ ContinuousElement: Element<Kind=ContinuousKind>       (pure Kind marker)


measure/mod.rs                   Measure       bounds on T: Element
ops/span.rs                      Span          bounds on T: TrySub (any T)
```

- `Count` / `Cardinality` / `Width` → unified `Measure`; method `.count()` → `.measure()`. Clean break.
- `.measure()` returns `Extent<T::Measure>` — cardinality count for discrete T, Lebesgue width for continuous T. Same trait, same call site, kind-projected output.
- `Span` stays in `ops/`, unchanged. Integer users who used to call `.width()` for `b−a` call `.span()` now.
- Per-T cardinality and width *computations* are no longer separate methods — both fold into `Element::try_measure_finite`.
- The unified `Measure` is Tier 3 — `try_measure` + panicking `measure` sugar — returning `Result<Extent<T::Measure>, MathError>`.
- Three states per call: `Ok(Finite)` / `Ok(Infinite)` (structural for Half/Unbounded) / `Err(overflow)`.

---

## Architecture

### Trait hierarchy

The Kind machinery was built from scratch in this PR (the `api/domain` branch the earlier drafts referenced was never merged). The shipped Element trait carries `type Kind`, `type Measure`, and a required `try_measure_finite` primitive; the discrete/continuous split is named at the type level via sealed marker types, without a `KindOps` dispatch helper.

```rust
// numeric/element.rs (as shipped)

mod sealed { pub trait Kind {} }

pub struct DiscreteKind;   impl sealed::Kind for DiscreteKind {}
pub struct ContinuousKind; impl sealed::Kind for ContinuousKind {}

pub trait Element: Sized + PartialEq + PartialOrd {
    type Kind: sealed::Kind;
    type Measure: Zero
        + TryAdd<Self::Measure, Output = Self::Measure>
        + Clone;

    fn try_adjacent(&self, side: Side) -> Option<Self>;

    /// Compute the natural measure of `[left, right]` (inclusive).
    /// - Discrete T: cardinality (e.g. `right - left + 1` for primitive integers).
    /// - Continuous T: Lebesgue width, i.e. `right - left`.
    /// `None` ⇒ representation overflow. (There is no "uncountable"
    /// sentinel — the natural measure on continuous T is width, not
    /// cardinality, so the uncountability distinction never enters.)
    fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure>;

    fn validate(self) -> Option<Self> {
        self.partial_cmp(&self).map(|_| self)
    }
}

pub trait DiscreteElement: Element<Kind = DiscreteKind> {}
impl<T: Element<Kind = DiscreteKind>> DiscreteElement for T {}

pub trait ContinuousElement: Element<Kind = ContinuousKind> {}
impl<T: Element<Kind = ContinuousKind>> ContinuousElement for T {}
```

Earlier drafts of this doc proposed a `KindOps<T>` helper that would
let `Element` provide a default `try_adjacent_or_none` body
dispatched through `Self::Kind`. The shipped implementation skips
`KindOps`: each T impl writes its own `try_adjacent` directly, and
the Kind markers are used purely for `where`-clause narrowing
(`where T: DiscreteElement` or `where T: Element<Kind = DiscreteKind>`).
This is the smallest machinery that supports the unified `Measure`
trait; `KindOps`-style default dispatch can be reintroduced later
without breaking existing impls if a future use case warrants it.

`DiscreteElement` and `ContinuousElement` are pure Kind markers — they add no bounds beyond `Element` and exist for `where`-clause ergonomics and documentation (`where T: DiscreteElement` reads better than `where T: Element<Kind = DiscreteKind>`).

**Asymmetries (intentional):**

- `Element` bounds on `PartialEq + PartialOrd`, not `Eq + Ord`. This serves floats (NaN-bearing, !Ord) and any PartialOrd-only discrete type (Gaussian integers, multi-dimensional integer grids, power-set-discrete posets). Total order is **not** a property of discreteness — it's an orthogonal axis. Operations that require total order bound `+ Ord` at the call site, not on the marker trait.
- `Element::Measure` bounds `Zero + TryAdd + Clone`. One associated type per T, one bound chain, drives both the per-piece result and the `IntervalSet` fold so `IntervalSet::try_measure` has a one-line where-clause.
- `DiscreteElement` and `ContinuousElement` are intentionally empty markers. Their job is to name the Kind disjunction at bound sites, not to gate capability.

### Measures

```rust
// measure/mod.rs

pub trait Measure {
    type Output;
    type Error: core::error::Error;

    fn measure(&self) -> Extent<Self::Output> {
        self.try_measure().unwrap()
    }

    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error>;
}

impl<T: Element> Measure for FiniteInterval<T> {
    type Output = T::Measure;
    type Error = MathError;

    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self.view_raw() {
            None => Ok(Extent::Finite(T::Measure::zero())),
            Some((l, r)) => T::try_measure_finite(l.value(), r.value())
                .map(Extent::Finite)
                .ok_or(MathError::Range),
        }
    }
}

impl<T: Element> Measure for HalfInterval<T> {
    type Output = T::Measure;
    type Error = MathError;
    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error> {
        Ok(Extent::Infinite)
    }
}

// EnumInterval dispatches (Unbounded → Infinite);
// MaybeDisjoint folds per-piece extents via TryAdd, propagating Err and Infinite.
```

*Naming.* The trait is named `Measure` to express the math directly. `.measure()` returns the natural measure of the type — cardinality for discrete T, Lebesgue width for continuous T. The names `cardinality` and `width` are not exported as method aliases; users who want the descriptive name in call sites should rebind locally (`let cardinality = iv.measure();`) or read the type's documentation to confirm what `.measure()` computes for their T.

*Semantics on concrete types:*

| Interval                    | `.measure()` result            |
|-----------------------------|--------------------------------|
| empty                       | `Finite(0)`                    |
| `[0, 10]: i32`              | `Finite(11_u64)`  (cardinality, widened to `i32::Measure = u64`) |
| `[i128::MIN, i128::MAX]: i128` | `Err(MathError::Range)`     |
| `[0, ∞): i32`               | `Infinite`                     |
| `[5.0, 5.0]: f64`           | `Finite(0.0)` (Lebesgue width of a singleton on ℝ is 0) |
| `[0.0, 10.0]: f64`          | `Finite(10.0)`                 |
| `[0.0, ∞): f64`             | `Infinite`                     |
| `[0,5]: i32 ∪ [10,20]: i32` | `Finite(17_u64)` (sum of counts) |

The continuous-singleton case is a deliberate semantic change from the pre-unification `Cardinality`: today's `cardinality([5.0, 5.0])` returns `Finite(1)` (counting-measure interpretation); under the unified trait `.measure([5.0, 5.0])` returns `Finite(0.0)` because the natural measure on continuous T is Lebesgue width. Users who need "is this a single point?" check `measure == 0` or use a dedicated predicate. The counting-on-continuous distinction (`{0, ℵ₁}`) was always degenerate and is dropped.

#### `T::Measure` type convention

A measure is by definition a non-negative magnitude. The type system can't easily bound `Measure: Unsigned` (it would exclude `f32`/`f64`/`Decimal`/`BigDecimal`, which are signed types we use non-negatively by convention), so the convention is enforced per-impl rather than via a trait bound.

**Stepwise widening for integer primitives.** Each integer type widens its `Measure` by one bit-class to the next unsigned type. Width always fits; cardinality also fits except at the literal `[MIN, MAX]` edge of 128-bit types, which surfaces as `Err(MathError::Range)`.

| `T` (primitive)   | `T::Measure` | Width range            | Cardinality range       |
|-------------------|---------------|-------------------------|--------------------------|
| `u8`, `i8`        | `u16`         | ≤ 255                   | ≤ 256                    |
| `u16`, `i16`      | `u32`         | ≤ 2¹⁶ − 1               | ≤ 2¹⁶                    |
| `u32`, `i32`      | `u64`         | ≤ 2³² − 1               | ≤ 2³²                    |
| `u64`, `usize`, `i64`, `isize` | `u128` | ≤ 2⁶⁴ − 1            | ≤ 2⁶⁴                    |
| `u128`, `i128`    | `u128`        | ≤ 2¹²⁸ − 1              | `[MIN, MAX]` = 2¹²⁸, overflows → `Err` |
| `f32`             | `f32`         | up to `f32::MAX` finite | (n/a — Lebesgue width)   |
| `f64`             | `f64`         | up to `f64::MAX` finite | (n/a — Lebesgue width)   |

`usize` / `isize` widen to `u128` rather than to a platform-dependent type so the API surface is stable across 32-bit and 64-bit targets.

**Why stepwise (vs uniform `u128`).** Two reasons:

1. **Embedded performance.** On targets without native 128-bit arithmetic (Cortex-M, RISC-V RV32, anything sub-64-bit), `u128` operations are software-emulated and ~2× the cost of `u64` and several times the cost of `u32`. A tight loop over `interval.measure()` on `i32` data pays this on every call. Stepwise widening keeps the arithmetic in native or near-native widths.
2. **Fold-overflow headroom.** `IntervalSet::try_measure` sums per-piece measures via `TryAdd`. With same-width unsigned (e.g. `i32` → `u32`), summing two `[i32::MIN, 0]` + `[1, i32::MAX]` cardinalities exactly overflows `u32`. Stepwise widening (`i32` → `u64`) leaves ~32 bits of headroom — enough that fold-overflow is practically unreachable.

**Float primitives stay self-typed.** No native float widening exists in stable Rust; `f32 → f32`, `f64 → f64`. Overflow to non-finite surfaces as `Err(MathError::Range)`, same as today.

**Custom types pick their own.** `BigInt::Measure = BigUint`, `Decimal::Measure = Decimal`, fixed-point types use the smallest unsigned integer that fits their representable width. The trait bounds (`Zero + TryAdd + Clone`) are necessary; the non-negativity and one-step-widening conventions are documentary.

### Span

```rust
// ops/span.rs

/// The diameter sup − inf of a set's bounds.
///
/// NOT a measure: fails subadditivity on disjoint sets
/// (span([0,1] ∪ [3,4]) = 4, but Σ span = 2).
///
/// For the additive Lebesgue/counting measure of a set, see [`Measure`].
pub trait Span {
    type Output;
    type Error: core::error::Error;
    fn span(&self) -> Extent<Self::Output> { self.try_span().unwrap() }
    fn try_span(&self) -> Result<Extent<Self::Output>, Self::Error>;
}

// impls require for<'a> &'a T: TrySub<&'a T, Output = ...>
// Works on integer T (sup - inf as i32) and continuous T (sup - inf as Duration).
```

Span is the right tool for users who want "max − min on any T." It subsumes today's integer-Width use case and decouples diameter from measure-theory contracts.

### Three-state return semantics

Every user-facing measure call returns `Result<Extent<X>, Err>`:

| Outcome | Return |
|---|---|
| Finite-bounded interval, arithmetic succeeded | `Ok(Extent::Finite(_))` |
| Half-bounded or unbounded set | `Ok(Extent::Infinite)` |
| Finite-bounded, primitive op overflowed (e.g. `[i128::MIN, i128::MAX].try_measure()`) | `Err(_)` |

Internally, `Element::try_measure_finite` returns `Option<T::Measure>` (`None` = primitive overflow); the `Measure` impl wraps `None` into `Err(MathError::Range)`. `Extent::Infinite` is constructed structurally by `HalfInterval` and `EnumInterval::Unbounded`, never as a fallback for overflow.

### `Measurement` → `Extent`: rename and cleanup

Rename `Measurement<T>` to `Extent<T>` and address accumulated shortcomings in the current implementation. The cleanup ships in Stage 2 alongside the trait rename.

#### Shortcomings of the current `Measurement<T>`

1. **It is `Option<T>` with renamed methods.** `Finite(t)` ≡ `Some(t)`, `Infinite` ≡ `None`. The combinator API (`is_finite`, `finite`, `finite_or`, `expect_finite`, `map`, `flat_map`) is std-`Option` repackaged with novel names. Many useful `Option` combinators are absent (`unwrap_or_else`, `unwrap_or_default`, `or`, `or_else`, `as_ref`, `iter`/`into_iter`, `ok_or`, ...).

2. **The `Sub` impl has no measure-theoretic justification.** Measures aren't a group; `m(A) − m(B)` is well-defined only when `B ⊆ A`. The current impl computes `Infinite - Finite = Infinite`, `Infinite - Infinite = Infinite` — literal-infinity arithmetic, not measure arithmetic. No in-crate caller uses it.

3. **The `Add` impl is Tier-2-shaped, but the new design is Tier-3.** Current impl bounds `T: Clone + Add<T, Output=T>` and unwraps via `binop_map`. The dead-code `binop_try_map` at [intervalsets-core/src/measure/mod.rs:168](../intervalsets-core/src/measure/mod.rs#L168) — flagged `#[allow(dead_code)]` — is the seed of the `TryAdd` impl that should exist for fold-with-overflow-propagation.

4. **The `Clone` bound on `Add`/`Sub` is unnecessary.** `binop_map` consumes `self` and `rhs` by value; the closure receives `T` by value. The `T: Clone` bound is defensive and unused — should be just `T: Add<T, Output=T>`.

5. **`PartialOrd` is derived implicitly via variant order.** `Finite` declared first → `Finite(_) < Infinite`. Reordering variants silently flips the ordering. Plus, with today's `Width` producing `Finite(f32::INFINITY)` on overflow, `partial_cmp` says "finite-overflow < unbounded-set" — meaningless. Tier-3 migration fixes the corruption upstream, but the implicit derive remains brittle.

6. **`Infinite` is unsigned.** Closes the door on signed measures without a future breaking change. Deferred per appendix but worth flagging.

7. **No `Option<T>` interop.** No `From<Measurement<T>> for Option<T>` or vice versa. Users threading a measure through `Option`-shaped APIs (`.ok_or(err)?`) write the match themselves.

8. **Panic-prefixed naming doesn't match std.** `expect_finite(msg)` / `finite()` instead of `expect(msg)` / `unwrap()` adds no information std's names lack.

#### Replacement: `Extent<T>` (final spec)

The spec landed differs from earlier drafts of this doc on two
points: (1) `finite()` is **kept** as the panicking accessor (the
domain-named "give me the finite value, panic if it's infinite" is
more informative than `unwrap`); (2) `From<Option<T>>` **is**
provided in addition to `From<Extent<T>>` (the `None ≡ Infinite`
mapping is canonical by fiat). The full spec lives in
`/home/greg/.claude/plans/lets-come-up-with-proud-swing.md`.

```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Extent<T> {
    Finite(T),
    Infinite,
}

impl<T> Extent<T> {
    pub fn is_finite(&self) -> bool;
    pub fn is_infinite(&self) -> bool;
    pub fn finite(self) -> T;                       // panics on Infinite
    pub fn try_binop_map<E>(self, rhs: Self,
        f: impl FnOnce(T, T) -> Result<T, E>)
        -> Result<Self, E>;                         // load-bearing internal
}

impl<T> From<Extent<T>> for Option<T>;             // Finite → Some; Infinite → None
impl<T> From<Option<T>> for Extent<T>;             // canonical None ≡ Infinite
impl<T: Add<T, Output = T>> Add for Extent<T>;     // Infinite absorbs
impl<T: TryAdd<T, Output = T>> TryAdd for Extent<T>; // Err preserved, NOT collapsed to Ok(Infinite)
impl<T: Display> Display for Extent<T>;            // Finite delegates; Infinite → "∞"
```

`IntervalSet::try_measure` folds via `try_binop_map` over per-piece
extents, propagating both `Err` and `Infinite` correctly.

`Sub` / `Mul` / `Div` / `Neg` and their `Try*` siblings are intentionally
not implemented. The three-state contract (`Ok(Finite)` / `Ok(Infinite)` /
`Err`) is preserved by `TryAdd` — overflow surfaces as `Err`, never
collapses into `Ok(Infinite)`. Saturate-on-overflow semantics belong to
the `core::num::Saturating<T>` storage type.

#### Shipped (this redesign)

- Rename `Measurement<T>` → `Extent<T>`, move to `measure/extent.rs`
- Add `Eq`, `Hash`, `Ord` to the derive set (T-conditional)
- Keep `finite()` / `is_finite()` / `is_infinite()` / `try_binop_map()`
- Drop `Sub`, `expect_finite`, `finite_or`, `flat_map`, `map`
- Drop dead `T: Clone` bound on `Add`
- Add bidirectional `From` with `Option`
- Add `TryAdd` impl (no Range-error collapse to `Ok(Infinite)`)
- Add `Display` impl (`Infinite` prints as `"∞"`)

#### Deferred (won't address now)

- Signed-measure refactor (`PosInfinite` / `NegInfinite` variants)
- `IntoIterator`, `Default` impls
- Renaming `try_binop_map` (e.g. to `try_zip_with`)
- `unwrap_or_else`, `as_ref`, etc. — users access via `Option::from(extent)`

#### Why `Extent<T>` and not just `Option<T>`?

A type alias (`pub type Extent<T> = Option<T>;`) gets the full
`Option` ecosystem for free, zero code to maintain. **But the domain
type pays for itself in rustdoc.** `try_measure() -> Result<Extent<u64>, MathError>`
reads as "this might fail to compute (Err) OR tell you the set is
unbounded (Infinite)." `Result<Option<u64>, _>` reads as "this might
fail to compute (Err) OR have no value (None)" — true, but the "set
is unbounded" semantic is lost. Domain types carry meaning that
aliases erase. Bidirectional `From` lets callers reach into the
`Option` combinator ecosystem when they want it, without sacrificing
the domain naming.

---

## What survives unchanged

- `Extent<T>` (renamed from `Measurement<T>`) keeps its core `Finite(T)` / `Infinite` shape and the existing tests; surrounding combinator/impl surface gets the cleanup described above
- `Element` trait, `try_adjacent`, and the new `Kind` markers (`DiscreteKind` / `ContinuousKind` + `DiscreteElement` / `ContinuousElement` blanket-impl'd marker traits)
- The four-tier op contract documented in `ops/mod.rs`
- `OrderedFloat`, `Decimal`, `BigInt`, `BigDecimal` feature modules (each gets a categorical impl)
- All other ops (Union, Intersection, etc.)

## Breaking changes (Stage 2)

| Before | After |
|---|---|
| `Count`, `Cardinality`, `Width` traits | unified `Measure` trait |
| `.count()` / `.try_count()` | removed |
| `.cardinality()` / `.try_cardinality()` | `.measure()` / `.try_measure()` |
| `.width()` / `.try_width()` | `.measure()` / `.try_measure()` for continuous T; `.span()` for "diameter on any T" |
| `Countable` trait + `IS_CONTINUOUS` flag | absorbed: `Element::Measure` + `Element::try_measure_finite` |
| `Widthable` trait | absorbed (same as above) |
| `CountOverflowError`, `WidthOverflowError` | already unified into `MathError` (no further change) |
| `default_countable_impl!`, `default_width_impl!`, `continuous_countable_impl!` | unified: `default_discrete_element_impl!`, `default_continuous_element_impl!` |
| `integer_domain_impl!` | `default_discrete_element_impl_primitives!` |
| `continuous_domain_impl!` | `default_continuous_element_impl!` |
| `[5.0_f64, 5.0].cardinality() == Finite(1)` | `[5.0_f64, 5.0].measure() == Finite(0.0)` ← Lebesgue-width semantics on continuous singletons |
| `[0_i32, 10].width() == Finite(10_u128)` | no longer exists; users call `.span()` (`= 10_i32`) or `.measure()` (`= 11_u64` under stepwise widening) |
| `Measurement<T>` type | `Extent<T>` |
| `.finite()` on the value | unchanged (kept; domain-named panicking accessor) |
| `.expect_finite(msg)` on the value | `Option::from(x).expect(msg)` |
| `.flat_map(f)` on the value | `Option::from(x).and_then(...).into()` |
| `.finite_or(default)` on the value | `Option::from(x).unwrap_or(default)` |
| `.map(f)` on the value | `Option::from(x).map(f).into()` |
| `impl Sub for Measurement<T>` | removed (no measure-theoretic meaning) |
| `impl Add for Measurement<T>` requiring `T: Clone` | `impl Add` drops the `Clone` bound; `impl TryAdd` added for Tier-3 folds |

Clean break, no deprecation aliases. CHANGELOG entry must point both integer-`width` users at `span` (semantically equivalent for single intervals; differs only on disjoint sets where users will want `.measure()` anyway) AND `cardinality` / `width` users at the new unified `.measure()` spelling.

---

## Migration plan

| Stage | Scope | Status |
|---|---|---|
| 1 | Kind machinery (`type Kind`, `DiscreteKind` / `ContinuousKind` markers, `DiscreteElement` / `ContinuousElement` blanket-marker subtraits) | **Shipped** in this PR — built from scratch rather than merging the parked `api/domain` branch. No `KindOps` dispatch helper; each T's impl writes its own `try_adjacent` and `try_measure_finite` directly. |
| 2 | Unify Cardinality + Width into `Measure`, absorb the computation into `Element::Measure` + `Element::try_measure_finite`, public macro `default_continuous_element_impl!`, helper fn `default_discrete_count_inclusive`, `Measurement` → `Extent` rename + cleanup (drop Sub, add TryAdd, std-naming combinators, Option interop), `IntervalSet` folds via `try_fold` | **Shipped** in this PR. |
| 3 | `Span` in `ops/span.rs` | Already in tree (predates this PR). Integer users wanting `b − a` in the native type route through `.span()`. |
| 4 | Bisect collapse: drop closure, bound on `Measure`, single-interval split-at-midpoint | **Shipped** in this PR. |

---

## Documentation responsibilities

- **`measure/mod.rs`** — now hosts the `Measure` trait and retains the monotonicity + subadditivity contract. Update the copy from "Some common measures are Cardinality, Count, and the Lebesgue measure" to "`Measure` returns the natural additive measure of a set: cardinality on discrete types, Lebesgue width on continuous types."
- **`ops/span.rs`** — single cross-reference: *"Span is the diameter sup − inf. NOT a measure: fails subadditivity on disjoint sets. For the additive Lebesgue/counting measure of a set, see [`Measure`]."*
- ~~`measure/cardinality.rs`~~ and ~~`measure/width.rs`~~ — files cease to exist; their tests fold into the unified `Measure` impl tests.
- **CHANGELOG** — migration table mirroring the breaking-changes section above. Point both integer-`.width()` users at `.span()` AND discrete-`.cardinality()` / float-`.cardinality()` users at `.measure()`.
- **Book** — drop the "which measure do I want?" decision tree (the unification makes the decision for them). Add a "what does `.measure()` return for my T?" table mirroring the semantics table under §Architecture > Measures.

---

## Deferred

### Midpoint

Stays `pub(crate)` through Stage 2. The open question of `Result<Self, Self::Error>` vs `Option<Self>` revisits at the Bisect PR (Stage 4) where actual call sites will reveal whether the typed error pulls its weight. See [`project_midpoint_error_vs_option_open.md`](../../.claude/projects/c--Users-Admin-repos-intervalsets/memory/project_midpoint_error_vs_option_open.md).

### Set-level cast / `Widen`

The `Widen` trait from the original design summary is **dropped** — `T::Measure` plays two different roles depending on T's Kind, and a single per-T `Wider` type can't simultaneously serve both. For continuous T it is the displacement type (e.g. `DateTime::Measure = Duration`); for discrete T it is the arithmetic-headroom type (e.g. `i32::Measure = u64` under stepwise widening). These roles conflate affine-point semantics with overflow-prevention; forcing one trait to bear both was the original design error.

Users who want `i32` cardinality expressed as `u128`, or `f32` width as `f64`, are best served by a future set-level cast API — `IntervalSet<i32>::cast::<i64>()` style. That's a parallel PR not coupled to this design.

`Extent::map` is **not** an adequate substitute for caller-driven widening: the cast happens *after* subtraction, so any overflow already corrupted the result.

### Continuous types without meaningful `Measure`

Every continuous type must commit to a `T::Measure` (its Lebesgue-width displacement type). No in-crate continuous type lacks one today; if one appears, re-evaluate splitting back to a 3-tier `Continuous` marker + `ContinuousElement: Continuous` (where the inner trait carries `Measure`).

---

## Appendix: Measure-as-codomain (not in-crate)

Earlier drafts of this doc proposed two parallel measure traits keyed on Element kind. The shipped design instead uses a single `Measure` trait with `Output = T::Measure`; the codomain (cardinality type or displacement type) is committed by the T impl, and the Kind machinery picks the right interpretation at the computation site.

The math allows further axes — signed measures (`m: Set → ℝ`), complex-valued measures (`m: Set → ℂ`), or vector-valued measures (`m: Set → ℝⁿ`) — that would parameterize over their codomain. The crate has no in-tree use case for these. Users who need them define their own trait alongside `Measure` (e.g. `pub trait SignedMeasure { type Output; fn signed_measure(&self) -> Result<Self::Output, ...> }`) and impl it directly on the container types they care about. The `Measure` trait does not need to grow to accommodate; orthogonal user traits compose naturally with the Kind machinery via `where T: Element<Kind = ...>` bounds.

---

## Why the asymmetries are correct

1. **`Measure` is one trait, not two.** Earlier drafts gated `Width` and `Cardinality` separately by Element kind, on the theory that the math forbade cross-category calls. The math actually says each T has *one* natural measure (cardinality for discrete, Lebesgue width for continuous), and the Kind machinery already disambiguates. A single `Measure` trait with `Output = T::Measure` expresses this directly; the gating becomes redundant.

2. **`Span` is not a `Measure`.** Span fails subadditivity on disjoint sets (`span([0,1]∪[3,4]) = 4`, but the additive measure is `2`). It is the diameter `sup − inf`, not an integral. Keeping it in `ops/` rather than `measure/` is the contract-enforcing choice. `Span` bounds directly on `TrySub` — Span's whole point is being available on any T with subtraction, so a categorical gate would defeat it.

3. **Discreteness and total order are orthogonal.** Earlier drafts wrote `DiscreteElement: ... + Ord` on the basis that every in-crate discrete primitive is Ord. But discreteness is about adjacency — every element has a unique successor/predecessor under the type's order — not about totality. PartialOrd-only discrete types (Gaussian integers, integer lattices, power-set posets) are legitimate; pinning Ord onto the marker would exclude them for no structural reason. `Element` bounds on `PartialOrd + PartialEq` for the same reason it doesn't bound on `Ord` for floats: total order is its own axis. Operations that need it bound `+ Ord` locally.

4. **Bound-validity has a known leak we're not closing here.** `Element` says "this type can appear as an interval bound." But `Extent<T>` derives `Ord` when `T: Ord`, which means `Extent<u32>` would type-check as a bound — yet `Extent::Infinite` doesn't represent a real element, only the absence of one. The current trait surface relies on convention to keep nonsense bounds out. A future revision could introduce a `BoundEligible` marker that gates "this `Element` impl is meaningful as a bound type," but that's parallel to (and not blocked by) this redesign.

The unifying principle is **"each operation expresses its math directly."** `Measure` is the additive measure; `Span` is the diameter; the Kind machinery names the disjunction between discrete and continuous element types without forcing extra bounds. Naming the math directly catches the category errors users would otherwise make without inventing artificial asymmetries.
