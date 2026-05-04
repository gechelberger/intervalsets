//! Set-Arithmetic on intervals.
//!
//! Arithmetic operators (`+ - * /`) are implemented for
//! [`FiniteInterval`](crate::sets::FiniteInterval),
//! [`HalfInterval`](crate::sets::HalfInterval), and
//! [`EnumInterval`](crate::sets::EnumInterval), along with the
//! cross-type combinations needed for ergonomic chaining.
//!
//! # Panicking and fallible forms
//!
//! Each operation is offered in two flavors:
//!
//! - The infix operator (`+ - * /`) is the panicking, ergonomic
//!   form. It requires `T: Ord` so that `partial_cmp` on bounds is
//!   total and arithmetic on bounds is provably infallible; the
//!   underlying try-form's `.unwrap()` can never panic.
//! - The corresponding [`TryAdd`] / [`TrySub`] / [`TryMul`] /
//!   [`TryDiv`] trait returns `Result<_, Error>` and requires only
//!   `T: PartialOrd`. NaN-induced incomparability surfaces as
//!   `Err`, never as a panic.
//!
//! Float users without an [`Ord`]-providing wrapper (raw `f32` /
//! `f64`) cannot satisfy the infix operator bounds. Wrap floats in
//! `OrderedFloat` (with the `ordered-float` feature) to restore the
//! infix operators, or use the `Try*` traits directly.
//!
//! # Output shape
//!
//! Division can produce up to two pieces (e.g. `[1, 2] / [-1, 1]`
//! is unbounded with a hole at zero), so its output is a
//! [`MaybeDisjoint`](crate::disjoint::MaybeDisjoint). Add, sub, and
//! mul produce a single interval.
//!
//! # Overflow
//!
//! Arithmetic on bounds delegates to the underlying type's
//! [`Add`](::core::ops::Add) / [`Sub`](::core::ops::Sub) /
//! [`Mul`](::core::ops::Mul) / [`Div`](::core::ops::Div). Overflow
//! behavior is whatever those impls do — `i32` panics in debug and
//! wraps in release, `Wrapping<T>` always wraps, `checked_*` is not
//! used. Callers needing defined overflow should pick a numeric type
//! that provides it.

mod add;
mod sub;
mod mul;
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
