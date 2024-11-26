use core::ops::Sub;

use super::Measurement;
use crate::numeric::{Element, Zero};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Defines the [width measure](https://en.wikipedia.org/wiki/Lebesgue_measure) of a set in R1.
///
/// The width is defined as the absolute difference between the greatest and
/// least elements within the interval set. If one or more sides is unbounded
/// then the width is infinite.
///
/// > Mathematically speaking, the width of any Countable set is 0.
/// > We *do* allow calculating the width over the Reals between two integer bounds,
/// > however unexpected results may occur due to discrete normalization.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
///
///
/// let interval = EnumInterval::closed(10.0, 100.0);
/// assert_eq!(interval.width().finite(), 90.0);
/// ```
///
/// # Normalization problem
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let a = EnumInterval::closed(0.0, 10.0);
/// assert_eq!(a.width().finite(), 10.0);
///
/// let b = EnumInterval::open(0, 10);
/// assert_eq!(b.width().finite(), 8);
/// ```
pub trait Width {
    #[allow(missing_docs)]
    type Output;

    #[allow(missing_docs)]
    fn width(&self) -> Measurement<Self::Output>;
}

impl<T, Out> Width for FiniteInterval<T>
where
    Out: Zero,
    T: Element,
    for<'a> &'a T: Sub<Output = Out>, //T: Element + Clone + Sub<T, Output = Out>,
{
    type Output = Out;

    fn width(&self) -> Measurement<Self::Output> {
        match self.view_raw() {
            None => Measurement::Finite(Out::zero()),
            Some((left, right)) => Measurement::Finite(right.value() - left.value()),
        }
    }
}

impl<T, Out> Width for HalfInterval<T>
where
    Out: Zero,
    T: Element,
    for<'a> &'a T: Sub<Output = Out>,
{
    type Output = Out;

    fn width(&self) -> Measurement<Self::Output> {
        Measurement::Infinite
    }
}

impl<T, Out> Width for EnumInterval<T>
where
    Out: Zero,
    T: Element,
    for<'a> &'a T: Sub<Output = Out>,
{
    type Output = Out;

    fn width(&self) -> crate::measure::Measurement<Self::Output> {
        match self {
            Self::Finite(inner) => inner.width(),
            Self::Half(inner) => inner.width(),
            Self::Unbounded => Measurement::Infinite,
        }
    }
}

/*

#[cfg(test)]
mod tests {
    use approx::relative_eq;

    use super::*;
    use crate::ops::Intersects;
    use crate::Factory;

    #[quickcheck]
    fn check_finite_width(a: f32, b: f32) {
        if f32::is_nan(a) || f32::is_nan(b) || f32::is_infinite(a) || f32::is_infinite(b) {
            return;
        }

        let expected = f32::max(0.0, b - a);
        let open_interval = Interval::open(a, b);
        let closed_interval = Interval::closed(a, b);

        assert_eq!(open_interval.width().finite(), expected);
        assert_eq!(closed_interval.width().finite(), expected);
    }

    #[quickcheck]
    fn check_set_width_float(a: f32, b: f32, c: f32, d: f32) -> bool {
        if a.is_nan() || b.is_nan() || c.is_nan() || d.is_nan() {
            return true;
        }

        let ab = Interval::open(a, b);
        let cd = Interval::open(c, d);

        let expected = f32::max(0.0, b - a) + f32::max(0.0, d - c);
        let x = IntervalSet::new(vec![ab.clone(), cd.clone()]);

        if ab.intersects(&cd) {
            x.width().finite() <= expected
        } else {
            relative_eq!(x.width().finite(), expected)
        }
    }

    #[quickcheck]
    fn check_set_width_integer(a: i32, b: i32, c: i32, d: i32) -> bool {
        if b.checked_sub(a).is_none() || d.checked_sub(c).is_none() {
            return true; // overflow panic
        }

        let ab = i32::max(0, b - a);
        let ab_ivl = Interval::closed(a, b);

        let cd = i32::max(0, d - c);
        let cd_ivl = Interval::closed(c, d);

        if ab.checked_add(cd).is_none() {
            return true; // overflow
        }

        let expected = ab + cd;
        let x = IntervalSet::new(vec![ab_ivl.clone(), cd_ivl.clone()]);

        // subadditivity
        if ab_ivl.intersects(&cd_ivl) {
            x.width().finite() <= expected
        } else {
            x.width().finite() == expected
        }
    }
}
*/
