# Design: storage-type casts for set types

## Context

The crate has no first-class mechanism for converting the storage element type of a set: `Interval<i32>` to `Interval<i64>`, `IntervalSet<f64>` to `IntervalSet<f32>`, `FiniteInterval<u32>` to `FiniteInterval<i64>`, etc. Today users have to extract bounds manually and reconstruct via factory methods.

The only existing primitive that crosses types is `FiniteBound::map<U>(self, FnOnce(T) -> U) -> FiniteBound<U>` at `intervalsets-core/src/bound.rs:203`. It transforms the leaf value but does **not** revalidate. There are no `From` / `TryFrom` impls that cross element types — `intervalsets-core/src/from/try_from.rs` covers only same-`T` variant narrowing.

> **Aside — pre-existing bug.** `FiniteBound::map<U>(self, FnOnce(T) -> U) -> FiniteBound<U>` at `intervalsets-core/src/bound.rs:203` bypasses the `Element::validate` chokepoint: a caller can construct a `FiniteBound<f64>` containing NaN via `bound.map(|_| f64::NAN)`, or a user-defined `T` containing a value its own `validate` would reject. (A companion `FiniteBound::binary_map` had the same problem and was deleted in this branch — also had zero callers.)
>
> `map` has **zero callers in the entire workspace** outside its own doctest. It's dead public API.
>
> Replacement options, in order from cleanest to most invasive:
>
> 1. **Delete `map`.** Zero callers; nothing to migrate. The cast trait surface in this design supersedes any cross-type use case the method could have had.
> 2. **`try_map(FnOnce(T) -> Result<U, E>) -> Result<FiniteBound<U>, _>`.** Validation runs inside (routed through `FiniteBound::try_new`); no caller panic site; preserves `BoundType`. Adding this in the cast PR is the right time if a transform primitive turns out to be needed.
> 3. **`try_flat_map(FnOnce(BoundType, T) -> Result<FiniteBound<U>, E>) -> Result<FiniteBound<U>, E>`.** Like (2) but lets the closure change `BoundType` too. Useful only for atomic value+bound-type swaps; the value-only case (2) covers everything actually needed.
>
> **Strictly avoid: infallible `flat_map(FnOnce(T) -> FiniteBound<U>) -> FiniteBound<U>`.** The chokepoint runs because the caller had to construct a `FiniteBound<U>`, but the *only* paths to satisfy the closure signature are `FiniteBound::new` (panics on invalid input) or `FiniteBound::try_new(...).unwrap()` (panics on `Err`). The validation is preserved at the cost of forcing a panic site into every call site — strictly worse than `try_map`, which gives callers a `?`-friendly path.
>
> Recommended: (1) delete now, optionally add (2) as part of the cast PR if a transform helper turns out to be useful.

A correct cast must round-trip through three validation chokepoints:

| Chokepoint | Path | Catches |
|---|---|---|
| `FiniteBound::try_new` | `intervalsets-core/src/bound.rs:478` | `Element::validate` rejects (NaN, ±INF, user-defined invalid limits) |
| `FiniteInterval::try_new` | `intervalsets-core/src/sets.rs:124` | discrete renormalization, ordering / crossed-bound |
| `IntervalSet::try_new` | `intervalsets/src/sets.rs:280` | set invariants (sort, disjointness, no-touching) |

Element-level casts can fail in three independent ways:

1. **Element conversion fails** — `i64 -> i32` overflow, `f64 -> i32` out-of-range.
2. **Post-cast value rejected by `Element::validate`** — `f64::MAX -> f32::INFINITY` rejected as non-finite.
3. **Bounds collide after cast** — two distinct `f64` bounds round to the same `f32`, producing `lhs > rhs` or open-open at equality.
4. **(IntervalSet only) two distinct intervals collapse onto each other** — set invariants broken even when each individual interval is valid.

Any design has to address all four.

The repo's relevant idioms:

- **Four-tier contract** documented at `intervalsets-core/src/ops/mod.rs:23-116`. A cast naturally lands as **Tier 3a** (total, panic-free, returns `Result`).
- **`Try*` pattern** (`TryAdd`/`TrySub`/`TryMul`/`TryDiv`) at `intervalsets-core/src/ops/math/mod.rs:66-109`, each with `type Output`, `type Error`, `fn try_op(self, rhs) -> Result`. Wired through sets via the `extract bounds -> element op -> reassemble via try_new` pattern (`intervalsets-core/src/ops/math/add.rs:40`).
- **Error type composition** — `MathError` (`intervalsets-core/src/error.rs:56`) is a focused enum that converts via `From` into the crate-level `Error`. Same pattern is available for a `CastError`.
- **`num-traits` is already a default dep** (`intervalsets-core/Cargo.toml:41`). `num_traits::NumCast` provides fallible casts between *all* primitive number types — including `f64 -> f32` — which `std::convert::TryInto` does not.

## Two framings

There are two fundamentally different ways to organize this surface, both worth taking seriously.

### Framing 1 — One operation with policy axes

Treat "cast" as a single operation parameterized by policy choices (strict vs coercive on bound collision; strict vs repair on set-invariant breakage; one element-cast primitive). Four orthogonal axes; bundle the choices.

### Framing 2 — Three operations, each with one clear contract

Split by *semantic intent* into three traits, each landing in a distinct tier:

| Trait | Tier | Contract |
|---|---|---|
| `Cast<U>` | Tier 1 — truly infallible | Implemented only when `T -> U` cannot fail for any input. Returns `Self::Output` directly. |
| `LossyCast<U>` | Tier 1 — total, lossy by design | Always succeeds. Projects each `T` to its nearest representable `U` (clamping out-of-range to extrema, rounding in-range to the closest representable value). |
| `TryCast<U>` | Tier 3a — total, panic-free, `Result` | Strict: any conversion that would fail validation (overflow, post-cast non-finite, crossed bounds) is `Err`. |

