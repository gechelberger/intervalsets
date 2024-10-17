use crate::bound::Bound;
use crate::detail::{BoundCase, Finite, HalfBounded};
use crate::{Domain, Side};

#[derive(Debug, Clone, PartialEq)] // todo: partialord
pub struct Interval<T>(pub(crate) BoundCase<T>);

impl<T: Domain> Interval<T> {
    /// Returns a new Empty [`Interval`]
    ///
    /// {} = {x | x not in T }
    pub fn empty() -> Self {
        Self(BoundCase::Finite(Finite::Empty))
    }

    /// Returns a new closed finite [`Interval`] or Empty
    ///
    /// [a, b] = { x in T | a <= x <= b }
    pub fn closed(left: T, right: T) -> Self {
        Finite::new(Bound::Closed(left), Bound::Closed(right)).into()
    }

    /// Returns a new open finite [`Interval`] or Empty
    ///
    /// (a, b) = { x in T | a < x < b }
    pub fn open(left: T, right: T) -> Self {
        Finite::new(Bound::Open(left), Bound::Open(right)).into()
    }

    /// Returns a new left open finite [`Interval`] or Empty
    ///
    ///  (a, b] = { x in T | a < x <= b }
    pub fn open_closed(left: T, right: T) -> Self {
        Finite::new(Bound::Open(left), Bound::Closed(right)).into()
    }

    /// Returns a new right open finite [`Interval`] or Empty
    ///
    ///  [a, b) = { x in T | a <= x < b }
    pub fn closed_open(left: T, right: T) -> Self {
        Finite::new(Bound::Closed(left), Bound::Open(right)).into()
    }

    /// Returns a new open, right-unbound [`Interval`]
    ///
    ///  (a, ->) = { x in T | a < x }
    pub fn open_unbound(left: T) -> Self {
        HalfBounded::new(Side::Left, Bound::Open(left)).into()
    }

    /// Returns a new closed, right-unbound [`Interval`]
    ///
    ///  [a, ->) = {x in T | a <= x }
    pub fn closed_unbound(left: T) -> Self {
        HalfBounded::new(Side::Left, Bound::Closed(left)).into()
    }

    /// Returns a new open, left-unbound [`Interval`]
    ///
    /// (a, ->) = { x in T | a < x }
    pub fn unbound_open(right: T) -> Self {
        HalfBounded::new(Side::Right, Bound::Open(right)).into()
    }

    /// Returns a new closed, left-unbound [`Interval`]
    ///
    ///  [a, ->) = { x in T | a <= x }
    pub fn unbound_closed(right: T) -> Self {
        HalfBounded::new(Side::Right, Bound::Closed(right)).into()
    }

    /// Returns a new unbounded [`Interval`]
    ///
    /// (<-, ->) = { x in T }
    pub fn unbounded() -> Self {
        BoundCase::Unbounded.into()
    }

    pub fn new_finite(left: Bound<T>, right: Bound<T>) -> Self {
        Finite::new(left, right).into()
    }

    pub fn new_half_bounded(side: Side, bound: Bound<T>) -> Self {
        HalfBounded::new(side, bound).into()
    }

    pub fn is_fully_bounded(&self) -> bool {
        match &self.0 {
            BoundCase::Finite(inner) => matches!(inner, Finite::FullyBounded(_, _)),
            _ => false,
        }
    }

    pub fn is_half_bounded(&self) -> bool {
        matches!(&self.0, BoundCase::Half(_))
    }

    pub fn is_unbounded(&self) -> bool {
        matches!(&self.0, BoundCase::Unbounded)
    }
}

#[derive(Debug, Clone, PartialEq)] // PartialOrd
pub struct IntervalSet<T> {
    intervals: Vec<Interval<T>>,
}

impl<T> IntervalSet<T> {
    pub fn new(intervals: Vec<Interval<T>>) -> Self {
        todo!()
    }

    pub fn new_unchecked(intervals: Vec<Interval<T>>) -> Self {
        Self { intervals }
    }

    pub fn intervals(&self) -> &Vec<Interval<T>> {
        &self.intervals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_normalization() {
        let interval = Interval::open(0, 10);
        assert_eq!(interval, Interval::closed(1, 9));
    }
}
