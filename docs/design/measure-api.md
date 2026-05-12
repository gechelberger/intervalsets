# Measure API Redesign

**Status:** design committed, implementation pending Stage 1 (api/domain merge)
**Date:** 2026-05-10
**Drives:** the api/domain merge/revert decision

---

## Problem statement

The current measure module has three latent issues:

1. **Width on integers is a mathematical category error.** `width([10, 100]: i32) = 90` today, but `[10, 100] ∩ ℤ` is a countable set with Lebesgue measure 0. The docstring carries a caveat about "discrete normalization" rather than rejecting the call outright.

2. **Width silently violates its `Measurement` contract.** `Width` bounds on `Sub`, which means float overflow produces `Measurement::Finite(f32::INFINITY)` — a lie. `Measurement::Infinite` is supposed to mean "set is unbounded," not "arithmetic overflowed." Three distinct states are wearing two masks.

3. **`Count` collides with `Iterator::count`.** Method chains like `a.complement().count()` are ambiguous between the iterator combinator and the cardinality measure. README drafting surfaced this; a rename to `Cardinality` was parked.

A fourth issue is structural: `Width` and `Countable` are parallel per-T traits that don't acknowledge the deeper structural distinction the crate already encodes (discrete vs continuous via `Element::try_adjacent`). This redesign re-expresses the trait surface so that the math drives the types.

---

## Decision summary

```
Element { type Kind; try_adjacent_or_none(side) -> Option<Self> }   // from api/domain
  ├─ DiscreteElement: Element<Kind=DiscreteKind> + Ord
  │     type Cardinality: Zero + TryAdd<Self, Output=Self> + Clone
  │     fn try_adjacent(&self, side: Side) -> Option<Self>
  │     fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Cardinality>
  │
  └─ ContinuousElement: Element<Kind=ContinuousKind>
        type Displacement: Zero + TryAdd<Self, Output=Self> + Clone
        fn try_diff(left: &Self, right: &Self) -> Option<Self::Displacement>


measure/cardinality.rs           Cardinality   bounds on T: DiscreteElement
measure/width.rs                 Width         bounds on T: ContinuousElement
ops/span.rs                      Span          bounds on T: TrySub (any T)
```

- `Count` → `Cardinality`; method `.count()` → `.cardinality()`. Clean break.
- `Width` no longer compiles on integer types. Users call `.span()` or cast.
- `Span` is **not a measure** (fails subadditivity on disjoint sets) and lives in `ops/`, not `measure/`.
- Both measures are Tier 3 — `try_*` + panicking sugar — returning `Result<Extent<X>, Err>`.
- Three states per call: `Ok(Finite)` / `Ok(Infinite)` (structural for Half/Unbounded) / `Err(overflow)`.

---

## Architecture

### Trait hierarchy

The api/domain branch (commit `b1a032f`) already introduced `Element { type Kind }` with sealed `DiscreteKind`/`ContinuousKind` markers and a `KindOps<T>` helper for coherence-disjoint dispatch. This redesign **collapses** api/domain's `Discrete: Element + Ord` and the old `Countable: Discrete` into a single `DiscreteElement`, and similarly defines `ContinuousElement` to carry displacement.

```rust
// numeric.rs

pub trait Element: Sized + PartialEq + PartialOrd
where Self::Kind: KindOps<Self>,
{
    type Kind: sealed::Kind;
    fn try_adjacent_or_none(&self, side: Side) -> Option<Self> {
        <Self::Kind as KindOps<Self>>::try_adjacent_or_none(self, side)
    }
}

pub trait DiscreteElement: Element<Kind = DiscreteKind> + Ord {
    type Cardinality: Zero
        + TryAdd<Self::Cardinality, Output = Self::Cardinality>
        + Clone;

    fn try_adjacent(&self, side: Side) -> Option<Self>;

    fn count_inclusive(left: &Self, right: &Self)
        -> Option<Self::Cardinality>;
}

pub trait ContinuousElement: Element<Kind = ContinuousKind> {
    type Displacement: Zero
        + TryAdd<Self::Displacement, Output = Self::Displacement>
        + Clone;

    fn try_diff(left: &Self, right: &Self)
        -> Option<Self::Displacement>;
}
```

**Asymmetries (intentional):**