The three traits map directly onto the existing four-tier contract documented at `intervalsets-core/src/ops/mod.rs:23-116`. Each has exactly one behavior; no policy parameters. This is the framing the user proposed. Section "Framing 2 detail" below works through it.

---

The next four sections enumerate the axes that apply under Framing 1. The "Framing 2 detail" section after that walks through the three-trait design. The "Viable bundles" section at the end lists coherent picks under each framing.

## Design axes (Framing 1)

### Axis A — Primary surface

How does the user spell a cast?

### Axis B — Element-cast primitive

What underlying conversion does the set-level cast delegate to?

### Axis C — Bound-collision policy

What happens when a cast produces `lhs > rhs` or other crossed pairs?

### Axis D — IntervalSet invariant policy

What happens when two intervals collapse onto each other after element narrowing?

---

## Axis A — Primary surface

### A1. Trait `TryCast<U>` (method discovered via `.try_cast::<U>()`)

```rust
pub trait TryCast<U> {
    type Output;
    type Error;
    fn try_cast(self) -> Result<Self::Output, Self::Error>;
}
```

Implemented for each set type. Caller writes `interval.try_cast::<i64>()?`.

- **+** Discoverable (visible in rustdoc method lists, prelude-friendly).
- **+** Composes with `?` and generic bounds: `fn widen<T, U>(x: Interval<T>) -> Result<Interval<U>, _> where Interval<T>: TryCast<U, Output = Interval<U>>`.
- **+** Matches the established `Try*` trait family (parallels `TryAdd<U>` for output type).
- **−** Forces a fixed delegation strategy (whatever element primitive Axis B picks).
- **−** Generic on `U` at the impl level — slightly more boilerplate than inherent methods.

### A2. Inherent method `try_cast<U>` on each type

```rust
impl<T: Element> FiniteInterval<T> {
    pub fn try_cast<U: Element>(self) -> Result<FiniteInterval<U>, CastError<...>> where T: ... { ... }
}
```

- **+** Simplest possible surface; no trait import needed.
- **−** Can't be the basis for generic code over "things that cast". The `Try*` family treats casts as second-class compared to arithmetic.
- **−** N parallel impl blocks; trait would centralize the surface.

### A3. Closure-based `try_map<U, F, E>(f)`

```rust
pub trait TryMap {
    type Element;
    type Output<U>;
    fn try_map<U, F, E>(self, f: F) -> Result<Self::Output<U>, CastError<E>>
    where F: FnMut(Self::Element) -> Result<U, E>, U: Element;
}
```

Caller writes `interval.try_map(|x: f64| Ok::<_, Infallible>(x as f32))?`.

- **+** Most flexible: caller supplies any `T -> Result<U, E>`. Handles lossy casts with custom rounding (`f.round() as i32`), unit conversions (`secs as millis`), domain-specific scaling, and trivially handles `as` casts when `TryInto` is missing (no `TryInto<f32> for f64`).
- **+** Single underlying primitive that everything else can be built on; `try_cast` becomes sugar for `try_map(|t| t.try_into())`.
- **−** Less discoverable; type inference for `U` often needs a turbofish on the closure return type.
- **−** Reads heavier at the call site for the common widening case.
- **−** GAT-style `Output<U>` is unconventional in this crate (current `Try*` traits put `Output` and `Error` as plain associated types). An alternative shape is per-type inherent `try_map`, sidestepping the GAT.

### A4. `From` / `TryFrom` blanket impls

`impl<T, U> TryFrom<Interval<T>> for Interval<U> where U: TryFrom<T>`

- **+** Standard-library spelling.
- **−** **Doesn't compile** — conflicts with the reflexive `TryFrom<T> for T` blanket. Would require negative trait bounds (unstable) or a marker. Effectively off the table in stable Rust.

### A5. Hybrid `TryCast<U>` + closure escape hatch

Both A1 and A3, with A1 as the discoverable default and A3 as the escape hatch for lossy / custom casts. The trait delegates to the closure internally:

```rust
impl<T, U> TryCast<U> for FiniteInterval<T> where T: TryCastElement<U> {
    fn try_cast(self) -> ... { self.try_map(|t| t.try_cast_element()) }
}
```

- **+** Best of both worlds. Trait covers the 90% case; closure covers the 10%.
- **−** Roughly 2× the API surface area (two traits + impls).

---

## Axis B — Element-cast primitive

This is what the set-level `try_cast` delegates to for the actual `T -> U` step.

### B1. `std::convert::TryInto`

- **+** Standard, no extra trait machinery.
- **−** **No impl between f32 and f64** — `f64 -> f32` is a primitive `as` cast in Rust, not modeled in `TryInto`. Float narrowing — arguably the headline use case — is impossible without a custom primitive.
- **−** No impl between integer and float either; `f64 -> i32` requires a custom path.

### B2. `num_traits::NumCast`

```rust
pub trait NumCast: Sized + ToPrimitive {
    fn from<T: ToPrimitive>(n: T) -> Option<Self>;
}
```

Already a default dep. Covers **all** primitive number pairs (int <-> int, int <-> float, float <-> float). `None` for out-of-range or non-finite.

- **+** Single primitive covers every primitive cast users want.
- **+** Already in the dep tree; no new transitive cost.
- **+** Aligns with the rest of `num-traits` already used in `numeric.rs`.
- **−** Returns `Option`, so the crate has to invent an error variant or wrap as `CastError::Element(None)`.
- **−** Only covers types implementing `NumCast`. User-defined `T` (e.g. `BigInt`, `FixedPoint`) wouldn't get `try_cast` automatically; they'd need a manual impl. The `feat/bigint.rs` and `feat/fixed.rs` modules show the crate already accommodates non-primitive `T`.

