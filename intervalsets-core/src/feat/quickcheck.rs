use quickcheck::{Arbitrary, Gen};

use crate::bound::BoundType::{self, *};
use crate::bound::FiniteBound;
use crate::bound::Side::{self, *};
use crate::numeric::{Element, Zero};
use crate::{EnumInterval, FiniteInterval, HalfInterval};

const fn first_n_i32s<const N: usize>() -> [i32; N] {
    let mut res = [0i32; N];
    let mut i = 0;
    while i < N {
        res[i] = i as i32;
        i += 1;
    }
    res
}

static CHANCES_100: [i32; 100] = first_n_i32s();

impl Arbitrary for Side {
    fn arbitrary(g: &mut Gen) -> Self {
        *g.choose(&[Left, Right]).unwrap()
    }
}

impl Arbitrary for BoundType {
    fn arbitrary(g: &mut Gen) -> Self {
        *g.choose(&[Closed, Open]).unwrap()
    }
}

impl<T: Clone + Arbitrary> Arbitrary for FiniteBound<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        FiniteBound::new(BoundType::arbitrary(g), T::arbitrary(g))
    }
}

impl<T: Element + Clone + Arbitrary> Arbitrary for FiniteInterval<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.choose(&CHANCES_100).unwrap() {
            // empty 3% of the time
            &0 | &1 | &2 => Self::empty(),
            _ => {
                let a = FiniteBound::<T>::arbitrary(g);
                let b = FiniteBound::<T>::arbitrary(g);

                let (left, right) = if a.ord(Left) < b.ord(Left) {
                    (a, b)
                } else {
                    (b, a)
                };

                match FiniteInterval::new_strict(left, right) {
                    Ok(interval) => interval,
                    Err(_) => Self::arbitrary(g),
                }
            }
        }
    }
}

impl<T: Element + Clone + Arbitrary + Zero> Arbitrary for HalfInterval<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        let side = Side::arbitrary(g);
        let bound = FiniteBound::<T>::arbitrary(g);

        match HalfInterval::new_strict(side, bound) {
            Ok(interval) => interval,
            Err(_) => Self::arbitrary(g),
        }
    }
}

impl<T: Element + Clone + Arbitrary + Zero> Arbitrary for EnumInterval<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        let x = *g.choose(&CHANCES_100).unwrap();
        if x < 75 {
            EnumInterval::Finite(FiniteInterval::<T>::arbitrary(g))
        } else if x < 95 {
            EnumInterval::Half(HalfInterval::<T>::arbitrary(g))
        } else {
            EnumInterval::Unbounded
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use crate::prelude::*;

    #[quickcheck]
    fn check_qc_interval(interval: EnumInterval<f32>) {
        let hull = EnumInterval::strict_hull([interval]).unwrap();
        assert_eq!(hull, interval);
    }

    #[quickcheck]
    fn check_qc_two_intervals(a: EnumInterval<f32>, b: EnumInterval<f32>) {
        check_intersect_and_merge(a, b)
    }

    fn check_intersect_and_merge(a: EnumInterval<f32>, b: EnumInterval<f32>) {
        let intersection = a.intersection(b);
        let merge = a.try_merge(b);

        if a.intersects(&b) {
            assert!(intersection.is_inhabited());
            assert!(merge.is_some());
        } else {
            assert!(intersection.is_empty());
            if a.connects(&b) {
                assert!(merge.is_some());
            } else {
                if a.is_empty() || b.is_empty() {
                    assert!(merge.is_some());
                } else {
                    assert!(merge.is_none());
                }
            }
        }
    }

    #[test]
    fn test_regressions() {
        let a = EnumInterval::Half(HalfInterval::new(Side::Right, FiniteBound::closed(-0.0)));

        let b = EnumInterval::Finite(FiniteInterval::new(
            FiniteBound::open(-0.0),
            FiniteBound::closed(44411.26),
        ));

        check_intersect_and_merge(a, b);

        let a = EnumInterval::Finite(FiniteInterval::new(
            FiniteBound::open(-6.2386875e25),
            FiniteBound::open(0.0),
        ));
        let b = EnumInterval::Finite(FiniteInterval::new(
            FiniteBound::open(0.0),
            FiniteBound::open(2.0899204e32),
        ));

        check_intersect_and_merge(a, b);
    }
}
