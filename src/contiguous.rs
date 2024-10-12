use crate::{intersects::Intersects, ival::IVal, FiniteInterval, HalfInterval, Interval};

/// Union for two intervals that are not disjoint
pub trait Contiguous<Rhs = Self> {
    type Output;

    fn contiguous(&self, rhs: &Rhs) -> Option<Self::Output>;
}

impl<T: Copy + PartialOrd> Contiguous<Self> for FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    fn contiguous(&self, rhs: &Self) -> Option<Self::Output> {
        if self.is_disjoint_from(rhs) {
            return None;
        }

        self.map(|a_left, a_right| {
            rhs.map_bounds(|b_left, b_right| {
                FiniteInterval::NonZero(
                    IVal::min(a_left, b_left),
                    IVal::max(a_right, b_right),
                )
            })
        })
    }
}

impl<T: Copy + PartialOrd> Contiguous<Self> for HalfInterval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &Self) -> Option<Self::Output> {
        todo!()
    }
}


impl<T: Copy + PartialOrd> Contiguous<Self> for Interval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &Self) -> Option<Self::Output> {
        todo!()
    }
}