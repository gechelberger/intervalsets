use intervalsets_core::EnumInterval;
use quickcheck::{Arbitrary, Gen};

use crate::numeric::{Element, Zero};
use crate::ops::{Intersects, Union};
use crate::{Interval, IntervalSet};

impl<T: Element + Clone + Arbitrary + Zero> Arbitrary for Interval<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(EnumInterval::arbitrary(g))
    }
}

impl<T: Element + Clone + Arbitrary + Zero> Arbitrary for IntervalSet<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        let n = *g
            .choose(&[
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            ])
            .unwrap();

        let mut iset = IntervalSet::<T>::empty();
        for _ in 0..n {
            let interval = Interval::<T>::arbitrary(g);
            if iset.is_disjoint_from(&interval) {
                // otherwise the result will practically always be (<-, ->)
                iset = iset.union(interval);
            }
        }

        iset
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use super::*;
    use crate::factory::traits::*;
    use crate::ops::{Complement, Difference, Union};

    #[quickcheck]
    fn check_qc_interval(interval: Interval<f32>) {
        assert_eq!(
            interval.complement().complement().expect_interval(),
            interval
        );
    }

    #[quickcheck]
    fn check_qc_two_intervals(a: Interval<f32>, b: Interval<f32>) {
        assert_eq!(
            a.union(b).complement(),
            Interval::unbounded().difference(a).difference(b)
        );
    }

    #[quickcheck]
    fn check_qc_interval_set(iset: IntervalSet<f32>) {
        assert_eq!(iset.clone().complement().complement(), iset);
    }

    #[quickcheck]
    fn check_interval_set_invariants(iset: IntervalSet<f32>) {
        assert!(IntervalSet::satisfies_invariants(iset.slice()));
    }
}
