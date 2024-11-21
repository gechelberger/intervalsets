use core::error::Error;
use core::fmt;

/// Failed comparison of `PartialOrd` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TotalOrderError {
    msg: &'static str,
}

impl TotalOrderError {
    /// Creates a new `TotalOrderError` with a static message.
    pub const fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}

impl fmt::Display for TotalOrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TotalOrderError: {}.", self.msg)
    }
}

impl Error for TotalOrderError {}
