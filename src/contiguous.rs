use crate::{contains::Contains, intersects::Intersects, ival::{IVal, Side}, FiniteInterval, HalfInterval, Interval};

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
        if self.side == rhs.side {
            if self.contains(&rhs.ival.value) {
                Some(self.clone().into())
            } else {
                Some(rhs.clone().into())
            }
        } else {

            // unfortunately we have to check from both sides to catch the
            // case where left and right values are the same but open & closed
            if self.contains(&rhs.ival.value) && rhs.contains(&self.ival.value) {
                Some(Interval::Infinite)
            } else {
                None // disjoint
            }
        }
    }
}

impl<T: Copy + PartialOrd> Contiguous<FiniteInterval<T>> for HalfInterval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        match rhs {
            FiniteInterval::Empty => Some(self.clone().into()),
            FiniteInterval::NonZero(left, right) => {
                let n_seen = [left, right].into_iter()
                    .map(|ival| self.contains(&ival.value))
                    .count();

                match n_seen {
                    2 => Some(self.clone().into()),
                    1 => match self.side {
                        Side::Left => Some(HalfInterval::new(self.side, left.clone()).into()),
                        Side::Right => Some(HalfInterval::new(self.side, right.clone()).into())
                    },
                    _ => None, // disjoint
                }
                
            }
        }
    }
}

impl<T: Copy + PartialOrd> Contiguous<HalfInterval<T>> for FiniteInterval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &HalfInterval<T>) -> Option<Self::Output> {
        rhs.contiguous(self)
    }
}

////////////////// 

impl<T: Copy + PartialOrd> Contiguous<FiniteInterval<T>> for Interval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Infinite => Some(Self::Infinite),
            Self::Half(lhs) => lhs.contiguous(rhs),
            Self::Finite(lhs) => {
                lhs.contiguous(rhs)
                    .map(|itv| itv.into())
            },
        }
    }
}

impl<T: Copy + PartialOrd> Contiguous<HalfInterval<T>> for Interval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &HalfInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Infinite => Some(Self::Infinite),
            Self::Half(lhs) => lhs.contiguous(rhs),
            Self::Finite(lhs) => rhs.contiguous(lhs),
        }
    }
}

impl<T: Copy + PartialOrd> Contiguous<Self> for Interval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &Self) -> Option<Self::Output> {
        match self {
            Self::Infinite => Some(Self::Infinite),
            Self::Half(lhs) => rhs.contiguous(lhs),
            Self::Finite(lhs) => rhs.contiguous(lhs),
        }
    }
}

impl<T: Copy + PartialOrd> Contiguous<Interval<T>> for FiniteInterval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &Interval<T>) -> Option<Self::Output> {
        rhs.contiguous(self)
    }
}

impl<T: Copy + PartialOrd> Contiguous<Interval<T>> for HalfInterval<T> {
    type Output = Interval<T>;

    fn contiguous(&self, rhs: &Interval<T>) -> Option<Self::Output> {
        rhs.contiguous(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_finite_contiguous() {
        assert_eq!(
            Interval::open(0, 100).contiguous(&Interval::open(50, 150)),
            Some(Interval::open(0, 150))
        );

        assert_eq!(
            Interval::open(0, 100).contiguous(&Interval::open(100, 200)),
            None,
        );

        assert_eq!(
            Interval::closed(0, 100).contiguous(&Interval::closed(100, 200)),
            Some(Interval::closed(0, 200))
        );
    }
}