mod add;
mod sub;

#[doc(hidden)]
pub mod mul;

mod div;

/// Add that returns Result instead of panicking on logical violations.
///
/// The infix `+` operator panics if the operation would produce an
/// invalid bound (e.g., a NaN result). `TryAdd::try_add` returns
/// `Result<_, Self::Error>` so panic-free callers can detect and
/// handle failure.
pub trait TryAdd<Rhs = Self> {
    /// The type produced by a successful addition.
    type Output;
    /// The error returned when the operation cannot produce a valid result.
    type Error;
    /// Add `self` and `rhs`, returning `Err` instead of panicking.
    fn try_add(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

/// Sub that returns Result instead of panicking on logical violations.
///
/// See [`TryAdd`] for the convention.
pub trait TrySub<Rhs = Self> {
    /// The type produced by a successful subtraction.
    type Output;
    /// The error returned when the operation cannot produce a valid result.
    type Error;
    /// Subtract `rhs` from `self`, returning `Err` instead of panicking.
    fn try_sub(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

/// Mul that returns Result instead of panicking on logical violations.
///
/// See [`TryAdd`] for the convention.
pub trait TryMul<Rhs = Self> {
    /// The type produced by a successful multiplication.
    type Output;
    /// The error returned when the operation cannot produce a valid result.
    type Error;
    /// Multiply `self` and `rhs`, returning `Err` instead of panicking.
    fn try_mul(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

/// Div that returns Result instead of panicking on logical violations.
///
/// See [`TryAdd`] for the convention.
pub trait TryDiv<Rhs = Self> {
    /// The type produced by a successful division.
    type Output;
    /// The error returned when the operation cannot produce a valid result.
    type Error;
    /// Divide `self` by `rhs`, returning `Err` instead of panicking.
    fn try_div(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}
