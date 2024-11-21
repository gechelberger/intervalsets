
#[derive(Debug, thiserror::Error)]
pub enum Error {

    #[error(transparent)]
    TotalOrderError(#[from] TotalOrderError),

    #[error(transparent)]
    InvariantError(#[from] InvariantError),

    #[error(transparent)]
    BoundsViolationError(#[from] BoundsViolationError),
}

/// Failed comparison of `PartialOrd` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[derive(::thiserror::Error)]
#[error("failed comparison of unordered values: {msg}")]
pub struct TotalOrderError {
    msg: &'static str,
}

impl TotalOrderError {
    /// Creates a new `TotalOrderError` with a static message.
    pub const fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}

/// A type invariant has been violated.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[derive(thiserror::Error)]
#[error("invariant violated: {msg}")]
pub struct InvariantError {
    msg: &'static str,
}

impl InvariantError {
    /// Creates a new `InvariantError` with a static message.
    pub const fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[derive(thiserror::Error)]
#[error("bounds violation error: {msg}")]
pub struct BoundsViolationError {
    msg: &'static str
}

impl BoundsViolationError {
    /// Create a new `BoundViolationError` with a static message.
    pub const fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}