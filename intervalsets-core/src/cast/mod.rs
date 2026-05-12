//! Storage-type casts: convert the element type of a set without
//! reconstructing it.
//!
//! Three traits, three intents:
//!
//! - [`Cast<U>`] — **Tier 1**, infallible. For pairs where the element
//!   conversion `T -> U` cannot fail under the standard library's
//!   contract: `i32 -> i64`, `f32 -> f64`, `i8 -> f32`, etc. Keyed on
//!   `T: Into<U>` at the element layer.
//! - [`LossyCast<U>`] — **Tier 1**, total but lossy. Clamps out-of-range
//!   elements to `U`'s extrema and rounds in-range elements to the
//!   nearest representable `U`. For `f64 -> f32`, `i64 -> i32`, float
//!   `-> int`, etc. Keyed on [`LossyCastElement`], which has a blanket
//!   impl over [`az::SaturatingCast`].
//! - [`TryCast<U>`] — **Tier 3a**, strict. Returns `Err` on element
//!   overflow, post-cast non-finite, crossed bounds, or set-invariant
//!   violation. Keyed via [`NumCast`](num_traits::NumCast) inside the
//!   set-level impls.
//!
//! # Validation chokepoints
//!
//! A correct cast routes through the same chokepoints as any other
//! validating constructor:
//!
//! 1. `FiniteBound::try_new` runs `Element::validate` on the post-cast
//!    value (catches `f64::MAX -> f32::INFINITY` style post-cast
//!    non-finite results).
//! 2. `FiniteInterval::try_new` enforces the bound-pair invariant
//!    after both bounds have been cast (catches narrowing-induced
//!    crossed bounds).
//! 3. `IntervalSet::try_new` enforces the set invariants after every
//!    interval has been cast (catches narrowing-induced overlap).
//!
//! [`TryCast`] always routes through the strict (`try_new`) chokepoints.
//! [`LossyCast`] routes through coercive siblings where one exists
//! (`try_satisfy_bounds` for [`FiniteInterval`](crate::sets::FiniteInterval),
//! `IntervalSet::new` for the outer-crate `IntervalSet`) — consistent
//! with "we already discarded distinctions at the element layer".
//! [`Cast`] uses Tier 4 bypass constructors because monotonic widenings
//! are guaranteed to preserve every invariant.

mod element;
mod sets;

// Re-export the sealed `Primitive` marker for use by `feat/*.rs`
// modules that need to bound generic impls on "the std numeric
// primitives only" (e.g. `BigDecimal -> U_primitive` impls).
pub(crate) use element::Primitive;

/// Infallible storage-type cast. **Tier 1**.
///
/// Implemented for pairs where the element conversion is contractually
/// total and information-preserving — the standard pairs for which
/// `From<T> for U` exists in std (`i32 -> i64`, `f32 -> f64`,
/// `i8 -> f32`, etc.).
///
/// For pairs where the conversion can fail or lose information, use
/// [`TryCast`] or [`LossyCast`].
pub trait Cast<U> {
    /// The post-cast type.
    type Output;
    /// Casts `self` to its `U`-storage equivalent.
    fn cast(self) -> Self::Output;
}

/// Total but lossy storage-type cast. **Tier 1**.
///
/// Cannot fail; values that don't fit in `U` are projected onto `U`'s
/// representable lattice:
///
/// - Out-of-range elements clamp to `U`'s extremum (`f64::MAX`
///   clamps to `f32::MAX`, not `f32::INFINITY`).
/// - In-range elements round to the nearest representable `U`.
///
/// When a bound's value saturates to `U`'s extremum, its
/// [`BoundType`](crate::bound::BoundType) snaps to
/// [`Closed`](crate::bound::BoundType::Closed): the open/closed
/// distinction at the discarded boundary is meaningless once the
/// out-of-range region has been projected away.
///
/// At the set layer, `LossyCast` for
/// [`FiniteInterval`](crate::sets::FiniteInterval) routes crossed
/// bounds (two distinct `T`s collapsing to the same `U`) to `Empty`
/// rather than erroring, and `LossyCast` for the outer-crate
/// `IntervalSet` routes through the repairing constructor —
/// consistent with "we already discarded distinctions".
///
/// # NaN caveat
///
/// `az::SaturatingCast` (the underlying element-layer primitive) panics
/// when asked to saturate `NaN` to an integer type. `Element::validate`
/// rejects NaN at construction time for every library float type, so
/// the validating API cannot route NaN into `LossyCast`. The Tier 4
/// `new_assume_valid` bypass can; misuse there is documented as
/// out-of-contract.
pub trait LossyCast<U> {
    /// The post-cast type.
    type Output;
    /// Projects `self` onto its `U`-storage representation. Lossy.
    fn lossy_cast(self) -> Self::Output;
}

