//! A Measure is a function of a set that gives a comparable size between sets.
//!
//! They must obey the following invariants:
//!
//! ```ignore
//! Let m(S) be our measure.
//!
//! 1) Monotonicity:
//!     If A is subset of B then m(A) <= m(B)
//!
//! 2) Subadditivity:
//!     If A0, A1, .. An is a countable set of possibly intersecting sets:
//!         m(A0 U A1 .. An) <= Sum { m(Ai) for i in 0..n }
//! ```
//!
//! Some common measures are Cardinality, Count, and
//! the Lebesgue measure which is Width in R1.

mod count;
pub use count::{Count, Countable};
mod width;
pub use width::Width;

/// The result of applying a Measure to an Interval/Set.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Measurement<T> {
    Finite(T),
    Infinite,
}

impl<T> Measurement<T> {
    /// Returns `true` if the measurement is an [`Infinite`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets::measure::Measurement;
    ///
    /// let x: Measurement<u32> = Measurement::Finite(10);
    /// assert_eq!(x.is_infinite(), false);
    ///
    /// let x: Measurement<u32> = Measurement::Infinite;
    /// assert_eq!(x.is_infinite(), true);
    /// ```
    pub fn is_infinite(&self) -> bool {
        matches!(self, Self::Infinite)
    }

    /// Returns `true` if the measurement is a [`Finite`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets::measure::Measurement;
    ///
    /// let x: Measurement<u32> = Measurement::Finite(10);
    /// assert_eq!(x.is_finite(), true);
    ///
    /// let x: Measurement<u32> = Measurement::Infinite;
    /// assert_eq!(x.is_finite(), false);
    /// ```
    pub fn is_finite(&self) -> bool {
        matches!(self, Self::Finite(_))
    }

    /// Returns the contained Finite measurements, consuming self.
    ///
    /// # Panics
    ///
    /// Panics if self is [`Infinite`] with a custom panic msg.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets::measure::Measurement;
    ///
    /// let x: Measurement<i32> = Measurement::Finite(0);
    /// assert_eq!(x.expect_finite("Measurement should be finite"), 0);
    /// ```
    ///
    /// ```should_panic
    /// use intervalsets::measure::Measurement;
    ///
    /// let x: Measurement<i32> = Measurement::Infinite;
    /// assert_eq!(x.expect_finite("Measurement should be finite"), 0);
    /// ```
    pub fn expect_finite(self, msg: &str) -> T {
        match self {
            Self::Finite(inner) => inner,
            _ => panic!("{}", msg),
        }
    }

    /// Returns the contained Finite measurement, consuming self.
    ///
    /// See also: [`expect_finite`](enum.Measurement.html#method.expect_finite)
    pub fn finite(self) -> T {
        self.expect_finite("Measurement should be finite")
    }

    /// Returns the contained Finite value or a provided default.
    pub fn finite_or(self, default: T) -> T {
        match self {
            Self::Finite(inner) => inner,
            _ => default,
        }
    }

    /// Returns Infinite if the measurement is Infinite, otherwise
    /// calls `func` with the Finite value and returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets::measure::Measurement;
    ///
    /// let m1 = Measurement::Finite(10);
    /// let m2 = m1.flat_map(|x| Measurement::Finite(x as f32));
    /// assert_eq!(m2.finite(), 10.0);
    /// ```
    pub fn flat_map<F, U>(self, func: F) -> Measurement<U>
    where
        F: FnOnce(T) -> Measurement<U>,
    {
        match self {
            Self::Finite(inner) => func(inner),
            Self::Infinite => Measurement::<U>::Infinite,
        }
    }

    /// Maps a `Measurement<T>` to `Measurement<U>` by applying
    /// a function to a finite value or returns `Infinite`.
    ///
    /// Examples
    ///
    /// ```
    /// use intervalsets::measure::Measurement;
    ///
    /// let x: Measurement<i32> = Measurement::Finite(10);
    /// assert_eq!(x.map(|v| v as f32 * 2.0), Measurement::Finite(20.0));
    ///
    /// let x: Measurement<i32> = Measurement::Infinite;
    /// assert_eq!(x.map(|v| v * 2), Measurement::Infinite);
    /// ```
    pub fn map<U>(self, func: impl FnOnce(T) -> U) -> Measurement<U> {
        match self {
            Self::Finite(inner) => Measurement::Finite(func(inner)),
            Self::Infinite => Measurement::<U>::Infinite,
        }
    }

    /// Compose with core::ops binary operations
    fn binop_map(self, rhs: Self, func: impl Fn(T, T) -> T) -> Self {
        let lhs = match self {
            Self::Finite(inner) => inner,
            Self::Infinite => return Self::Infinite,
        };

        let rhs = match rhs {
            Self::Finite(inner) => inner,
            Self::Infinite => return Self::Infinite,
        };

        Self::Finite(func(lhs, rhs))
    }

    /// Compose with TryFiniteOp
    #[allow(dead_code)]
    fn binop_try_map(self, rhs: Self, func: impl Fn(&T, &T) -> Option<T>) -> Self {
        let lhs = match self {
            Self::Finite(inner) => inner,
            Self::Infinite => return Self::Infinite,
        };

        let rhs = match rhs {
            Self::Finite(inner) => inner,
            Self::Infinite => return Self::Infinite,
        };

        func(&lhs, &rhs)
            .map(|v| Self::Finite(v))
            .unwrap_or(Self::Infinite)
    }
}

impl<T> core::ops::Add for Measurement<T>
where
    T: Clone + core::ops::Add<T, Output = T>,
{
    type Output = Self;

    /// Add a Measurement with another.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets::measure::Measurement;
    ///
    /// let x = Measurement::Finite(100);
    /// let y = Measurement::Finite(10);
    /// assert_eq!(x + y, Measurement::Finite(110));
    ///
    /// let x = Measurement::Infinite;
    /// assert_eq!(x + y, Measurement::Infinite);
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        self.binop_map(rhs, T::add)
    }
}

impl<T> core::ops::Sub for Measurement<T>
where
    T: Clone + core::ops::Sub<T, Output = T>,
{
    type Output = Self;

    /// Subtract a Measurement from another.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets::measure::Measurement;
    ///
    /// let x = Measurement::Finite(100);
    /// let y = Measurement::Finite(10);
    /// assert_eq!(x - y, Measurement::Finite(90));
    ///
    /// let x = Measurement::Infinite;
    /// assert_eq!(x - y, Measurement::Infinite);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        self.binop_map(rhs, T::sub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_ord() {
        assert_eq!(Measurement::Finite(10) < Measurement::Infinite, true,);
    }
}
