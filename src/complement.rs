use crate::infinite::IntervalSet;
use crate::intersection::Intersection;
use crate::ival::Side;
use crate::{FiniteInterval, HalfInterval, Interval};

pub trait Complement {
    type Output;

    fn complement(&self) -> Self::Output;
}

impl<T: Copy> Complement for FiniteInterval<T> {
    type Output = IntervalSet<T>;

    fn complement(&self) -> Self::Output {
        match self {
            Self::Empty => Interval::Infinite.into(),
            Self::NonZero(left, right) => {
                let intervals: Vec<Interval<T>> = vec![
                    HalfInterval::new(Side::Right, left.flip()).into(),
                    HalfInterval::new(Side::Left, right.flip()).into(),
                ];
                IntervalSet { intervals }
            }
        }
    }
}

impl<T: Copy> Complement for HalfInterval<T> {
    type Output = HalfInterval<T>;

    fn complement(&self) -> Self::Output {
        Self::new(self.side.flip(), self.ival.flip())
    }
}

impl<T: Copy> Complement for Interval<T> {
    type Output = IntervalSet<T>;

    fn complement(&self) -> Self::Output {
        match self {
            Self::Infinite => FiniteInterval::Empty.into(),
            Self::Half(interval) => interval.complement().into(),
            Self::Finite(interval) => interval.complement(),
        }
    }
}

impl<T: Copy + PartialOrd> Complement for IntervalSet<T> {
    type Output = IntervalSet<T>;

    /// DeMorgan's Law:
    /// (A U B U C)^c = (A^c ∩ B^c ∩ C^c)
    fn complement(&self) -> Self::Output {
        naive_set_complement(&self.intervals)
    }
}

fn naive_set_complement<T>(intervals: &Vec<Interval<T>>) -> IntervalSet<T>
where
    T: Copy + PartialOrd,
{
    intervals
        .iter()
        .map(|x| x.complement())
        .fold(Interval::Infinite.into(), |l, r| l.intersection(&r))
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::contains::Contains;

    #[quickcheck]
    fn test_finite_complement_i8(a: i8) {
        let baseline = Interval::open_closed(0, 50);
        let complement = baseline.complement();

        assert!(baseline.contains(&a) != complement.contains(&a))
    }

    #[quickcheck]
    fn test_finite_complement_f32(a: f32) {
        if f32::is_nan(a) {
            return;
        }

        let baseline = Interval::open_closed(0 as f32, 50.0);
        let complement = baseline.complement();
        assert!(baseline.contains(&a) != complement.contains(&a))
    }

    #[quickcheck]
    fn test_half_complement_i8(a: i8) {
        let baseline = Interval::unbound_closed(50 as i8);
        let complement = baseline.complement();

        assert!(baseline.contains(&a) != complement.contains(&a));
    }
}