### B3. Custom trait `TryCastElement` in the crate

```rust
pub trait TryCastElement<U>: Sized {
    type Error;
    fn try_cast_element(self) -> Result<U, Self::Error>;
}
```

Default impls keyed on a blanket `impl<T: TryInto<U>, U> TryCastElement<U> for T` + targeted impls for the float pairs and float/int pairs that `TryInto` misses.

- **+** Full control over which conversions are blessed and what errors look like.
- **+** Lets user `T`s opt in by implementing the trait.
- **−** Macro / impl boilerplate proportional to N² primitive pairs (or use `NumCast` internally and just wrap it — see B4).

### B4. Composite: `TryCastElement` blanket-impl'd over `NumCast`

```rust
// blanket: anything NumCast can cast
impl<T: ToPrimitive, U: NumCast> TryCastElement<U> for T { ... }
// user types can manually implement the trait for their own cases
```

- **+** Discoverable + extensible. Built-in primitives just work; user types opt in.
- **−** Two traits to document; need a clear "implement this when adding a new T" story.

### Note on `Element::validate`

Whichever primitive is chosen, the cast must still route the *result* through `FiniteBound::try_new` so `Element::validate` runs. `NumCast` catching `f64::MAX -> Some(f32::INFINITY)` would otherwise slip past — `NumCast` doesn't check finiteness of the result, only that it didn't overflow during the cast operation.

---

## Axis C — Bound-collision policy after cast

After both bounds of a `FiniteInterval` are cast, they may end up crossed (`lhs > rhs`) or in an open-open-at-equality configuration. `FiniteInterval::try_new` returns `Err(Error::InvalidBoundPair)` in this case. The factory module already offers a coercive sibling (`try_satisfy_bounds` at `intervalsets-core/src/factory.rs:540`) that turns crossed bounds into `Empty`.

### C1. Strict only

`try_cast` returns `Err(CastError::Set(InvalidBoundPair))` on collision.

- **+** Surfaces narrowing-induced data loss — usually a bug worth catching.
- **+** Matches `try_new` semantics; no surprise.
- **−** Users who legitimately want "narrow to f32 and let collapsed intervals vanish" have to catch the error and convert.

### C2. Coercive only

`try_cast` routes through `try_satisfy_bounds`; crossed -> `Empty`.

- **+** Mirrors `satisfy_bounds` / repairing constructors elsewhere in the crate.
- **−** Silent data loss. A non-empty input becomes an empty output with no signal.

### C3. Both — strict default + named coercive variant

`try_cast` is strict; `try_cast_satisfy` (name TBD) is coercive.

- **+** Both behaviors available; the named variant advertises the coercion.
- **−** Two methods per type. Trait approach (A1) needs a sibling trait or a method on the same trait.

### C4. Caller-supplied policy

```rust
fn try_cast_with<U>(self, policy: CollisionPolicy) -> ...
```

- **+** One method, parameterized.
- **−** Heavier signature; less idiomatic than two named methods in a Rust API.

---

## Axis D — IntervalSet invariant policy after cast

Even if every individual interval casts cleanly, the result `Vec<Interval<U>>` may break `IntervalSet`'s invariants (sort, disjointness, no-touching) when narrowing collapses two intervals onto the same value or makes them connect.

### D1. Strict — `IntervalSet::try_new`

Returns `Err` if invariants are violated.

- **+** Surfaces invariant breakage.
- **−** Common narrowing scenarios (e.g. `f64 -> f32` of a set with tight nearby intervals) become hard errors.

### D2. Repair — `IntervalSet::new`

Silently sorts, merges, drops empties.

- **+** Matches the repairing constructor's purpose.
- **−** Silent loss of distinction between distinct intervals.

### D3. Both — strict default + named repair variant

Same shape as C3.

### Note on layered choice

Axes C and D are nominally independent but pair naturally. A user who wants strict bound semantics (C1) probably also wants strict set semantics (D1). Bundling them as a single "strict vs repaired" flavor keeps the surface tight.

---

## Cross-cutting concerns

### Error type

```rust
// intervalsets-core/src/error.rs, next to MathError
#[non_exhaustive]
pub enum CastError<E = core::convert::Infallible> {
    Element(E),
    Set(Error),
}
impl<E> From<Error> for CastError<E> { ... }
```

- Default type param `E = Infallible` makes `CastError<Infallible>` the natural type for ordering-preserving widenings (`i32 -> i64`) where the element layer can't fail. The `Element` variant is then uninhabited at that layer, and only `Set(_)` is reachable (and even that is unreachable for already-validated inputs widening to a larger type).
- Composability with the crate `Error` enum mirrors how `MathError` lives alongside `Error`.
- An alternative is folding into `Error` as a new variant `Cast(...)`. That avoids the second enum but couples cast failures to general-error consumers who don't care.

### Infallible widening — is a separate `cast` trait worth it?

Probably not. For `i32 -> i64`, `f32 -> f64`, etc., the `TryCast` path returns `Result<_, CastError<Infallible>>`, which compiles to a clean unwrap. A dedicated `Cast<U>` trait keyed on `From`/widening would:

- Require a marker trait (`WideningInto<U>`) since Rust has no built-in widening notion;
- Have to be reimplemented per primitive pair;
- Save no runtime cost over `TryCast`.

The cost-benefit doesn't pay off. Document that widenings hit a zero-cost path through `TryCast`.

### Validation chokepoints (must all run)

A correct cast on `FiniteInterval` is:

```
extract bounds via FiniteInterval::into_raw (sets.rs:188)
  for each bound:
    extract via FiniteBound::into_raw (bound.rs:154)   -- gets (BoundType, T)
    apply element cast: T -> Result<U, _>
    rebuild via FiniteBound::try_new (bound.rs:478)    -- runs Element::validate
  rebuild via FiniteInterval::try_new (sets.rs:124)    -- normalize + ordering check
```

