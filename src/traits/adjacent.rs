use crate::numeric::Domain;
use crate::Interval;

/// Defines whether two sets are contiguous.
///
/// Given two Sets A and B which are both
/// Subsets of T:
///
/// > A and B are adjacent if their extrema
/// > have no elements in T between them.
///
/// # Example
///
/// > [1, 5] is adjacent to [6, 10]
///
/// > [1.0, 5.0] is not adjacent to [6.0, 10.0]
///
pub trait Adjacent<Rhs = Self> {
    fn is_adjacent_to(&self, rhs: &Rhs) -> bool;
}

impl<T: Domain> Adjacent<Self> for Interval<T> {
    fn is_adjacent_to(&self, rhs: &Self) -> bool {
        self.0.is_adjacent_to(&rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_adjacent_to() {
        assert_eq!(
            Interval::closed(0, 10).is_adjacent_to(&Interval::closed(10, 20)),
            false
        );

        assert_eq!(
            Interval::closed(0, 10).is_adjacent_to(&Interval::closed(11, 20)),
            true
        );

        assert_eq!(
            Interval::closed(0.0, 10.0).is_adjacent_to(&Interval::closed(10.0, 20.0)),
            true
        );

        assert_eq!(
            Interval::open(0.0, 10.0).is_adjacent_to(&Interval::open(10.0, 20.0)),
            false,
        );
    }
}
