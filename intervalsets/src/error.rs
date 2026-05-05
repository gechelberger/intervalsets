//! Errors returned by fallible `intervalsets` APIs.
//!
//! [`Error`] wraps and hides [`intervalsets_core::error::Error`] so the
//! outer-crate surface never names a core type. The wrap exists so that
//! richer, alloc-enabled error context (messages, source chains, the
//! offending value) can be added additively in the future without
//! touching the core's no-alloc contract.

use intervalsets_core::error::Error as CoreError;
use thiserror::Error as ThisError;

/// Errors returned by fallible `intervalsets` APIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, ThisError)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    TotalOrder(#[from] TotalOrderError),

    /// The counting measure of a set cannot be represented by the
    /// `Countable::Output` type (e.g. counting `[i32::MIN, i32::MAX]`
    /// overflows `i32`).
    #[error("count overflows the Countable Output type")]
    CountOverflow,

    /// Bound-pair invariants violated: crossed bounds in a finite
    /// interval, or a structurally invalid `OrdBoundPair`.
    #[error("interval or bound-pair invariants violated (crossed bounds, or structurally invalid OrdBoundPair)")]
    InvalidBoundPair,

    /// An [`IntervalSet`](crate::IntervalSet)'s stored intervals
    /// violated its invariants: an empty interval was stored,
    /// intervals were not in ascending order, or two consecutive
    /// intervals were connected (would have been merged in canonical
    /// form). Outer-crate-only; core has no `IntervalSet`.
    #[error("interval set invariants violated")]
    InvalidIntervalSet,
}

/// Failed comparison of `PartialOrd` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ThisError)]
#[error("incomparable values")]
pub struct TotalOrderError;

impl From<CoreError> for Error {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::TotalOrderError(_) => Error::TotalOrder(TotalOrderError),
            CoreError::CountOverflow => Error::CountOverflow,
            CoreError::InvalidBoundPair => Error::InvalidBoundPair,
            // CoreError is #[non_exhaustive]; if a new variant is added,
            // this `From` lift must be extended to map it. Leaving the
            // wildcard as `unreachable!` makes the gap surface as a runtime
            // panic during the first conversion of the new variant.
            _ => unreachable!(
                "intervalsets_core::error::Error gained a variant that intervalsets::error::Error does not yet map"
            ),
        }
    }
}

impl From<intervalsets_core::error::TotalOrderError> for Error {
    fn from(_: intervalsets_core::error::TotalOrderError) -> Self {
        Error::TotalOrder(TotalOrderError)
    }
}
