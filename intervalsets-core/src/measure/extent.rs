use core::fmt;

use crate::ops::math::TryAdd;

/// The result of applying a Measure to a `Set`.
///
/// `Extent<T>` is structurally `Option<T>` with domain-meaningful
/// variant names: `Finite(t)` ≡ `Some(t)` and `Infinite` ≡ `None`.
/// Bidirectional [`From`] impls connect the two so callers who want
/// the full `Option` combinator surface (`map`, `and_then`,
/// `unwrap_or`, ...) round-trip through it:
///
/// ```
/// use intervalsets_core::measure::Extent;
///
/// let e: Extent<u32> = Extent::Finite(10);
/// let opt: Option<u32> = e.into();
/// let doubled: Extent<u32> = opt.map(|x| x * 2).into();
/// assert_eq!(doubled, Extent::Finite(20));
/// ```
///
/// Note: `Option::from(e)` is ambiguous with std's blanket
/// `impl<T> From<T> for Option<T>` (which would wrap the whole
/// `Extent` into `Some(extent)`). Use `.into()` with an explicit
/// `Option<T>` target type, or `<Option<T>>::from(e)`.
///
/// # Three-state honesty
///
/// `Extent` is the value half of a three-state contract used by
/// fallible measures:
///
/// - `Ok(Finite(v))` — bounded set, arithmetic succeeded
/// - `Ok(Infinite)`  — structurally unbounded (half-bounded / unbounded set)
/// - `Err(_)`        — bounded set whose measure overflowed the
///   representation
///
/// `Extent::Infinite` is **never** used as a fallback for arithmetic
/// overflow — that path surfaces as `Err`. Users who want
/// saturate-to-`Infinite` semantics either pick the
/// [`core::num::Saturating`] storage type or wrap manually.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Extent<T> {
    #[allow(missing_docs)]
    Finite(T),
    #[allow(missing_docs)]
    Infinite,
}

impl<T> Extent<T> {
    /// Returns `true` if the extent is an `Infinite` value.
    pub fn is_infinite(&self) -> bool {
        matches!(self, Self::Infinite)
    }

    /// Returns `true` if the extent is a `Finite` value.
    pub fn is_finite(&self) -> bool {
        matches!(self, Self::Finite(_))
    }

    /// Returns the contained finite value, consuming `self`.
    ///
    /// Named `finite` (not `unwrap`) because the domain semantic
    /// "give me the finite value, panic if it's infinite" is more
    /// informative than the generic "expose the underlying T."
    /// Callers who want a custom panic message use
    /// `Option::from(x).expect(msg)`.
    ///
    /// # Panics
    ///
    /// Panics if `self` is `Infinite`.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets_core::measure::Extent;
    ///
    /// assert_eq!(Extent::Finite(7).finite(), 7);
    /// ```
    ///
    /// ```should_panic
    /// use intervalsets_core::measure::Extent;
    ///
    /// let _ = Extent::<i32>::Infinite.finite();
    /// ```
    pub fn finite(self) -> T {
        match self {
            Self::Finite(t) => t,
            Self::Infinite => panic!("Extent::finite called on Infinite"),
        }
    }

    /// Compose with a fallible binary operation. `Infinite`
    /// short-circuits to `Infinite`; `Finite + Finite` runs the
    /// closure and propagates any error.
    ///
    /// Load-bearing for the wrapper crate's `IntervalSet::try_count`
    /// and `IntervalSet::try_width` summation folds.
    pub fn try_binop_map<E>(
        self,
        rhs: Self,
        func: impl FnOnce(T, T) -> Result<T, E>,
    ) -> Result<Self, E> {
        match (self, rhs) {
            (Self::Finite(a), Self::Finite(b)) => func(a, b).map(Self::Finite),
            _ => Ok(Self::Infinite),
        }
    }
}

impl<T> From<Extent<T>> for Option<T> {
    /// `Finite(t)` → `Some(t)`, `Infinite` → `None`.
    fn from(extent: Extent<T>) -> Self {
        match extent {
            Extent::Finite(t) => Some(t),
            Extent::Infinite => None,
        }
    }
}

impl<T> From<Option<T>> for Extent<T> {
    /// `Some(t)` → `Finite(t)`, `None` → `Infinite`.
    ///
    /// The mapping `None ≡ Infinite` is canonical by fiat — callers
    /// constructing an `Extent` from an `Option` must accept this
    /// interpretation.
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(t) => Self::Finite(t),
            None => Self::Infinite,
        }
    }
}

impl<T> core::ops::Add for Extent<T>
where
    T: core::ops::Add<T, Output = T>,
{
    type Output = Self;

    /// Add two `Extent`s. `Infinite + _` and `_ + Infinite` both yield
    /// `Infinite`.
    ///
    /// For float `T`, the result may be non-finite if `T::add` overflows
    /// to ±INF or produces NaN. Callers who want overflow-awareness
    /// should reach for [`TryAdd`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets_core::measure::Extent;
    ///
    /// let x = Extent::Finite(100);
    /// let y = Extent::Finite(10);
    /// assert_eq!(x + y, Extent::Finite(110));
    ///
    /// let x = Extent::Infinite;
    /// assert_eq!(x + y, Extent::Infinite);
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Finite(a), Self::Finite(b)) => Self::Finite(a + b),
            _ => Self::Infinite,
        }
    }
}

