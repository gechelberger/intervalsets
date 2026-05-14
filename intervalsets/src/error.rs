//! Errors returned by fallible `intervalsets` APIs.
//!
//! [`Error`] is the unified error type for set-algebra and bound
//! operations. Set ops chain into other set ops, so a single error
//! type along that chain keeps `?`-propagation simple.
//!
//! Terminal failures that do not chain into further set operations
//! (e.g. [`MathError`] from
//! [`Cardinality::try_cardinality`](crate::measure::Cardinality::try_cardinality),
//! which yields a scalar cardinality) surface as their own precise types
//! and live next to their producer instead of in this module.
//!
//! [`Error`] wraps [`intervalsets_core::error::Error`] so the variants
//! it carries can grow alloc-enabled context (messages, source chains,
//! the offending value) without touching core's no-alloc contract.
//! [`TotalOrderError`] is re-exported from core verbatim — it's the
//! return type of `TryCmp::try_cmp` for callers that want the precise
//! single-bit "incomparable" failure; in the umbrella `Error` it
//! collapses to [`InvalidBoundLimit`](Error::InvalidBoundLimit).

use intervalsets_core::error::Error as CoreError;
pub use intervalsets_core::error::{MathError, TotalOrderError};
use thiserror::Error as ThisError;

/// Errors returned by fallible `intervalsets` APIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, ThisError)]
#[non_exhaustive]
pub enum Error {
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

    /// A bound's limit value was rejected as a valid bound limit.
    /// Covers both `Element::validate` rejection and `partial_cmp`
    /// failure (NaN-style incomparability) — see the core variant
    /// docs for details.
    #[error("bound limit rejected (validate or partial_cmp failure)")]
    InvalidBoundLimit,

    /// Arithmetic-on-bounds failure surfaced by a `try_*` math impl —
    /// integer overflow / signed `MIN / -1` (`MathError::Range`),
    /// integer divide-by-zero or non-finite float result
    /// (`MathError::Domain`).
    #[error(transparent)]
    Math(#[from] MathError),
}

impl From<TotalOrderError> for Error {
    fn from(_: TotalOrderError) -> Self {
        Error::InvalidBoundLimit
    }
}

impl From<CoreError> for Error {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::InvalidBoundPair => Error::InvalidBoundPair,
            CoreError::InvalidBoundLimit => Error::InvalidBoundLimit,
            CoreError::Math(m) => Error::Math(m),
            // CoreError is #[non_exhaustive]; if a new variant is added,
            // this `From` lift must be extended to map it. The wildcard
            // surfaces the gap as a runtime panic on first conversion.
            _ => unreachable!(
                "intervalsets_core::error::Error gained a variant that intervalsets::error::Error does not yet map"
            ),
        }
    }
}
