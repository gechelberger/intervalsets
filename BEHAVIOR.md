# intervalsets — Construction Behavior Spec

A behavior reference for **how each concrete type can be constructed** and
**what you get back** for every interesting input shape (well-ordered,
degenerate, crossed, NaN). The corresponding rationale lives in
`intervalsets-core/src/lib.rs` under "Invariants" and "Construction at
boundaries"; this document is the dense lookup table version.

---

## 1. Construction rules

### 1.1 Invariants every non-empty set must satisfy

1. **Discrete normalization.** Discrete types (e.g. `i32`) are normalized to
   closed form so each set has exactly one valid bit-pattern.
   `open(0, 1) → ∅`, `open(0, 2) → [1, 1]`, `closed_open(0, 5) → [0, 4]`.
2. **`lhs <= rhs`.** All non-empty intervals have a left and right side
   (possibly implicit / unbounded). Equality is allowed only when both bounds
   are closed.
3. **Comparable limit values.** A `FiniteBound`'s value must lie in some chain
   `S ⊆ T` with a strict total order — even if `T` itself is only partially
   ordered. NaN and `±INF` are *not* in `S` for floats and are rejected.

`IntervalSet` adds two more invariants on top:

4. **Strictly ascending, no connections.** Component intervals are sorted
   ascending and no two adjacent intervals are equal, overlap, or *connect*
   (touch at a shared boundary point).
5. **No stored empties.** `Empty` never appears as a component of a non-empty
   `IntervalSet`; the canonical empty set is the zero-component vector.

### 1.2 Constructor tiers

Every constructor falls into one of four tiers. Choose by what the caller
controls:

| Tier | Examples | Crossed bounds (`a > b`) | NaN / incomparable |
|---|---|---|---|
| **Strict primitive** | `FiniteInterval::new`, `try_new`, `IntervalSet::try_new` | `panic InvalidBoundPair` / `Err(InvalidBoundPair)` | `panic` / `Err(TotalOrderError)` |
| **Coercive factory** | `closed`, `open`, `try_closed`, `try_open`, `try_new_or_empty`, `From<Range>`, `From<(T,T)>` | `∅` (silent) | `panic` / `Err(TotalOrderError)` |
| **Pre-normalized** | `new_assume_normed`, `try_new_assume_normed` | `∅` (silent) | `debug_assert!` / `Err(TotalOrderError)` |
| **Bypass** | `new_assume_valid`, `IntervalSet::new_assume_valid` | undefined-but-safe (debug asserts only) | undefined-but-safe (debug asserts only) |

**Where each fits:**
- *Strict primitive* — the "I mean exactly `Bounded(a, b)`" path. Used by
  `Deserialize` because malformed payloads should be hard errors.
- *Coercive factory* — the user-facing default; "give me the interval these
  bounds describe; empty if they don't describe one." Used by `Range`
  conversions, factory traits, `with_left` / `with_right`, splits.
- *Pre-normalized* — internal hot paths (intersection, etc.) where bounds
  are already normalized but may legitimately cross.
- *Bypass* — caller asserts every precondition; release builds skip checks.
  Workspace is `#![forbid(unsafe_code)]` so a violation is a wrong answer,
  never UB.

### 1.3 Panicking vs. `try_*` rule of thumb

Every fallible constructor has both forms.

- **Panicking form** (`new`, `closed`, `open`, …) — ergonomic; for ordered
  types it never panics in practice.
- **Fallible form** (`try_new`, `try_closed`, `try_open`, …) — returns
  `Result<_, Error>`, never panics. Required if `T` may be a NaN-bearing
  float and you can't pre-validate.

The two forms always agree on **the produced set**, differing only on how
they signal failure (panic vs. `Err`). So in the tables below a single row
covers both: a panic in column N means the `try_*` variant returns the
matching `Err` instead.

### 1.4 Float gotcha

`FiniteBound::try_new` is the validation chokepoint for NaN / `±INF`. The
Tier-4 bypass `FiniteBound::new` / `closed` / `open` (note: bypass on
`FiniteBound` only; same names on intervals are coercive factories) accept
NaN silently — the bound exists, it just answers comparisons wrong. Wrap
floats in `OrderedFloat` / `NotNan` if you want `Ord` and don't want to
think about it.