- `DiscreteElement: Ord` — every discrete type is totally ordered. Required for the panicking-sugar Tier-3 unwraps.
- `ContinuousElement` has no `Ord` — floats stay in domain at `PartialOrd`-only. This is the load-bearing reason api/domain's Kind dispatch exists: half the crate's complexity exists to make floats work.
- Both Output types bound `Zero + TryAdd + Clone` — the bound is for folding per-piece measures on `IntervalSet`. Baked into the trait so `IntervalSet::try_width` / `try_cardinality` have one-line where-clauses.

### Measures

```rust
// measure/cardinality.rs

pub trait Cardinality {
    type Output;
    type Error: core::error::Error;

    fn cardinality(&self) -> Extent<Self::Output> {
        self.try_cardinality().unwrap()
    }

    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error>;
}

impl<T: DiscreteElement> Cardinality for FiniteInterval<T> {
    type Output = T::Cardinality;
    type Error = CardinalityOverflowError;

    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self.view_raw() {
            None => Ok(Extent::Finite(T::Cardinality::zero())),
            Some((l, r)) => T::count_inclusive(l.value(), r.value())
                .map(Extent::Finite)
                .ok_or(CardinalityOverflowError),
        }
    }
}

impl<T: DiscreteElement> Cardinality for HalfInterval<T> {
    type Output = T::Cardinality;
    type Error = CardinalityOverflowError;
    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error> {
        Ok(Extent::Infinite)
    }
}

// EnumInterval dispatches; IntervalSet folds via TryAdd on Extent<Cardinality>.
```

`Width` is structurally identical with `ContinuousElement::try_diff` substituted for `count_inclusive` and `WidthOverflowError` (or a shared error type) substituted for `CardinalityOverflowError`.

### Span

