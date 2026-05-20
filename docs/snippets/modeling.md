An interval is a subset of some mathematical universe `S′` (the reals
`ℝ`, the integers `ℤ`, the decimal rationals `ℚ₁₀`, …), but the element
type `T` you supply is a Rust type with a finite, discrete set of
bit-patterns. Three layers are at play, and they generally do not
coincide:

| Layer | Symbol | What it is |
|---|---|---|
| storage type | `T` | every bit-pattern the Rust type can hold — including ones the library treats as invalid (e.g. `NaN`, `±∞` for floats) |
| representable universe | `S` | the finite subset of `T` admitted as a valid `Element`|
| universe | `S′` | the mathematical set the interval semantically inhabits — `ℝ` for `f64`, `ℤ` for `i32`, `ℕ₀` for `u32`, `ℚ₁₀` for `BigDecimal` |

Under the natural embedding of `T` into mathematical objects, the clean
identity is **`S = T ∩ S′`**. Limits and member-tests live in `S`;
intervals themselves are subsets of `S′`.

## Why three layers

`S′` may strictly exceed `T`. `Interval<i32>::unbounded()` denotes all
of `ℤ` — 2³¹, 10¹⁰⁰, and every integer in between — not just the
roughly 4.3 × 10⁹ values that fit in `i32`. Half-bounded and unbounded
intervals exist *because* the abstract universe is in general larger
than what the storage type can hold — they are shapes a set inhabits,
not specific Rust types.

`T` may also strictly exceed `S′`. For `f64`, the bit-patterns `NaN`,
`+∞`, `-∞` have no real-number referent at all; they sit outside `S′`,
not merely outside `S`. The library rejects them as candidate limits —
that is the job of `Element::validate`. 

User defined types may enforce their own universe `S'` constraints, so long
as they are consistent with other invariants.

## Consequences

- **`Unbounded` is unbounded in `S′`, not in `T`.** `IntoFiniteInterval`
  clamps an abstract-extent interval down to the representable range of
  `T` when that is what you want. It is gated on `T: num_traits::Bounded`
  — the clamp uses `T::min_value()` / `T::max_value()` — so types with
  no finite extremum (`BigInt`, `BigDecimal`) do not get the operation.
- **`measure` lives in different universes per `T`.** `Measure` returns
  the natural additive measure of the set: for discrete `T` that is
  cardinality (a count on `S`, the representable elements); for
  continuous `T` that is Lebesgue width (the abstract extent in `S′`).
  For an `i32` interval `[0, 5]`, `measure = 6`. For an `f64` interval
  `[0.0, 1.0]`, `measure = 1.0`. The continuous singleton `[5.0, 5.0]`
  has `measure = 0.0` — its Lebesgue width — *not* `1` (cardinality is
  not the natural measure on a continuous `T`).
- **Member-testing accepts any `T`, but only resolves in `S`.**
  `contains(x)` takes `x: T`, so every `T`-valued query is legitimate.
  Values in `S` get a truthful yes/no. Values in `T \ S` — e.g. `NaN`
  for `f64` — are incomparable with the bounds and collapse to `false`
  rather than panicking.
- **Complement reaches beyond `T`.** `[i32::MIN, i32::MAX].complement()`
  is a non-empty set in `ℤ` (every integer outside the `i32` range), but
  `contains(x)` returns `false` for every `x: i32` — there is no `T`
  value that witnesses it.

## Storage-type cases

The combinatorics of `(T, S, S′)` shake out into four canonical patterns:

| Case | Example `T` | `T` vs `S` | `S` vs `S′` |
|---|---|---|---|
| fixed-precision discrete | `i32`, `u64` | `T = S` | `S ⊊ S′` (more integers exist than `T` can hold) |
| arbitrary-precision discrete | `BigInt` | `T = S` | `S = S′` (the only case where all three layers coincide) |
| lossy continuous | `f64`, `OrderedFloat<f64>` | `S ⊊ T` (`NaN`/`±∞` excluded) | `S ⊊ S′` (only countably many representable reals) |
| lossless decimal / rational | `BigDecimal` | `T = S` | `S ⊊ S′` (no irrationals; `S` has Lebesgue measure zero in `ℝ`) |

Choosing a `T` is choosing where to put the gap. `BigDecimal` looks
"fully precise" but cannot represent `√2`; `f64` is fast but admits
`NaN` as a failure mode; `i32` is exact within its range but every
integer outside that range is unrepresentable. The right choice
depends on whether your problem's true universe `S′` is `ℤ`, `ℝ`, or
something narrower — and on which of `T \ S′`, `S′ \ T`, or neither is
the one you can tolerate.
