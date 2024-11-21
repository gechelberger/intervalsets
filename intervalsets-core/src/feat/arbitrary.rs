use arbitrary::{Arbitrary, Result};

use crate::bound::BoundType::{self, *};
use crate::bound::FiniteBound;
use crate::bound::Side::{self, *};
use crate::numeric::{Domain, Zero};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl Arbitrary<'_> for Side {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        static ALL_SIDES: [Side; 2] = [Left, Right];
        u.choose(&ALL_SIDES).copied()
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (1, Some(1))
    }
}

impl Arbitrary<'_> for BoundType {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> Result<Self> {
        static ALL_BOUND_TYPES: [BoundType; 2] = [Open, Closed];
        u.choose(&ALL_BOUND_TYPES).copied()
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (1, Some(1))
    }
}

impl<'a, T: Arbitrary<'a>> Arbitrary<'a> for FiniteBound<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self> {
        let bound = FiniteBound::new(BoundType::arbitrary(u)?, T::arbitrary(u)?);

        Ok(bound)
    }
}

impl<'a, T: Domain + Arbitrary<'a>> Arbitrary<'a> for FiniteInterval<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self> {
        let interval =
            FiniteInterval::new_strict(FiniteBound::arbitrary(u)?, FiniteBound::arbitrary(u)?)
                .unwrap_or(Self::empty());

        Ok(interval)
    }
}

impl<'a, T: Domain + Zero + Arbitrary<'a>> Arbitrary<'a> for HalfInterval<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self> {
        let interval = HalfInterval::new_strict(Side::arbitrary(u)?, FiniteBound::arbitrary(u)?);

        match interval {
            Some(inner) => Ok(inner),
            None => Self::arbitrary(u),
        }
    }
}

impl<'a, T: Domain + Zero + Arbitrary<'a>> Arbitrary<'a> for EnumInterval<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self> {
        let n = usize::arbitrary(u)? % 100;
        let interval = if n < 75 {
            Self::Finite(FiniteInterval::arbitrary(u)?)
        } else if n < 95 {
            Self::Half(HalfInterval::arbitrary(u)?)
        } else {
            Self::Unbounded
        };

        Ok(interval)
    }
}

#[cfg(test)]
mod tests {

    use arbitrary::Unstructured;

    use super::*;

    #[test]
    fn test_arbitrary() {
        let mut u =
            Unstructured::new(b"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUV");

        let _ = Side::arbitrary(&mut u).unwrap();
        let _ = FiniteBound::<i32>::arbitrary(&mut u).unwrap();
        let _ = FiniteBound::<f32>::arbitrary(&mut u).unwrap();
        let _ = FiniteInterval::<i32>::arbitrary(&mut u).unwrap();
        let _ = FiniteInterval::<f32>::arbitrary(&mut u).unwrap();
        let _ = HalfInterval::<i32>::arbitrary(&mut u).unwrap();
        let _ = HalfInterval::<f32>::arbitrary(&mut u).unwrap();
        let _ = EnumInterval::<i32>::arbitrary(&mut u).unwrap();
        let _ = EnumInterval::<f32>::arbitrary(&mut u).unwrap();
    }
}