```rust
// ops/span.rs

/// The diameter sup − inf of a set's bounds.
///
/// NOT a measure: fails subadditivity on disjoint sets
/// (span([0,1] ∪ [3,4]) = 4, but Σ span = 2).
///
/// For Lebesgue measure on continuous sets, see [`Width`].
/// For cardinality on discrete sets, see [`Cardinality`].
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
| Finite-bounded, primitive op overflowed (e.g. `[i128::MIN, i128::MAX].try_cardinality()`) | `Err(_)` |

Internally, `count_inclusive` and `try_diff` use `Option<Output>` (`None` = primitive overflow); the measure trait wraps `None` into the typed `Err`. `Extent::Infinite` is constructed structurally by `HalfInterval` and `EnumInterval::Unbounded`, never as a fallback for overflow.

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

`IntervalSet::try_cardinality` and `try_width` fold via `try_binop_map`
over per-piece extents, propagating both `Err` and `Infinite` correctly.

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
type pays for itself in rustdoc.** `try_cardinality() -> Result<Extent<u128>, CardinalityOverflowError>`
reads as "this might fail to compute (Err) OR tell you the set is
unbounded (Infinite)." `Result<Option<u128>, _>` reads as "this might
fail to compute (Err) OR have no value (None)" — true, but the "set
is unbounded" semantic is lost. Domain types carry meaning that
aliases erase. Bidirectional `From` lets callers reach into the
`Option` combinator ecosystem when they want it, without sacrificing
the domain naming.

---

## What survives unchanged

- `Extent<T>` (renamed from `Measurement<T>`) keeps its core `Finite(T)` / `Infinite` shape and the existing tests; surrounding combinator/impl surface gets the cleanup described above
- `Element` trait and `try_adjacent_or_none` (from api/domain)
- The four-tier op contract documented in `ops/mod.rs`
- `OrderedFloat`, `Decimal`, `BigInt`, `BigDecimal` feature modules (each gets a categorical impl)
- All other ops (Union, Intersection, etc.)

## Breaking changes (Stage 2)

| Before | After |
|---|---|
| `Count` trait | `Cardinality` trait |
| `.count()` / `.try_count()` | `.cardinality()` / `.try_cardinality()` |
| `Countable` trait | absorbed into `DiscreteElement` |
| `CountOverflowError` | `CardinalityOverflowError` |
| `default_countable_impl!` | `default_discrete_element_impl!` |
| `integer_domain_impl!` | `default_discrete_element_impl_primitives!` |
| `continuous_domain_impl!` | `default_continuous_element_impl!` |
| `interval.width()` on `Interval<i32>` works | compile error; use `.span()` or cast to `f64` |
| `Measurement<T>` type | `Extent<T>` |
| `.finite()` on the value | unchanged (kept; domain-named panicking accessor) |
| `.expect_finite(msg)` on the value | `Option::from(x).expect(msg)` |
| `.flat_map(f)` on the value | `Option::from(x).and_then(...).into()` |
| `.finite_or(default)` on the value | `Option::from(x).unwrap_or(default)` |
| `.map(f)` on the value | `Option::from(x).map(f).into()` |
| `impl Sub for Measurement<T>` | removed (no measure-theoretic meaning) |
| `impl Add for Measurement<T>` requiring `T: Clone` | `impl Add` drops the `Clone` bound; `impl TryAdd` added for Tier-3 folds |

Clean break, no deprecation aliases. CHANGELOG entry must point integer-`width` users at `span` (semantically equivalent for single intervals; differs only on disjoint sets where users will want `cardinality` anyway).

---

## Migration plan

| Stage | Scope | Notes |
|---|---|---|
| 1 | api/domain merge or revert | Already partial on `api/domain` branch. Decision gated by Stage 2 outcome. |
| 2 | **This redesign** | Cardinality rename, Width gated on ContinuousElement, macro renames, `Measurement`→`Extent` rename + cleanup (drop Sub, add TryAdd, std-naming combinators, Option interop), IntervalSet folds via try_fold |
| 3 | `Span` in `ops/span.rs` | Small, independent. Integer-width users now have a landing pad. |
| 4 | Midpoint + Bisect | Resurrect Midpoint (Result vs Option call), implement Bisect on top |

Stage 2 is the **empirical test for Stage 1**. The categorical pattern (Element → {DiscreteElement, ContinuousElement} with fused storage types) is the api/domain bet writ slightly larger. If Stage 2's bound chains stay tight at user-facing signatures and implementer-side cost is bounded, api/domain ships. If trait bounds metastasize or per-T impl cost explodes, revert api/domain before merging Stage 2.

**Concrete pass/fail metrics:**

- Count of `where T: ...` clauses on `try_width`, `try_cardinality`, `IntervalSet::try_*` signatures
- Lines of code an implementer of a custom type adds (today's `default_countable_impl!` macro use + `Element` impl + `Zero`/`Add`/`Sub` impls is the baseline)
- Whether the existing primitive impls (i32 family, u8 family, f32/f64, OrderedFloat, Decimal, BigInt) port without surgery

Numbers, not vibes.

---

## Documentation responsibilities

- **`measure/mod.rs`** — retains the monotonicity + subadditivity contract for the measures it exports. The "Some common measures are Cardinality, Count, and the Lebesgue measure" sentence updates to drop the stale `Count` reference.
- **`ops/span.rs`** — docstring explicitly states: *"Span is the diameter sup − inf. NOT a measure: fails subadditivity on disjoint sets. For Lebesgue measure on continuous sets, see Width; for cardinality on discrete sets, see Cardinality."*
- **`measure/cardinality.rs`** — explains why the trait gates on `DiscreteElement` (cardinality is only meaningful on countable sets) and points integer-width users at `span` in the migration note.
- **`measure/width.rs`** — explains why the trait gates on `ContinuousElement` (Lebesgue measure on a countable set is 0; the math forbids the call) and points integer users at `span` or numeric conversion.
- **CHANGELOG** — migration table mirroring the breaking-changes section above.
- **Book** — at minimum, the existing measure chapter (if any) updates; ideally a "which measure do I want?" decision tree.

---

## Deferred

### Midpoint

Stays `pub(crate)` through Stage 2. The open question of `Result<Self, Self::Error>` vs `Option<Self>` revisits at the Bisect PR (Stage 4) where actual call sites will reveal whether the typed error pulls its weight. See [`project_midpoint_error_vs_option_open.md`](../../.claude/projects/c--Users-Admin-repos-intervalsets/memory/project_midpoint_error_vs_option_open.md).

### Set-level cast / `Widen`

The `Widen` trait from the original design summary is **dropped** — the per-T `Wider` type can't simultaneously serve as displacement type (Width's natural output: DateTime → Duration) AND arithmetic-headroom type (Cardinality's natural output: i32 → u128). These are different roles, and forcing one trait to bear both conflates affine-point semantics with overflow-prevention.

Users who want `i32` cardinality expressed as `u128`, or `f32` width as `f64`, are best served by a future set-level cast API — `IntervalSet<i32>::cast::<i64>()` style. That's a parallel PR not coupled to this design.

`Extent::map` is **not** an adequate substitute for caller-driven widening: the cast happens *after* subtraction, so any overflow already corrupted the result.

### Continuous types without meaningful Displacement

The 2-tier collapse (`Continuous == ContinuousElement`) forces every continuous type to commit to a `Displacement`. No such type exists in-crate today; if one appears, re-evaluate splitting back to a 3-tier `Continuous` marker + `ContinuousElement: Continuous`.

---

## Appendix: Measure-as-codomain (not in-crate)

This redesign treats `Cardinality` and `Width` as two distinct traits keyed by element-category. An alternative axis the math allows is to keep one measure trait and parameterize it over its codomain — useful for signed measures (`m: Set → ℝ`), complex-valued measures (`m: Set → ℂ`), or vector-valued measures (`m: Set → ℝⁿ`).

**Why not in-crate now:** the crate has no use case for non-Cardinality non-Width measures. Adding the abstraction now is premature; the categorical-trait pattern doesn't preclude adding it later if needed.

**For users who want it:** the recipe is straightforward. Define a trait parameterized over the codomain. (Named `Measurable` here; with the value type renamed to `Extent<T>`, the name `Measure` is also free and would be the more natural choice — `Measurable` is kept here only because the codebase already gravitates toward `-able` suffixes for trait-of-capability.)

```rust
pub trait Measurable<Codomain> {
    type Error: core::error::Error;
    fn measure(&self) -> Result<Codomain, Self::Error>;
}