`HalfInterval` mirrors this (uses `into_raw` at sets.rs:327 + `try_new` at sets.rs:280). `EnumInterval` dispatches per variant; `Unbounded` is `Ok(Unbounded)`. `Interval` delegates to inner `EnumInterval`. `IntervalSet` maps each `Interval`, then routes through `IntervalSet::try_new` or `::new` (intervalsets/src/sets.rs:280, 328) per Axis D.

### Module placement & prelude

- `intervalsets-core/src/ops/cast.rs` — trait + element/interval impls. Parallel to `ops/math/`.
- `intervalsets/src/ops/cast.rs` — `Interval`/`IntervalSet` wrappers.
- `intervalsets-core/src/ops/mod.rs` member list (tier 3a, lines 89-92) — add `TryCast`.
- Both `prelude.rs` files — re-export `TryCast` (+ `CastError`, + `TryMap` if A3/A5).

### Coverage of user-defined `T`

The crate supports custom `T` (e.g. `feat/bigint.rs`, `feat/fixed.rs`). Choice of element primitive (Axis B) determines whether their `T` is castable out of the box:

- B1 (`TryInto`): yes, if `T: TryInto<U>`.
- B2 (`NumCast`): yes, if `T: NumCast` (true for `BigInt` via num-bigint; `FixedPoint` would need a manual impl).
- B3/B4 (custom trait): explicit opt-in.

A3 / A5 (closure form) sidesteps the issue entirely — caller supplies the per-type function.

---

## Framing 2 detail — `Cast` / `LossyCast` / `TryCast`

### `Cast<U>` — Tier 1, infallible

```rust
pub trait Cast<U> {
    type Output;
    fn cast(self) -> Self::Output;
}
```

Implemented for each set type. The signature has no `Result` — calling it cannot fail.

**Element primitive: `T: Into<U>`.** Rust's `From`/`Into` is *contractually* infallible. The standard library carefully restricts `From` impls between primitive numeric types to exactly the *exact, monotonic, validate-preserving* pairs:

- `From<i32> for i64` exists. `From<i64> for i32` does not.
- `From<f32> for f64` exists. `From<f64> for f32` does not.
- `From<i8> for f32` exists (every `i8` is exactly representable). `From<i32> for f32` does *not* (precision loss).
- `From<u32> for f64` exists. `From<u32> for f32` does not.

So `T: Into<U>` is a remarkably clean filter for "this cast can't fail". For user-defined `T` and `U`, implementing `From<T> for U` *is* the standard contract for "this is infallible and lossless".

**Implementation:** `FiniteBound::into_raw -> map -> FiniteBound::new_assume_valid` (i.e. skip `try_new`). Skipping is sound because for `T: Into<U>` standard pairs, `U::validate(t.into())` is always `Some` — `f32 -> f64` preserves finiteness; integer widenings preserve comparability. For user-defined `T: Into<U>` impls we are *trusting* the user's `From` contract; if they implement an `Into` that produces invalid `U` values, that is a `From`-contract violation on their end.

Alternative: still route through `try_new` and `unwrap_assume_valid`-style; eat the perf cost for the safety floor. Probably not worth it given `Into`'s contract.

**Bound ordering:** `T: Into<U>` doesn't guarantee monotonicity in general, but every standard-library `Into` between primitives *is* monotonic. For user types, the trait bound is again a contract. If `Cast` users want belt-and-braces, they can call `try_cast` instead and pay for one ordering check.

### `LossyCast<U>` — Tier 1, total but lossy

```rust
pub trait LossyCast<U> {
    type Output;
    fn lossy_cast(self) -> Self::Output;
}
```

Total: cannot fail. Lossy: values outside `U`'s representable range clamp to `U`'s nearest extremum.

**What the trait actually does — semantic clarification.** This trait performs *two* projections in one operation, both lossy:

1. **Out-of-range elements clamp to U's extrema.** A `f64` magnitude beyond `f32::MAX` becomes `f32::MAX`. An `i64` outside `i32::MIN..=i32::MAX` clamps to the nearest of those bounds.
2. **In-range elements round to the nearest representable U.** A `f64` like `1.0 + f64::EPSILON` projects to its nearest `f32` neighbor; an `f32::PI` projects to a different (rounded) `i32` than its mathematical value.

Both are "projection of T onto U's representable lattice". The trait's name should communicate the *whole* operation, not just the boundary clamping — otherwise users assume in-range values pass through exactly and are surprised by rounding.

**Name candidates** — must (per family-branding constraint) include "Cast", must communicate broader-than-clamping projection:

