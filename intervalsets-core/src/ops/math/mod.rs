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
//! Each operation is offered in two flavors per Tier 3 (see
//! [`crate::ops`]):
//!
//! - The [`TryAdd`] / [`TrySub`] / [`TryMul`] / [`TryDiv`] traits
//!   are **Tier 3a**: total, panic-free in release. `Err` covers
//!   incomparable bounds (NaN), integer overflow / signed
//!   `MIN / -1` (`MathError::Range`), integer divide-by-zero /
//!   non-finite float result (`MathError::Domain`), or any
//!   user-defined error. Bound: `T: PartialOrd` (raw floats are
//!   accepted; they surface their failure modes as `Err`).
//! - The infix operators (`+ - * /`) are **Tier 3b**: panicking
//!   sugar defined as `lhs.try_op(rhs).unwrap()`. They **may panic
//!   in release and debug** when the corresponding `try_*` would
//!   have returned `Err`. The panic site is part of the documented
//!   contract.
//!
//! Floats without an [`Ord`]-providing wrapper (raw `f32` / `f64`)
//! used to be barred from the infix operators by a `T: Ord` bound.
//! That bound is gone — they are now barred by the more honest fact
//! that `<T as TryAdd>::Error: Debug` is required for the unwrap.
//! Wrap floats in `OrderedFloat` (with the `ordered-float` feature)
//! to access the infix operators ergonomically; use the `Try*`
//! traits directly for the panic-free contract.
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
//! [`TryAdd`] / [`TrySub`] / [`TryMul`] / [`TryDiv`]. Library
//! integer impls use `checked_*` and surface overflow as
//! `Err(MathError::Range)`; library float impls check
//! `is_finite()` after each op and surface `INF`/`NaN` as
//! `Err(MathError::Domain)`. The infix operators panic on these
//! errors per Tier 3b. Callers needing wrapping behavior should use
//! a `Wrapping<T>` newtype that supplies the wrapping semantics in
//! its own `TryAdd` impl, or pre-validate operands.

mod add;
mod div;
pub(crate) mod macros;
mod mul;
mod sub;

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
