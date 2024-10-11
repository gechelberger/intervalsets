use crate::{contains::Contains, ival::Side, FiniteInterval, HalfInterval, Interval};

/// Intersects is commutative 
pub trait Intersects<Rhs = Self> {
    fn intersects(&self, rhs: &Rhs) -> bool;
}


impl<T: Copy + PartialOrd> Intersects<Self> for FiniteInterval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.map_or::<bool>(false, |l1, r1| {
            rhs.map_or::<bool>(false, |l2, r2| {
                return todo!(); // i think this is wrong
                // TODO: convince myself that the boundary conditions are being handled properly
                // l1.contains(Side::Left, &r2.value) || l2.contains(Side::Left, &r1.value)
            })
        })
    }
}

impl<T: Copy + PartialOrd> Intersects<HalfInterval<T>> for FiniteInterval<T> {
    fn intersects(&self, rhs: &HalfInterval<T>) -> bool {
        rhs.intersects(self)
    }
}

impl<T: Copy + PartialOrd> Intersects<Interval<T>> for FiniteInterval<T> {
    fn intersects(&self, rhs: &Interval<T>) -> bool {
        rhs.intersects(self)
    }
}

impl<T: Copy + PartialOrd> Intersects<FiniteInterval<T>> for HalfInterval<T> {
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        rhs.map_or(false, |left, right| {
            self.contains(&left.value) || self.contains(&right.value)
        })
    }
}

impl<T: Copy + PartialOrd> Intersects<Self> for HalfInterval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        let lhs = self;
        lhs.contains(&rhs.ival.value) || rhs.contains(&lhs.ival.value)
    }
}

impl<T: Copy + PartialOrd> Intersects<Interval<T>> for HalfInterval<T> {
    fn intersects(&self, rhs: &Interval<T>) -> bool {
        rhs.intersects(self)
    }
}

impl<T: Copy + PartialOrd> Intersects<FiniteInterval<T>> for Interval<T> {
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Infinite => *rhs != FiniteInterval::Empty,
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Finite(lhs) => lhs.intersects(rhs),
        }
    }
}

impl<T: Copy + PartialOrd> Intersects<HalfInterval<T>> for Interval<T> {
    fn intersects(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Infinite => true,
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Finite(lhs) => lhs.intersects(rhs),
        }
    }
}

impl<T: Copy + PartialOrd> Intersects<Self> for Interval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        match self {
            Self::Infinite => *rhs != FiniteInterval::Empty.into(),
            Self::Half(lhs) => rhs.intersects(lhs),
            Self::Finite(lhs) => rhs.intersects(lhs),
        }
    }
}