| Name | Reads as | For us | Against us |
|---|---|---|---|
| `LossyCast` | "may lose information" — umbrella | Friendliest; familiar concept (lossy compression / serialization); covers both clamping and rounding under one banner; reads naturally at call sites (`big.lossy_cast()`). Matches the user's framing of "elements beyond U's representation". | Doesn't specify *how* lossy. If we later want a sibling that wraps instead of clamps, both would be "lossy" — disambiguation lives in docs, not the name. |
| `ProjectCast` | "project T onto U's lattice" | Mathematically precise; captures both clamping and rounding cleanly under "projection"; family-branded. | "Project" is overloaded in software (record projection in DBs, type projection in TS, etc.). Less ecosystem precedent. |
| `CollapseCast` | "collapse T's distinctions onto U" | Matches the user's original `CollapseProjection` framing while keeping the `Cast` family suffix; communicates the lossy-merging-of-distinctions intuition (two distinct `f64`s becoming the same `f32`). | "Collapse" can suggest set-collapse rather than element projection. |
| `ApproxCast` | "approximate result, not exact" | Communicates rounding directly; has ecosystem precedent (`conv::ApproxFrom`); family-branded. | "Approx" sometimes read as "approximately equal" / tolerance-comparison rather than projection. |
| `SaturatingCast` | "out-of-range clamps to extrema" | Strongest ecosystem precedent (`az::SaturatingCast`, `conv::SaturatingFrom`, `i32::saturating_*`). | Undersells the in-range rounding behavior. Users may expect in-range values to pass through unchanged. |
| `ClampCast` | "clamp to U's range" | Compact; widely understood. | Same problem as `SaturatingCast`: doesn't capture in-range rounding. |
| `SurjectionCast` | "function covers all of U" | Mathematically rigorous *when accurate*. | **Technically not always correct.** Surjection requires every `U` value have a `T` preimage. True for `i64 → i32`, `f64 → f32`. False for `u128 → f32` (no negatives reachable), `u32 → i32` (no negatives reachable), `BigInt → f32` with saturation (same). The trait name would advertise a property it doesn't always have. Also: surjection isn't the user-relevant property — *non-injectivity* (many-to-one) is. A bijection is also surjective but not lossy. |
| `QuotientCast` | "quotient map — identifies/collapses elements" | The mathematically *correct* term for "many-to-one mapping that may cover codomain": a quotient map in category theory. Captures the lossiness as the defining property (collapsing distinct T's onto one U). Doesn't overpromise codomain coverage. | "Quotient" is overloaded (division, vector-space quotients, equivalence classes). Likely opaque to users without abstract-algebra background. |
| `RetractionCast` | "function with a section recovering identity" | Cleaner than surjection — implies every U has a canonical T preimage via the section. | Same universality problem as `SurjectionCast`: `u128 → f32` isn't a retraction (no section recovers negative f32). |
| `QuantizationCast` | "round samples to nearest representable value" (signal-processing) | Exactly describes `f64 → f32` and `f → i`: the standard term in DSP for mapping continuous amplitude to a discrete lattice via rounding. Captures both clamping (overflow) and in-range rounding under one banner. | Awkward for `i64 → i32` (no rounding involved, just clamping — i64 is already discrete). Domain-restricted: feels float-specific to most readers. |
| `ForgetfulCast` | "drops structure" (category-theoretic) | Accurate intuition for "lossy because we forget precision/range". Doesn't overpromise codomain coverage. | Obscure outside category theory; most users have to look it up. |

**On mathematical rigor.** There is **no single canonical math term** that precisely describes "a total function that may be many-to-one and may not cover the codomain". In math that's just "a function" — the property we care about (lossiness / non-injectivity) is the *negation* of an interesting property (injectivity), and absence-of-injectivity doesn't have its own name. Every rigorous candidate either:

- isn't universally accurate (`SurjectionCast`, `RetractionCast` — fail for `u128 → f32`);
- is jargon (`QuotientCast`, `ForgetfulCast` — accurate but opaque);
- is domain-restricted (`QuantizationCast` — natural for floats, awkward for int saturation);
- captures the wrong property (`ProjectionCast` — idempotence is incidental, not the contract).

`LossyCast`'s "intentional vagueness" is a feature: it makes the minimum claim (information *may* be lost) that is universally true across every T,U pair the trait supports, and trades mathematical precision for accessibility. The rustdoc carries the precise contract.

**Working recommendation:** **`LossyCast`** — friendliest, broadest, communicates "may lose info" which subsumes both clamping and rounding, and makes no claim about codomain coverage so it's accurate for every T,U pair. Strongest runner-up: **`ProjectCast`** if we want the mathematical framing without the overload baggage of "quotient" or the technical inaccuracy of "surjection".

Either way, the rustdoc on the trait must spell out both behaviors explicitly: "casts every element to its nearest representation in `U`, clamping out-of-range elements to `U`'s extrema". The name is a banner; the doc is the contract.

The rest of this document uses `LossyCast` as the placeholder name; substitute as desired without affecting any other design choices.

**Element primitive options:**

- `num_traits::cast::AsPrimitive<U>` — `T as U` semantics. For `f64 -> f32` produces `f32::INFINITY` for over-range values, *not* saturated to `f32::MAX`. Wrong semantics for this trait.
- `num_traits` doesn't have a `Saturating` trait that covers float pairs; `Saturating` covers integer arithmetic.
- **Most likely path:** define a small in-crate `LossyCastElement<U>` trait with explicit primitive impls. Internally calls saturating `as` for integer→integer / float→integer, and for `f64 -> f32` clamps to `[f32::MIN, f32::MAX]` before the `as` cast (which performs the in-range rounding). Boilerplate is ~30 impls under a macro.

**`U: Bounded` requirement.** `LossyCast<U>` only makes sense when `U` has well-defined min/max. Trait bound `U: Bounded` (from `num_traits`) gates the impls. Unbounded user types (`BigInt`, arbitrary-precision) won't get `LossyCast<U>` automatically — they can opt in via a manual impl if it makes sense, or be deliberately excluded.

**Bound-type policy under saturation.** When a bound's *value* is clamped to `U`'s extremum, what happens to its open/closed flag? Three options:

1. **Snap to closed.** Saturation means "you wanted to include everything up to here, but it didn't fit; the boundary is now at the extremum and we include it." Natural interpretation: open becomes closed at the saturated extremum. Preserves "this interval represents the projection of the original onto U's range."
2. **Preserve bound type.** Open stays open; closed stays closed. Risk: open-open at the same extremum after both bounds saturate — degenerate (empty under `try_new` semantics).
3. **Map degenerate to empty.** Bounds preserve type but if the result is degenerate, collapse to `Empty`. Means `LossyCast` can produce `Empty` from a non-empty input — arguably still total, since `Empty` is a valid output.

Recommendation: **snap to closed at the saturation boundary**. The user's intent in calling `lossy_cast` is "project into U's representable space"; once we've discarded "outside U's range", the open/closed distinction at the discarded boundary is meaningless.

**IntervalSet under lossy cast.** Multiple intervals can project to overlapping ranges (anything above `i32::MAX` collapses to the singleton at `i32::MAX`; two close `f64` intervals can round to the same `f32`). After per-interval projection, route the `Vec<Interval<U>>` through `IntervalSet::new` (the repairing constructor) — merging is the natural completion of "we already discarded distinctions in T that don't survive in U".

### `TryCast<U>` — Tier 3a, strict

```rust
pub trait TryCast<U> {
    type Output;
    type Error;
    fn try_cast(self) -> Result<Self::Output, Self::Error>;
}
```

Strict contract: returns `Err` if any of the four failure modes from the context section occurs. Bound-collision policy is **strict** (no coercion); IntervalSet policy is **strict** (no repair).

**Element primitive options** (same as Axis B above):

- B1 `TryInto` — std-aligned but no float-narrowing impl.
- B2 `NumCast` — covers all primitive pairs including `f64 -> f32`. Already in dep tree.
- B3/B4 — custom trait, possibly with `NumCast` blanket.

For three-trait framing, **B2 (`NumCast`)** pairs naturally — it covers exactly the cases where `Cast` doesn't apply, and the trait is already in scope.

### Blanket: `Cast => TryCast`

```rust
impl<T, U> TryCast<U> for T
where T: Cast<U>
{
    type Output = <T as Cast<U>>::Output;
    type Error = Infallible;
    fn try_cast(self) -> Result<Self::Output, Infallible> {
        Ok(self.cast())
    }
}
```

Lets generic code over `TryCast<U>` work uniformly with widenings. Returns `Result<_, Infallible>` for the widening case, which the user can `.unwrap()` or pattern-match exhaustively for free.

**Coherence concern.** If `TryCast` is also implemented directly for `T: Into<U>` *and* via this blanket, there's a conflicting-impl error. Solutions:

- Implement `TryCast` only via the blanket from `Cast`; don't write a direct `TryCast` impl for `T: Into<U>`.
- Or: pick disjoint element bounds. `Cast` keyed on `Into`, `TryCast` keyed on `NumCast` minus `Into`. Awkward.
- Or: skip the blanket; force user to choose. Loses ergonomics in generic code.

Recommended: **blanket from `Cast`**. Direct `TryCast` impls cover the non-`Cast` cases (any `NumCast` pair where `Into` doesn't apply: narrowings, lossy precision, etc.).

### Three traits, three call sites

```rust
let widened: Interval<i64>  = i32_interval.cast();                  // Tier 1, infallible
let clamped: Interval<i32>  = i64_interval.lossy_cast();            // Tier 1, lossy
let checked: Result<Interval<i32>, _> = i64_interval.try_cast();    // Tier 3a, fallible
```

Each call site advertises its own semantics. No policy enum at the type level; no policy method names like `try_cast_satisfy` / `try_cast_repaired`.

### Tradeoffs of Framing 2

- **+** Each trait has exactly one behavior. Easier to teach, easier to audit, harder to misuse.
- **+** Maps directly onto the existing tier contract — `cast` is Tier 1, `lossy_cast` is Tier 1, `try_cast` is Tier 3a. Drop them into the tier-3a member list at `ops/mod.rs:89-92` and the tier-1/2 lists; the surface explains itself.
- **+** The blanket `Cast => TryCast` covers generic code without forcing a choice.
- **+** No "coercive vs strict" or "strict vs repair" decisions surfaced to users — different intents have different names.
- **−** Three traits to document and re-export. Roughly 1.5× the bundle β surface.
- **−** `LossyCast` needs explicit primitive impls or a wrapper around `num_traits`/`az`. Real work, but a one-time cost.
- **−** Doesn't include a closure form for custom rounding / unit conversion. That's a Framing 1 surface (or a follow-up addition).
- **−** Trusting `From`'s contract for `Cast`. For std primitives this is bulletproof; for user types it's a contract violation by them if invalid.

### Open question specific to Framing 2

Should `Cast<U>` be implemented at the *set* layer (`Cast<Interval<i64>> for Interval<i32>`) or at the *element* layer with the set layer deriving via blanket?

- Element layer: `impl<T, U> Cast<U> for FiniteBound<T> where T: Into<U>`, set-level impls then chain `bound.cast()`.
- Set layer direct: `impl<T, U> Cast<U> for FiniteInterval<T> where T: Into<U>, U: Element`, calls into element-level `Into` directly.

Element-layer-first is what `ops/math/add.rs:40` does for `TryAdd`. Same pattern works here. Recommended.

---

## Ecosystem trait alignment

Whether we should adopt or wrap traits from `num_traits` or other ecosystem crates rather than rolling our own. The question splits into two parts: **element-level primitive** (what `T -> U` machinery is the cast built on) and **set-level public API** (do we expose set-level traits in our crate, or have users go through some external trait).

### What's available

#### `num_traits` (already a default dep)

Defined in `num-traits-0.2.19/src/cast.rs`:

| Item | Shape | Use as element primitive? | Use as set-level API? |
|---|---|---|---|
| `ToPrimitive` | `fn to_i32(&self) -> Option<i32>` etc. (one method per primitive) | Indirectly (it's the source bound for `NumCast`). | No — primitive-target only. |
| `FromPrimitive` | `fn from_i32(n: i32) -> Option<Self>` etc. | Indirectly (companion to `ToPrimitive`). | No — primitive-source only. |
| `NumCast::from<T: ToPrimitive>(n: T) -> Option<Self>` | one fallible method, no error info | **Yes.** Covers all primitive pairs incl. `f64 <-> f32`. | No — element-level. |
| `cast::cast<T, U>(n: T) -> Option<U>` | free function sugar over `NumCast::from` | Same as `NumCast`. | No — free function, not a trait. |
| `AsPrimitive<T>::as_(self) -> T` | infallible `as`-semantics | **Mostly no.** `as` semantics are non-uniform: saturating for `f -> i`, wrapping for `i -> i`, INF for `f64 -> f32`. Not what saturating-cast wants. | No. |

**Critical detail re `NumCast`:** its doc explicitly admits *precision-loss / saturation-to-INF as success*: "even a large f64 saturating to f32 infinity" returns `Some(INF)`. So a `TryCast` built on `NumCast` is *not* automatically a strict cast — it must follow up with `Element::validate` (which the chokepoint pattern in the design already does). For `f64 -> f32` of `f64::MAX`, `NumCast` returns `Some(INF)` and `Element::validate` then rejects via `InvalidBoundLimit`. End-to-end semantics are correct.

`num_traits` does **not** offer a saturating cast trait. It does not offer a structured error for failed casts (just `Option`).

#### `az` crate (not currently a dep)

Designed precisely for casts:

- `az::Cast<U>::cast(self) -> U` — panics on overflow (debug), wraps on release for ints? Actually behavior is more nuanced — see crate docs. Generally: "infallible if you trust the conversion".
- `az::CheckedCast<U>::checked_cast(self) -> Option<U>` — returns None on overflow / NaN.
- `az::SaturatingCast<U>::saturating_cast(self) -> U` — clamps to U's range. Covers all primitive pairs *uniformly*, including `f64 -> f32` (clamps to `f32::MIN..=f32::MAX`).
- `az::WrappingCast<U>` — wrapping cast.
- `az::OverflowingCast<U>` — returns `(U, bool)`.
- `az::UnwrappedCast<U>` — unwraps the checked variant.

Maintained by tspiteri (rug ecosystem). Reasonably stable.

#### `conv` crate (older, less maintained)

- `ApproxFrom`, `SaturatingFrom`, `ValueFrom`, `UnwrappedFrom`, etc. with structured `RangeError`, `Underflow`, `Overflow` types.
- Comprehensive but last release 2021.

#### `cast` crate

- Free functions (`cast::i32(x) -> Result<i32, cast::Error>`) with structured error.
- Doesn't define traits; conversion-as-functions style. Hard to plug into a trait-based set-level API.

### Could we *be* a `num_traits` trait?

Could the set types implement a `num_traits` trait directly? **No.**

- `NumCast for Interval<U>` requires `Interval<U>: ToPrimitive`, which doesn't make sense (an interval isn't a single number).
- `AsPrimitive<Interval<U>> for Interval<T>` — the trait requires `T: 'static + Copy` and is intended for primitives.
- `From`/`TryFrom` blanket conflicts with the reflexive impl (Axis A4).

The set-level surface must be in-crate traits regardless. Ecosystem alignment is therefore a question only at the *element* layer.

### Could we use `az::SaturatingCast` directly as our element primitive?

Yes, if we add `az` as a dep. The trait shape matches what we need uniformly across all primitive pairs. Tradeoffs:

- **+** `az::SaturatingCast<U>` already has correct, audited impls for every primitive pair. We don't write or maintain them.
- **+** `az` is small and focused; not a heavy dep.
- **+** Users with custom `T` can implement `az::SaturatingCast<U> for MyT` and our trait picks it up automatically.
- **−** New transitive dep on `az` (currently zero). Crate count is a real cost in `no_std` / minimal-dep environments.
- **−** Couples our public element primitive to `az`'s contract. If `az` evolves its trait, we either pin the version or break.
- **−** `az` is less well-known than `num_traits`; users may have to learn it.

### Could we use `num_traits::NumCast` as our element primitive (B2)?

Already covered in Axis B. Recap:

- **+** Already a dep. No new transitive cost.
- **+** Covers every primitive pair (incl. float narrowing, with the INF caveat handled by `Element::validate`).
- **−** Returns `Option`, so our `CastError::Element` variant is content-free for the failed case. Compare to `az::CheckedCast` (also `Option`) or `cast::cast` (`Result` with structured error).
- **−** No lossy/saturating sibling. We'd still need a custom or `az`-based primitive for `LossyCast`.

### Combined options for element primitives

| Option | Element-fallible primitive (`TryCast`) | Lossy/saturating primitive (`LossyCast`) | New deps |
|---|---|---|---|
| **E1. All in-crate** | custom trait + macro impls | custom trait + macro impls | none |
| **E2. `num_traits` only (current scope)** | `NumCast` | custom trait + macro impls | none |
| **E3. `num_traits` + `az`** | `NumCast` (or `az::CheckedCast`) | `az::SaturatingCast` | `az` |
| **E4. `az` only** | `az::CheckedCast` | `az::SaturatingCast` | `az` |
| **E5. `conv`** | `conv::TryFrom` (with structured error) | `conv::SaturatingFrom` | `conv` |

E2 keeps the current dep footprint and reuses `num_traits` for the fallible side, which is the most natural fit since `num_traits` is already in scope. The saturating side is the only piece that's genuinely new code.

E3 adds `az` for `LossyCast` only — small, well-targeted dep; trades crate count for less crate-internal code.

E4/E5 are more uniform but pull in larger ecosystem deps; E5 in particular brings a 2021-era crate.

**Recommendation if Framing 2 is the direction:** **E2** as the default, with E3 as an alternative if the lossy-impl macro work feels heavy. If Framing 1 is the direction and `LossyCast` doesn't exist, **E2** trivially because `NumCast` covers all `TryCast` needs.

### Public-API stylistic question

Even with custom set-level traits, the *call site* can be styled to match ecosystem conventions:

```rust
// num_traits style — free function
let widened: Option<Interval<i64>> = intervalsets::cast::cast(narrow);

// az style — turbofish on inherent method
let widened: Interval<i64> = narrow.az::<Interval<i64>>();

// our trait style — turbofish on trait method
let widened: Interval<i64> = narrow.cast::<Interval<i64>>();
let widened: Interval<i64> = Cast::<Interval<i64>>::cast(narrow);
```

The trait-method style is what `Try*` already uses (`a.try_add(b)`). Stylistic consistency with the existing crate idiom argues for trait-method style.

---

## Viable bundles

Each bundle picks one option per axis. Some natural combinations:

### Bundle α — "Minimal trait"
**A1 (`TryCast<U>`) + B1 (`TryInto`) + C1 (strict) + D1 (strict)**

Smallest surface. `i32 <-> i64`, `u32 -> i64` all work; **no float narrowing**. Caller has to reach for `as` casts wrapped in their own closure helper for `f64 -> f32`.

- Tightest API, smallest doc footprint.
- Largest user-facing limitation. Likely insufficient if float narrowing is a target use case.

### Bundle β — "Primitive coverage"
**A1 + B2 (`NumCast`) + C1 + D1**

Covers every primitive cast (incl. float narrowing) with the discoverable trait. Strict bounds on collision.

- Most uniform "just works for numbers" feel.
- User types with custom `T` that aren't `NumCast` can't use the trait; they need a manual `TryCast` impl. Acceptable since they already implement `Element` manually.

### Bundle γ — "Primitive + escape hatch"
**A5 (trait + closure) + B2 + C1 + D1**

Bundle β plus a `try_map` closure form for lossy / unit / custom casts.

- Doubles API surface but covers every reasonable use case.
- Closure form is also the underlying primitive — trait delegates internally — so the implementation cost is mostly more rustdoc.

### Bundle δ — "Primitive + repair"
**A1 + B2 + C3 (strict + `try_cast_satisfy`) + D3 (strict + `try_cast_repaired`)**

Bundle β plus coercive siblings for callers who want narrowing-collapse to vanish silently.

- Surfaces both intents explicitly.
- More method names to remember; combinatorial growth if mixed with A5.

### Bundle ε — "Full kitchen"
**A5 + B4 (custom trait + NumCast blanket) + C3 + D3**

Everything.

- Most powerful and most extensible to user-defined `T`.
- Largest surface to learn, document, and maintain.

### Bundle ζ — "Three traits, three intents" (Framing 2)
**`Cast<U>` (`Into`-keyed, Tier 1) + `LossyCast<U>` (`Bounded`-keyed, Tier 1, lossy) + `TryCast<U>` (`NumCast`-keyed, Tier 3a, strict) + blanket `Cast => TryCast`**

No policy axes. Three call sites, three contracts. IntervalSet repair-vs-strict is implicit: `lossy_cast` repairs (consistent with "we already discarded distinctions"), `try_cast` is strict, `cast` doesn't break invariants.

- Strongest tier-contract alignment. Cleanest call-site reading.
- Three traits to maintain; `LossyCast` element impls are the bulk of the new code.
- Doesn't include a closure form. Could add `try_map` as a follow-up if escape-hatch demand materializes.

### Bundle ζ+ — "Three traits + closure escape hatch"
**Bundle ζ + `try_map` closure form**

For cases where none of `Cast` / `LossyCast` / `TryCast` fit (custom rounding, unit conversion, domain projections).

- Covers every reasonable use case.
- Four surfaces. Worth doing if `try_map` use cases come up; can be added later without breaking the three-trait surface.

## Verification (regardless of bundle)

Tests added in `intervalsets-core/src/ops/cast.rs` and `intervalsets/src/ops/cast.rs`:

- **Widening**: `FiniteInterval::closed(0_i32, 10).try_cast::<i64>()` round-trips. `f32 -> f64` likewise. Compile-check that `CastError<Infallible>` lets `let x: Result<_, _> = ...; let x = x.unwrap_or_else(|e| match e {});` work.
- **Element overflow**: `closed(0_i64, i64::MAX).try_cast::<i32>()` -> `CastError::Element(_)`.
- **Post-cast non-finite**: `closed(0.0_f64, f64::MAX).try_cast::<f32>()` -> `CastError::Set(InvalidBoundLimit)`.
- **Bound collision**: open-open pair of `f64` values that round to the same `f32`. Strict variant -> `Set(InvalidBoundPair)`; coercive variant (if bundle has it) -> `Empty`.
- **HalfInterval**: side preserved through cast.
- **EnumInterval::Unbounded**: `Unbounded` for any `U`.
- **IntervalSet sort**: monotone cast preserves order. Two-interval collapse: strict variant errors; repair variant merges and matches `IntervalSet::new(...)` semantics.
- **Closure form (if bundle has it)**: `try_map(|x: f64| Ok::<_, Infallible>(x as f32))`, `try_map(|x: f64| if x.is_normal() { Ok(x as f32) } else { Err(MyErr) })`.
- **Generic call site (trait only)**: a function generic over `T: TryCast<U, Output = ...>` compiles and runs.

Run: `cargo test -p intervalsets-core ops::cast && cargo test -p intervalsets ops::cast`.

## Files involved (any bundle)

- **New:** `intervalsets-core/src/ops/cast.rs`
- **New:** `intervalsets/src/ops/cast.rs`
- `intervalsets-core/src/ops/mod.rs` — module decl + re-export + tier-3a member list (lines 89-92)
- `intervalsets-core/src/error.rs` — `CastError<E>` after `MathError` (around line 98)
- `intervalsets-core/src/bound.rs` — impl block near `FiniteBound::map` (line 203)
- `intervalsets-core/src/prelude.rs`, `intervalsets/src/prelude.rs` — re-exports
