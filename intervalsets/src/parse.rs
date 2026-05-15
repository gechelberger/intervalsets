//! `FromStr` for [`Interval`] — delegates to the inner
//! [`EnumInterval`] parser.

use core::str::FromStr;

use intervalsets_core::error::ParseIntervalError;
use intervalsets_core::sets::EnumInterval;

use crate::numeric::Element;
use crate::Interval;

/// Parses an [`Interval`] from its `Display` form. See
/// [`EnumInterval`'s `FromStr` impl](intervalsets_core::sets::EnumInterval)
/// for the full grammar.
///
/// # Examples
///
/// ```
/// use intervalsets::prelude::*;
///
/// let x: Interval<i32> = "[0, 10]".parse().unwrap();
/// assert_eq!(x, Interval::closed(0, 10));
///
/// let x: Interval<f64> = "(0.0, 10.0]".parse().unwrap();
/// assert_eq!(x, Interval::open_closed(0.0, 10.0));
///
/// let x: Interval<i32> = "(.., 10)".parse().unwrap();
/// assert_eq!(x, Interval::unbound_open(10));
///
/// let x: Interval<i32> = "{}".parse().unwrap();
/// assert_eq!(x, Interval::empty());
///
/// // Round-trip with Display.
/// let x = Interval::open_closed(1.5, 7.5);
/// assert_eq!(format!("{x}").parse::<Interval<f64>>().unwrap(), x);
/// ```
impl<T> FromStr for Interval<T>
where
    T: Element + FromStr,
{
    type Err = ParseIntervalError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EnumInterval::<T>::from_str(s).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};

    #[test]
    fn round_trip_each_form() {
        let cases: [Interval<f64>; 9] = [
            Interval::closed(0.0, 10.0),
            Interval::open(0.0, 10.0),
            Interval::closed_open(0.0, 10.0),
            Interval::open_closed(0.0, 10.0),
            Interval::closed_unbound(0.0),
            Interval::open_unbound(0.0),
            Interval::unbound_closed(10.0),
            Interval::unbound_open(10.0),
            Interval::unbounded(),
        ];
        for x in cases {
            let printed = format!("{x}");
            let parsed: Interval<f64> = printed.parse().unwrap();
            assert_eq!(parsed, x, "round-trip failed for {printed}");
        }
    }

    #[test]
    fn empty_round_trip() {
        let x = Interval::<i32>::empty();
        let parsed: Interval<i32> = format!("{x}").parse().unwrap();
        assert_eq!(parsed, x);
    }
}
