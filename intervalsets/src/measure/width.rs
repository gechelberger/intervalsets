use intervalsets_core::ops::math::TryAdd;

/// Defines the [width measure](https://en.wikipedia.org/wiki/Lebesgue_measure) of a set in R1.
///
/// The width is defined as the absolute difference between the
/// greatest and least elements within the interval set. If one or
/// more sides is unbounded, the width is infinite.
///
/// > Mathematically speaking, the width of any Countable set is 0.
/// > We *do* allow calculating the width over the Reals between two
/// > integer bounds, however unexpected results may occur due to
/// > discrete normalization. For discrete `T`, prefer
/// > [`Count`](crate::measure::Count).
///
/// # Example
/// ```
/// use intervalsets::prelude::*;
///
/// let interval = Interval::closed(10.0, 100.0);
/// assert_eq!(interval.width().finite(), 90.0);
///
/// let interval = Interval::open(10.0, 100.0);
/// assert_eq!(interval.width().finite(), 90.0);
///
/// let interval = Interval::closed(0, 10);
/// assert_eq!(interval.width().finite(), 10u128);
///
/// let set = Interval::closed(0.0, 10.0)
///     .union(Interval::closed(5.0, 15.0))
///     .union(Interval::open(100.0, 110.0));
/// assert_eq!(set.width().finite(), 25.0);
/// ```
///
/// # Normalization problem
///
/// ```
/// use intervalsets::prelude::*;
///
/// let a = Interval::closed(0.0, 10.0);
/// let a = a.difference(Interval::closed(5.0, 15.0));
/// assert_eq!(a.width().finite(), 5.0);
///
/// let b = Interval::closed(0, 10);
/// let b = b.difference(Interval::closed(5, 15));
/// assert_eq!(b.width().finite(), 4u128);
/// ```
use super::{Extent, Width, Widthable};
use crate::error::MathError;
use crate::numeric::Zero;
use crate::{Interval, IntervalSet};

impl<T> Width for Interval<T>
where
    T: Widthable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = MathError;

    fn try_width(&self) -> Result<Extent<Self::Output>, Self::Error> {
        self.0.try_width()
    }
}

impl<T, Out> Width for IntervalSet<T>
where
    T: Widthable<Output = Out>,
    Out: Zero + TryAdd<Out, Output = Out>,
    <Out as TryAdd>::Error: Into<MathError>,
{
    type Output = Out;
    type Error = MathError;

    /// Sum per-component widths via [`TryAdd`] so a summation that
    /// exceeds `Out`'s representable range surfaces as [`MathError`]
    /// rather than panicking in debug / wrapping in release.
    fn try_width(&self) -> Result<Extent<Self::Output>, Self::Error> {
        self.iter()
            .try_fold(Extent::Finite(<Out as Zero>::zero()), |accum, subset| {
                accum.try_binop_map(subset.try_width()?, |a, b| a.try_add(b).map_err(Into::into))
            })
    }
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;

    use super::*;
    use crate::factory::FiniteFactory;
    use crate::ops::Intersects;

    #[quickcheck]
    fn check_finite_width(a: f32, b: f32) {
        if f32::is_nan(a) || f32::is_nan(b) || f32::is_infinite(a) || f32::is_infinite(b) {
            return;
        }
        // open(a, a) panics under strict semantics; require a < b.
        if a >= b {
            return;
        }

        let expected = b - a;
        // Filter cases where the diff itself overflows to ±INF; those
        // surface as Err on try_width and panic on width().
        if !expected.is_finite() {
            return;
        }
        let open_interval = Interval::open(a, b);
        let closed_interval = Interval::closed(a, b);

        assert_eq!(open_interval.width().finite(), expected);
        assert_eq!(closed_interval.width().finite(), expected);
    }

    #[quickcheck]
    fn check_set_width_float(a: f32, b: f32, c: f32, d: f32) -> bool {
        if !a.is_finite() || !b.is_finite() || !c.is_finite() || !d.is_finite() {
            return true;
        }
        if a >= b || c >= d {
            return true;
        }

        let ab = Interval::open(a, b);
        let cd = Interval::open(c, d);

        let expected = (b - a) + (d - c);
        if !expected.is_finite() {
            return true;
        }
        let x = IntervalSet::new(vec![ab, cd]);

        if ab.intersects(&cd) {
            x.width().finite() <= expected
        } else {
            relative_eq!(x.width().finite(), expected)
        }
    }

    #[quickcheck]
    fn check_set_width_integer(a: i32, b: i32, c: i32, d: i32) -> bool {
        if a > b || c > d {
            return true;
        }

        let ab = (b as i64 - a as i64) as u128;
        let ab_ivl = Interval::closed(a, b);

        let cd = (d as i64 - c as i64) as u128;
        let cd_ivl = Interval::closed(c, d);

        let expected = ab + cd;
        let x = IntervalSet::new(vec![ab_ivl, cd_ivl]);

        // subadditivity
        if ab_ivl.intersects(&cd_ivl) {
            x.width().finite() <= expected
        } else {
            x.width().finite() == expected
        }
    }
}
