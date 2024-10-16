use crate::{Bound, Contains, Domain, Intersection, Side};

use super::{Finite, HalfBounded};

impl<T: Domain> Intersection<Self> for Finite<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        self.map(|a_left, a_right| {
            rhs.map(|b_left, b_right| {
                // new() will clean up empty sets where left & right have violated bounds
                Self::new(
                    Bound::max_left(a_left, b_left),
                    Bound::min_right(a_right, b_right),
                )
            })
        })
    }
}

impl<T: Domain> Intersection<HalfBounded<T>> for Finite<T> {
    type Output = Finite<T>;

    fn intersection(&self, rhs: &HalfBounded<T>) -> Self::Output {
        self.map(|left, right| {
            let n_seen = [left, right]
                .into_iter()
                .filter(|limit| rhs.contains(limit.value()))
                .count();

            if n_seen == 2 {
                Self::new(left.clone(), right.clone())
            } else if n_seen == 1 {
                match rhs.side {
                    Side::Left => Self::new(rhs.bound.clone(), right.clone()),
                    Side::Right => Self::new(left.clone(), rhs.bound.clone()),
                }
            } else {
                Self::Empty
            }
        })
    }
}
