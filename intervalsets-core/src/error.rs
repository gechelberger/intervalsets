#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    TotalOrderError(#[from] TotalOrderError),

    /// The counting measure of a set cannot be represented by the
    /// `Countable::Output` type (e.g. counting `[i32::MIN, i32::MAX]`
    /// overflows `i32`).
    #[error("count overflows the Countable Output type")]
    CountOverflow,

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
}

/// Failed comparison of `PartialOrd` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[error("incomparable values")]
pub struct TotalOrderError;