/// Fallible storage-type cast. **Tier 3a**.
///
/// Returns `Err` on any of:
///
/// 1. Element overflow (`NumCast::from` returns `None`).
/// 2. Post-cast value rejected by [`Element::validate`](crate::numeric::Element::validate)
///    (e.g. `f64::MAX -> f32::INFINITY` rejected as non-finite).
/// 3. Post-cast bound pair is crossed or open-open at equality
///    (returned as [`Error::InvalidBoundPair`](crate::error::Error::InvalidBoundPair)).
/// 4. (`IntervalSet` only) post-cast intervals violate set invariants
///    (sort, disjointness, non-touching).
pub trait TryCast<U> {
    /// The post-cast type, returned in the `Ok` branch.
    type Output;
    /// The error type returned when any cast chokepoint rejects the
    /// input.
    type Error;
    /// Attempts to cast `self` to its `U`-storage equivalent. Returns
    /// `Err` on element overflow, post-cast non-finite, crossed
    /// bounds, or set-invariant violation.
    fn try_cast(self) -> Result<Self::Output, Self::Error>;
}

/// Element-layer hook for [`LossyCast`]. Blanket-implemented over
/// [`az::SaturatingCast`], so every primitive numeric pair (and every
/// user `T` that opts in via `az::SaturatingCast<U>`) gets
/// [`LossyCast`] for the corresponding set types for free.
///
/// Implementers of new element types typically don't need to touch this
/// trait directly — implementing `az::SaturatingCast<U> for MyT`
/// suffices.
pub trait LossyCastElement<U> {
    /// Projects `self` onto the nearest representable `U`.
    fn lossy_cast_element(self) -> U;
}

/// Element-layer hook for [`Cast`] (the infallible cast). For primitive
/// pairs there's a blanket impl over `T: Into<U>` keyed on the sealed
/// `Primitive` marker; feat storage types provide their own impls in
/// their `feat/<type>.rs` modules.
///
/// # Contract
///
/// `cast_element` is **infallible by precondition**: implementors may
/// assume the input value is a valid bound limit for `U` (i.e.
/// `U::validate(x)` would accept the post-cast value). Callers from
/// the set-level [`Cast`] impls always satisfy this — the input came
/// from a [`FiniteBound<T>`](crate::bound::FiniteBound) that already
/// passed `T::validate`, and the chosen impl for `(T, U)` is one where
/// validity is preserved under the conversion.
///
/// Impls for cases where this assumption could be violated (e.g.
/// `CastElement<BigDecimal> for f64`, where `f64::NaN` would map to
/// `BigDecimal::try_from`'s `Err`) document the assumption in their
/// impl block and may `.expect()` internally. Tier 4 `new_assume_valid`
/// bypass that puts NaN in a `FiniteBound<f64>` would reach the
/// `.expect()` and panic — documented bypass-misuse, not a contract
/// violation.
pub trait CastElement<U> {
    /// Casts `self` to its `U` representation. See trait docs for the
    /// precondition.
    fn cast_element(self) -> U;
}

/// Element-layer hook for [`TryCast`]. Returns `None` when the value
/// cannot be represented in `U` (overflow, NaN, etc.).
///
/// All primitive pairs are blanket-implemented via
/// [`NumCast`](num_traits::NumCast); see `cast::element` for the
/// implementation. Feature-gated storage types
/// (`bigdecimal::BigDecimal`, `num_bigint::BigInt`,
/// `rust_decimal::Decimal`, `fixed::Fixed*`) cannot impl
/// `NumCast` themselves (orphan rule) and instead provide their own
/// `TryCastElement` impls in their `feat/<type>.rs` modules.
///
/// Implementers of new `T` types should impl this trait for each `U`
/// they want to support as a cast target / source.
pub trait TryCastElement<U> {
    /// Attempts to cast `self` to its `U` representation. Returns
    /// `None` if the value doesn't fit (overflow, NaN, etc.).
    fn try_cast_element(self) -> Option<U>;
}
