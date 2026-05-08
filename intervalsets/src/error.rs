//! Errors returned by fallible `intervalsets` APIs.
//!
//! [`Error`] is the unified error type for set-algebra and bound
//! operations. Set ops chain into other set ops, so a single error
//! type along that chain keeps `?`-propagation simple.
//!
//! Terminal failures that do not chain into further set operations
//! (e.g. [`CountOverflowError`](crate::measure::CountOverflowError)
//! from [`Count::try_count`](crate::measure::Count::try_count), which
//! yields a scalar count) are returned as their own precise types and
//! live next to their producer instead of in this module.
//!
//! [`Error`] wraps [`intervalsets_core::error::Error`] so the variants
//! it carries can grow alloc-enabled context (messages, source chains,
//! the offending value) without touching core's no-alloc contract.
//! [`TotalOrderError`] is re-exported from core verbatim — it has no
//! payload to enrich.

use intervalsets_core::error::Error as CoreError;
pub use intervalsets_core::error::TotalOrderError;
use thiserror::Error as ThisError;

/// Errors returned by fallible `intervalsets` APIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, ThisError)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    TotalOrderError(#[from] TotalOrderError),

    /// Bound-pair invariants violated. Covers two related conditions:
    ///
    /// 1. **Crossed bounds in a `FiniteInterval`** — `lhs > rhs` after
    ///    normalization. Raised by
    ///    [`FiniteInterval::try_new`](intervalsets_core::sets::FiniteInterval::try_new)
    ///    and the interval types' `Deserialize` paths.
    /// 2. **Structural `OrdBoundPair` violations** — an `OrdBound` of
    ///    the wrong kind for its position (e.g. `RightUnbounded` on
    ///    the left), an unmatched empty marker, or
    ///    `left.value() > right.value()`. Raised by
    ///    [`OrdBoundPair::try_new`](crate::bound::ord::OrdBoundPair::try_new).
    ///
    /// Both contexts share this variant because callers rarely need
    /// to distinguish them in error handling. If a future use case
    /// requires distinguishing, this variant can be split additively
    /// (the enum is `#[non_exhaustive]`).
    #[error("interval or bound-pair invariants violated (crossed bounds, or structurally invalid OrdBoundPair)")]
    InvalidBoundPair,

    /// An [`IntervalSet`](crate::IntervalSet)'s stored intervals
    /// violated its invariants: an empty interval was stored,
    /// intervals were not in ascending order, or two consecutive
    /// intervals were connected (would have been merged in canonical
    /// form). Raised by [`IntervalSet::try_new`](crate::IntervalSet::try_new)
    /// and the `Deserialize` path on outer-crate set types.
    #[error("interval set invariants violated")]
    InvalidIntervalSet,

    /// A `FiniteBound`'s limit value was rejected by
    /// [`Element::validate`](intervalsets_core::numeric::Element::validate).
    /// Library float types reject `±INF` and `NaN` here; user types
    /// override `validate` to enforce their own predicate.
    #[error("bound limit rejected by Element::validate")]
    InvalidBoundLimit,
}

impl From<CoreError> for Error {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::TotalOrderError(e) => Error::TotalOrderError(e),
            CoreError::InvalidBoundPair => Error::InvalidBoundPair,
            CoreError::InvalidBoundLimit => Error::InvalidBoundLimit,
            // CoreError is #[non_exhaustive]; if a new variant is added,
            // this `From` lift must be extended to map it. The wildcard
            // surfaces the gap as a runtime panic on first conversion.
            _ => unreachable!(
                "intervalsets_core::error::Error gained a variant that intervalsets::error::Error does not yet map"
            ),
        }
    }
}
