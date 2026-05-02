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

    /// An [`OrdBoundPair`](crate::bound::ord::OrdBoundPair) did not
    /// match a valid bit pattern (e.g. after deserialization from an
    /// archive whose invariants weren't validated).
    #[error("OrdBoundPair did not match a valid bit pattern")]
    InvalidBoundPair,
}

/// Failed comparison of `PartialOrd` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[error("incomparable values")]
pub struct TotalOrderError;
