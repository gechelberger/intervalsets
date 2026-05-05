#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    TotalOrderError(#[from] TotalOrderError),

    /// Bound-pair invariants violated. Covers two related conditions:
    ///
    /// 1. **Crossed bounds in a `FiniteInterval`** — `lhs > rhs` after
    ///    normalization. Raised by
    ///    [`FiniteInterval::try_new`](crate::sets::FiniteInterval::try_new)
    ///    and the interval types' `Deserialize` paths.
    /// 2. **Structural `OrdBoundPair` violations** — an `OrdBound` of
    ///    the wrong kind for its position (e.g. `RightUnbounded` on
    ///    the left), an unmatched empty marker, or `left.value() > right.value()`.
    ///    Raised by
    ///    [`OrdBoundPair::try_new`](crate::bound::ord::OrdBoundPair::try_new).
    ///
    /// Both contexts share this variant because callers rarely need to
    /// distinguish them in error handling. If a future use case
    /// requires distinguishing, this variant can be split additively
    /// (the enum is `#[non_exhaustive]`).
    #[error("interval or bound-pair invariants violated (crossed bounds, or structurally invalid OrdBoundPair)")]
    InvalidBoundPair,

    #[error(transparent)]
    MidpointError(#[from] MidpointError),
}

/// Failed comparison of `PartialOrd` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[error("incomparable values")]
pub struct TotalOrderError;

/// Inputs to [`Midpoint`](crate::numeric::Midpoint) were not suitable
/// for computing a midpoint — either incomparable (NaN) or non-finite
/// (±∞), both of which are degenerate as midpoint endpoints.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[error("midpoint requires finite, comparable values")]
pub struct MidpointError;
