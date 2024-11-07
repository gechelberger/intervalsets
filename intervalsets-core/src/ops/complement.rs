use super::{Complement, Intersection};
use crate::bound::Side;
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, StackSet};
use crate::Factory;

impl<T: Domain + Ord> Complement for FiniteInterval<T> {
    type Output = StackSet<T>;

    fn complement(self) -> Self::Output {
        match self {
            Self::Empty => EnumInterval::Unbounded.into(),
            Self::Bounded(lhs, rhs) => unsafe {
                // SAFETY: ...
                StackSet::new_unchecked([
                    EnumInterval::half_bounded(Side::Right, lhs.flip()),
                    EnumInterval::half_bounded(Side::Left, rhs.flip()),
                ])
            },
        }
    }
}

impl<T> Complement for HalfInterval<T> {
    type Output = HalfInterval<T>;

    fn complement(self) -> Self::Output {
        Self::new(self.side.flip(), self.bound.flip())
    }
}

impl<T: Domain + Ord> Complement for EnumInterval<T> {
    type Output = StackSet<T>;

    fn complement(self) -> Self::Output {
        match self {
            Self::Finite(interval) => interval.complement(),
            Self::Half(interval) => interval.complement().into(),
            Self::Unbounded => FiniteInterval::Empty.into(),
        }
    }
}

impl<T: Domain + Ord + Clone> Complement for StackSet<T> {
    type Output = Self;

    fn complement(self) -> Self::Output {
        let intervals = self.into_raw();

        intervals
            .into_iter()
            .map(|x| x.complement())
            .fold(EnumInterval::Unbounded.into(), Intersection::intersection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_complements() -> Result<(), Error> {
        let x = EnumInterval::closed(0, 10);
        let c = x.complement();
        let xx = c.complement().expect_interval()?;
        assert_eq!(x, xx);

        Ok(())
    }
}
