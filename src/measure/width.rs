use crate::{Domain, Interval, IntervalSet};

use super::Measurement;

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
/// use intervalsets::{Interval, IntervalSet, Union};
/// use intervalsets::measure::Width;
///
/// let interval = Interval::closed(10.0, 100.0);
/// assert_eq!(interval.width().finite(), 90.0);
///
/// let interval = Interval::open(10.0, 100.0);
/// assert_eq!(interval.width().finite(), 90.0);
///
/// let interval = Interval::closed(0, 10);
/// assert_eq!(interval.width().finite(), 10);
///
/// let set = Interval::closed(0.0, 10.0)
///     .union(&Interval::closed(5.0, 15.0))
///     .union(&Interval::open(100.0, 110.0));
/// assert_eq!(set.width().finite(), 25.0);
/// ```
/// 
/// ## Normalization problem
/// ```
/// use intervalsets::{Interval, Difference};
/// use intervalsets::measure::Width;
/// 
/// let a = Interval::closed(0.0, 10.0);
/// let a = a.difference(&Interval::closed(5.0, 15.0));
/// assert_eq!(a.width().finite(), 5.0);
/// 
/// let b = Interval::closed(0, 10);
/// let b = b.difference(&Interval::closed(5, 15));
/// assert_eq!(b.width().finite(), 4);
/// ```
pub trait Width {
    type Output;

    fn width(&self) -> Measurement<Self::Output>;
}

impl<T, Out> Width for Interval<T>
where
    T: Domain + core::ops::Sub<T, Output = Out>,
    Out: num_traits::Zero,
{
    type Output = Out;

    fn width(&self) -> Measurement<Self::Output> {
        self.0.width()
    }
}

impl<T, Out> Width for IntervalSet<T>
where
    T: Domain + core::ops::Sub<T, Output = Out>,
    Out: num_traits::Zero + core::ops::Add<Out, Output = Out> + Clone,
{
    type Output = Out;

    fn width(&self) -> Measurement<Self::Output> {
        self.intervals()
            .iter()
            .map(|subset| subset.width())
            .fold(Measurement::Finite(Out::zero()), |accum, item| accum + item)
    }
}

#[cfg(test)]
mod tests {
    use crate::Intersects;

    use super::*;

    use approx::relative_eq;

    #[quickcheck]
    fn check_finite_width(a: f32, b: f32) {
        if a.is_nan() || b.is_nan() {
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