// Cardinality and Width become impls:
impl<T: DiscreteElement> Measurable<Extent<T::Cardinality>> for FiniteInterval<T> {
    type Error = CardinalityOverflowError;
    fn measure(&self) -> Result<Extent<T::Cardinality>, Self::Error> {
        self.try_cardinality()
    }
}

// And a user-defined signed measure for an interval-weighted-by-position:
pub struct SignedDisplacement<T>(T);

impl<T: ContinuousElement + Neg<Output = T>> Measurable<SignedDisplacement<T::Displacement>>
    for FiniteInterval<T>
{
    type Error = SignedMeasureError;
    fn measure(&self) -> Result<SignedDisplacement<T::Displacement>, Self::Error> {
        // ... compute signed displacement weighted by something user-specific
        todo!()
    }
}
```

The categorical Element split (`DiscreteElement` / `ContinuousElement`) is **orthogonal** to the codomain axis: a user's `Measurable<Codomain>` impl is free to bound on either categorical trait or on `Element` directly.

The two axes compose: element-category determines what set-structure information is available (cardinality vs displacement); codomain determines what the measure values look like. The crate commits to the first axis; users who need the second axis can build it on top.

---

## Why the asymmetries are correct

A reasonable critique: "Width and Cardinality have parallel shape, but Span lives somewhere else and has a different bound shape. Inconsistent?"

The asymmetries reflect mathematical reality:

1. **Width and Cardinality are measures; Span is not.** Span fails subadditivity on disjoint sets. Putting Span in `measure/` would break the module's own contract.

2. **DiscreteElement: Ord; ContinuousElement: no Ord.** Floats live in continuous; they're !Ord because of NaN. Forcing Ord on ContinuousElement would exclude `f32`/`f64`, which is half of the crate's user base. The asymmetry is the crate's central design tension; it doesn't get to disappear here.

3. **Span bounds directly on `TrySub`, not on a categorical trait.** Span's whole point is being available on any T with subtraction — including discrete T. If Span required `ContinuousElement`, integer users lose their replacement for integer-Width. If Span required `DiscreteElement`, float users lose diameter. The direct bound is the only one that serves both audiences.

The unifying principle is **"each operation expresses its math directly."** Width has a category (continuous); Cardinality has a category (discrete); Span doesn't have a category (it's pre-measure-theoretic). Naming the categories at the type level catches the math errors users would otherwise make.