---

## 2. Per-type constructor reference

Symbol legend:
- `OK` → returns the obvious well-formed bounded interval
- `∅` → returns `Empty`
- `panic` → panicking form panics; `try_*` form returns the matching `Err`
- `Bounded` → a non-empty `FiniteInterval` / `HalfInterval`
- *normalized* — discrete inputs are first normalized to closed form, then
  the column header (`a < b`, `a == b`, …) is evaluated against the
  *normalized* values

### 2.1 `FiniteBound<T>` — `intervalsets-core/src/bound.rs`

A single side of a finite interval (the bound type and the limit value).

| Method | Signature | Tier | Valid `T` | NaN / `±INF` |
|---|---|---|---|---|
| `FiniteBound::new(bound_type, t)` | `(BoundType, T) -> Self` | Bypass | `OK` | accepted silently — *wrong-answer* |
| `FiniteBound::closed(t)` | `(T) -> Self` | Bypass | `OK` | accepted silently |
| `FiniteBound::open(t)` | `(T) -> Self` | Bypass | `OK` | accepted silently |
| `FiniteBound::try_new(bound_type, t)` | `(BoundType, T) -> Result<Self, Error>` | Strict | `Ok` | `Err(InvalidBoundLimit)` |
| `FiniteBound::try_closed(t)` | `(T) -> Result<Self, Error>` | Strict | `Ok` | `Err(InvalidBoundLimit)` |
| `FiniteBound::try_open(t)` | `(T) -> Result<Self, Error>` | Strict | `Ok` | `Err(InvalidBoundLimit)` |

### 2.2 `FiniteInterval<T>` — `intervalsets-core/src/sets.rs`

#### Strict primitives

Reject crossed input outright. Bounds are normalized first (so column
headers are evaluated against the *normalized* value).

| Method | `a < b` | `a == b`, both closed | `a == b`, any open | `a > b` | NaN |
|---|---|---|---|---|---|
| `FiniteInterval::new(lhs, rhs)` | `Bounded` | `[a, a]` | `panic InvalidBoundPair` | `panic InvalidBoundPair` | `panic TotalOrderError` |
| `FiniteInterval::try_new(lhs, rhs)` | `Ok(Bounded)` | `Ok([a, a])` | `Err(InvalidBoundPair)` | `Err(InvalidBoundPair)` | `Err(TotalOrderError)` |

#### Coercive factories

Crossed and degenerate-by-construction inputs collapse to `∅`. NaN still
errors / panics — that's a data validity issue, not a "what set did you
mean" issue.

| Method | `a < b` | `a == b`, both closed | `a == b`, any open | `a > b` | NaN |
|---|---|---|---|---|---|
| `FiniteInterval::try_new_or_empty(lhs, rhs)` | `Ok(Bounded)` | `Ok([a, a])` | `Ok(∅)` | `Ok(∅)` | `Err(TotalOrderError)` |
| `FiniteInterval::closed(a, b)` | `[a, b]` | `[a, a]` | n/a | `∅` | `panic` |
| `FiniteInterval::open(a, b)` | `(a, b)` ¹ | `∅` | n/a | `∅` | `panic` |
| `FiniteInterval::closed_open(a, b)` | `[a, b)` ¹ | `∅` | n/a | `∅` | `panic` |
| `FiniteInterval::open_closed(a, b)` | `(a, b]` ¹ | `∅` | n/a | `∅` | `panic` |
| `FiniteInterval::singleton(a)` | — | `[a, a]` always | — | — | `panic` |
| `FiniteInterval::finite(lhs, rhs)` | `Bounded` | `[a, a]` if both closed | `∅` | `∅` | `panic` |
| `FiniteInterval::empty()` | always `∅` | | | | |

¹ Discrete `T` (e.g. `i32`): the result is **renormalized to closed form**.
   `FiniteInterval::open(0i32, 1)` → `∅`; `open(0, 2)` → `[1, 1]`;
   `closed_open(0, 5)` → `[0, 4]`. The "a == b" cell is evaluated on the
   *post-normalization* values for floats and the *pre-normalization*
   values for discrete; in practice both behave identically for these
   columns.

#### Pre-normalized & bypass

