use quickcheck::{Arbitrary, Gen};

use crate::bound::BoundType::{self, *};
use crate::bound::FiniteBound;
use crate::bound::Side::{self, *};
use crate::numeric::Element;
use crate::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

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

impl<T: Element + Clone + Arbitrary> Arbitrary for FiniteBound<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        loop {
            if let Ok(b) = FiniteBound::try_new(BoundType::arbitrary(g), T::arbitrary(g)) {
                return b;
            }
        }
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

                match FiniteInterval::try_new(left, right) {
                    Ok(interval) => interval,
                    Err(_) => Self::arbitrary(g),
                }
            }
        }
    }
}

impl<T: Element + Clone + Arbitrary> Arbitrary for HalfInterval<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        let side = Side::arbitrary(g);
        let bound = FiniteBound::<T>::arbitrary(g);

        match HalfInterval::try_new(side, bound) {
            Ok(interval) => interval,
            Err(_) => Self::arbitrary(g),
        }
    }
}

impl<T: Element + Clone + Arbitrary> Arbitrary for EnumInterval<T> {
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

impl<T: Element + Clone + Arbitrary> Arbitrary for MaybeDisjoint<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        // Generate EnumIntervals and let `from_pair` apply the invariants.
        let a = EnumInterval::<T>::arbitrary(g);
        let b = EnumInterval::<T>::arbitrary(g);
        MaybeDisjoint::from_pair(a, b)
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use crate::prelude::*;
    use crate::sets::MaybeDisjoint;

    #[quickcheck]
    fn check_qc_interval(interval: EnumInterval<f32>) {
        let hull = EnumInterval::try_hull([interval]).unwrap();
        assert_eq!(hull, interval);
    }

    #[quickcheck]
    fn check_qc_two_intervals(a: EnumInterval<f32>, b: EnumInterval<f32>) {
        check_intersect_and_merge(a, b)
    }

    /// Round-trip: arbitrary MD's pieces, when iterated and rebuilt
    /// via `from_pair`, yield the same MD. Pins the invariant that
    /// `arbitrary` produces canonical-form MDs.
    #[quickcheck]
    fn check_qc_maybe_disjoint_roundtrips(md: MaybeDisjoint<f32>) {
        let mut pieces = md.clone();
        let a = pieces.next().unwrap_or_default();
        let b = pieces.next().unwrap_or_default();
        assert!(pieces.next().is_none()); // never more than 2 pieces
        assert_eq!(MaybeDisjoint::from_pair(a, b), md);
    }

    /// `connects ⇒ merge_connected.is_some()` contract holds for
    /// arbitrary MD vs arbitrary EnumInterval.
    #[quickcheck]
    fn check_qc_md_connects_implies_merge(md: MaybeDisjoint<f32>, iv: EnumInterval<f32>) {
        if md.connects(&iv) {
            assert!(md.merge_connected(iv).is_some());
        }
    }

    /// MD's hull encloses all elements: any element of the MD is
    /// contained in the hull.
    #[quickcheck]
    fn check_qc_md_hull_contains_pieces(md: MaybeDisjoint<f32>) {
        let hull = md.hull();
        for piece in md {
            assert!(hull.contains(&piece));
        }
    }

    fn check_intersect_and_merge(a: EnumInterval<f32>, b: EnumInterval<f32>) {
        let intersection = a.intersection(b);
        let merge = a.merge_connected(b);

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
