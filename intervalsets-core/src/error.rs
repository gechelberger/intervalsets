use core::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    InvertedBoundsError,
    BoundsMismatchError,
    MultipleIntervalsError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvertedBoundsError => write!(f, "inverted bounds error"),
            Self::BoundsMismatchError => write!(f, "bounds mismatch error"),
            Self::MultipleIntervalsError => write!(f, "multiple possible intervals"),
        }
    }
}

impl core::error::Error for Error {
    fn description(&self) -> &str {
        "todo"
    }
}