Caller has already done the normalization; use these only when
profiling or layering shows it's worth bypassing the validator.

| Method | Tier | Crossed | NaN |
|---|---|---|---|
| `FiniteInterval::new_assume_normed(lhs, rhs)` | Pre-normed | `∅` (silent) | `debug_assert!` (debug only) |
| `FiniteInterval::try_new_assume_normed(lhs, rhs)` | Pre-normed | `Ok(∅)` | `Err(TotalOrderError)` |
| `FiniteInterval::new_assume_valid(lhs, rhs)` | Bypass | `debug_assert!` (debug only); release: bogus `Bounded` | `debug_assert!` (debug only); release: bogus `Bounded` |

### 2.3 `HalfInterval<T>` — `intervalsets-core/src/sets.rs`

A half-bounded interval; one side is finite, the other is implicitly
`±∞`. There are no two-bounds to cross — the only invariant to violate
is comparability of the single finite limit.

| Method | Tier | Valid | NaN |
|---|---|---|---|
| `HalfInterval::new(side, bound)` | Strict | `OK` | `panic` |
| `HalfInterval::try_new(side, bound)` | Strict | `Ok` | `Err(TotalOrderError)` |
| `HalfInterval::left(bound)` | Strict | `OK` | `panic` |
| `HalfInterval::right(bound)` | Strict | `OK` | `panic` |
| `HalfInterval::new_assume_valid(side, bound)` | Bypass | `OK` | `debug_assert!` (debug only) |

### 2.4 `EnumInterval<T>` — `intervalsets-core/src/sets.rs`

Sum of `Finite(FiniteInterval<T>)`, `Half(HalfInterval<T>)`, `Unbounded`.
All factory traits are implemented; behavior is **the same as the
delegated `FiniteInterval` / `HalfInterval` method**, with the result
wrapped in the appropriate variant.

| Method | Behavior |
|---|---|
| `EnumInterval::empty()` | `Finite(∅)` |
| `EnumInterval::unbounded()` | `Unbounded` |
| `EnumInterval::finite(lhs, rhs)` / `try_finite` | as `FiniteInterval::try_new_or_empty`, wrapped |
| `EnumInterval::closed(a, b)` / `open` / `closed_open` / `open_closed` (+ `try_*`) | as `FiniteInterval::*`, wrapped |
| `EnumInterval::singleton(a)` / `try_singleton` | as `FiniteInterval::*`, wrapped |
| `EnumInterval::half_bounded(side, bound)` / `try_half_bounded` | as `HalfInterval::new` / `try_new`, wrapped |
| `EnumInterval::left_bounded(bound)` / `right_bounded` (+ `try_*`) | as `HalfInterval::new`, wrapped |
| `EnumInterval::closed_unbound(a)` / `open_unbound` / `unbound_closed` / `unbound_open` (+ `try_*`) | builds a `FiniteBound`, wraps in `HalfInterval::new` |

### 2.5 `Interval<T>` — `intervalsets/src/sets.rs`

A newtype over `EnumInterval<T>`. Every constructor is pass-through; see
2.4 for behavior.

| Method | Behavior |
|---|---|
| `Interval::empty()` | wraps `EnumInterval::empty()` |
| All factory-trait methods (`closed`, `open`, …, plus `try_*`) | wraps the corresponding `EnumInterval::*` |

### 2.6 `IntervalSet<T>` — `intervalsets/src/sets.rs`

`IntervalSet` invariants are stronger than a single interval's: ascending,
no connections, no stored empties (see 1.1.4 / 1.1.5). The two
constructors that take a collection differ on whether they **repair**
violations or **reject** them.

| Method | Sorted, non-connecting, no empties | Has empties to filter | Out of order / connecting | NaN among bounds |
|---|---|---|---|---|
| `IntervalSet::new(intervals)` | `OK` (zero-cost passthrough) | filtered out | sorted + merged | `panic` (sort sees `partial_cmp == None`) |
| `IntervalSet::try_new(intervals)` | `Ok(Self)` | `Err(InvalidIntervalSet)` ² | `Err(InvalidIntervalSet)` | `Err(InvalidIntervalSet)` ² |
| `IntervalSet::new_assume_valid(intervals)` | `OK` | violation: bogus set (no asserts) | violation: bogus set | violation: bogus set |
| `IntervalSet::empty()` | always empty set | | | |
| `IntervalSet::from_iter(iter)` | calls `IntervalSet::new` (repairs) | | | |