impl<T> TryAdd for Extent<T>
where
    T: TryAdd<T, Output = T>,
{
    type Output = Self;
    type Error = T::Error;

    /// Fallible addition. `Infinite + _` and `_ + Infinite` both yield
    /// `Ok(Infinite)`; `Finite + Finite` delegates to `T::try_add` and
    /// propagates any error.
    ///
    /// Overflow is **not** collapsed into `Ok(Infinite)` — the three-
    /// state contract (`Ok(Finite)`, `Ok(Infinite)`, `Err`) depends on
    /// keeping these distinct. Saturate-on-overflow semantics belong
    /// to the [`core::num::Saturating`] storage type, not to `TryAdd`.
    ///
    /// # Examples
    ///
    /// ```
    /// use intervalsets_core::measure::Extent;
    /// use intervalsets_core::ops::math::TryAdd;
    ///
    /// let x: Extent<u32> = Extent::Finite(100);
    /// let y: Extent<u32> = Extent::Finite(10);
    /// assert_eq!(x.try_add(y).unwrap(), Extent::Finite(110));
    ///
    /// // Overflow surfaces as Err — not Ok(Infinite).
    /// let big: Extent<u8> = Extent::Finite(200);
    /// assert!(big.try_add(big).is_err());
    /// ```
    fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.try_binop_map(rhs, T::try_add)
    }
}

impl<T: fmt::Display> fmt::Display for Extent<T> {
    /// `Finite(t)` delegates to `T`'s [`Display`](fmt::Display);
    /// `Infinite` prints as `"∞"`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Finite(t) => write!(f, "{}", t),
            Self::Infinite => write!(f, "∞"),
        }
    }
}

// `Sub` / `Mul` / `Div` / `Neg` and their `Try*` siblings are
// intentionally not implemented. `Finite(n) - Infinite` should be
// `-Infinite` but we have no `-Infinite` variant; `Infinite - Infinite`
// is undefined; `0 * Infinite` is undefined; measures are non-negative.
// Callers needing a non-`Add` binop go through `try_binop_map` with a
// caller-supplied closure or round-trip through `Option::from(extent)`
// and compose with `Option::zip` / `Option::map`.

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn ord_finite_less_than_infinite() {
        assert!(Extent::Finite(10) < Extent::Infinite);
    }

    #[test]
    fn from_extent_to_option() {
        assert_eq!(Option::from(Extent::Finite(7)), Some(7));
        assert_eq!(Option::<i32>::from(Extent::Infinite), None);
    }

    #[test]
    fn from_option_to_extent() {
        assert_eq!(Extent::from(Some(7)), Extent::Finite(7));
        assert_eq!(Extent::<i32>::from(None), Extent::Infinite);
    }

    #[test]
    fn option_roundtrip_preserves_value() {
        let e = Extent::Finite(42_u32);
        let back: Extent<u32> = Option::from(e).into();
        assert_eq!(back, Extent::Finite(42));
    }

    #[test]
    fn add_infinite_absorbs() {
        assert_eq!(Extent::Finite(1) + Extent::Infinite, Extent::Infinite);
        assert_eq!(Extent::Infinite + Extent::Finite(1), Extent::Infinite);
        assert_eq!(Extent::<i32>::Infinite + Extent::Infinite, Extent::Infinite);
    }

    #[test]
    fn try_add_infinite_short_circuits_to_ok_infinite() {
        let a: Extent<u8> = Extent::Finite(200);
        assert_eq!(a.try_add(Extent::Infinite), Ok(Extent::Infinite));
        assert_eq!(Extent::<u8>::Infinite.try_add(a), Ok(Extent::Infinite));
        assert_eq!(
            Extent::<u8>::Infinite.try_add(Extent::Infinite),
            Ok(Extent::Infinite)
        );
    }

    #[test]
    fn try_add_overflow_propagates_err_not_ok_infinite() {
        // The crate's u8 TryAdd surfaces overflow as Err(MathError::Range).
        // Critical invariant: this does NOT collapse to Ok(Infinite).
        let a: Extent<u8> = Extent::Finite(200);
        let b: Extent<u8> = Extent::Finite(200);
        let result = a.try_add(b);
        assert!(
            result.is_err(),
            "u8 200+200 must surface as Err, not Ok(Infinite)"
        );
    }

    #[test]
    fn display_finite_delegates_to_inner() {
        assert_eq!(std::format!("{}", Extent::Finite(42)), "42");
        assert_eq!(std::format!("{}", Extent::Finite(3.5_f64)), "3.5");
    }

    #[test]
    fn display_infinite_prints_math_symbol() {
        assert_eq!(std::format!("{}", Extent::<i32>::Infinite), "∞");
    }
}