² `try_new` uses `satisfies_invariants`, which catches stored empties as a
   side effect of the strict-ascending check; NaN-bearing components fail
   the same predicate.

`IntervalSet` also implements every factory trait by building a
single-interval set from the corresponding `Interval::*` method:
`IntervalSet::closed(a, b)` ≡ `IntervalSet::from(Interval::closed(a, b))`
(empty in, empty out).

---

## 3. `From<...>` conversions

All `From<…>` impls below are **coercive** — crossed / reversed inputs
produce `∅`, NaN panics. They funnel through
`FiniteInterval::try_new_or_empty(...).unwrap()`.

| Source | Target | Bound interpretation | `a < b` | `a == b` | `a > b` |
|---|---|---|---|---|---|
| `()` | `FiniteInterval<T>` / `EnumInterval<T>` / `Interval<T>` / `IntervalSet<T>` | — | always `∅` | | |
| `(T, T)` / `&(T, T)` | `FiniteInterval` / `EnumInterval` / `Interval` / `IntervalSet` | `(open, open)` | `(a, b)` ¹ | `∅` | `∅` |
| `[T; 2]` / `&[T; 2]` | `FiniteInterval` / `EnumInterval` / `Interval` / `IntervalSet` | `[closed, closed]` | `[a, b]` | `[a, a]` | `∅` |
| `Range<T>` (`a..b`) | `FiniteInterval` / `Interval` / `IntervalSet` | `[closed, open)` | `[a, b)` ¹ | `∅` | `∅` |
| `RangeInclusive<T>` (`a..=b`) | as above | `[closed, closed]` | `[a, b]` | `[a, b]` | `∅` |
| `RangeFrom<T>` (`a..`) | `HalfInterval` / `Interval` / `IntervalSet` | `[a, ∞)` | always `OK` | | |
| `RangeTo<T>` (`..b`) | `HalfInterval` / `Interval` / `IntervalSet` | `(-∞, b)` | always `OK` | | |
| `RangeToInclusive<T>` (`..=b`) | as above | `(-∞, b]` | always `OK` | | |
| `RangeFull` (`..`) | `EnumInterval` / `Interval` / `IntervalSet` | `(-∞, ∞)` | always `Unbounded` | | |
| `FiniteInterval<T>` | `EnumInterval` / `Interval` / `IntervalSet` | wrap | passthrough | | |
| `HalfInterval<T>` | `EnumInterval` / `Interval` / `IntervalSet` | wrap | passthrough | | |
| `EnumInterval<T>` | `Interval` / `IntervalSet` | wrap | passthrough | | |
| `Interval<T>` | `IntervalSet` | empty in → empty set; non-empty → singleton set | passthrough | | |

¹ Discrete `T` is renormalized; see footnote in §2.2.

---

## 4. Quick answers to common questions

> **`FiniteInterval::new(FiniteBound::closed(10), FiniteBound::closed(0))` — panic or empty?**
> **Panic.** `new` is the strict primitive. It calls `try_new` and unwraps, so
> you get `panic InvalidBoundPair`. Use `FiniteInterval::closed(10, 0)` (the
> coercive factory) to get `∅` instead, or `try_new` for a `Result`.

> **`FiniteInterval::open(0i32, 1)` — what?**
> **`∅`.** Open-on-`i32` normalizes to closed: `(0, 1) → [1, 0] → ∅`.

> **`Interval::closed(f64::NAN, 1.0)` — panic or empty?**
> **Panic** (`TotalOrderError`). NaN never coerces to empty; it always errors.
> Use `try_closed` for the `Result` form, or wrap in `OrderedFloat`/`NotNan`.

> **`IntervalSet::try_new([Interval::closed(0, 10), Interval::closed(10, 20)])` — Ok or Err?**
> **Err.** Those two intervals *connect* at 10 (and would be valid as the
> single `[0, 20]`). `try_new` does not repair. `IntervalSet::new` does and
> would return `[0, 20]` as a single component.
