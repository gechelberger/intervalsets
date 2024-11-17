use arbitrary::{Arbitrary, Result};

use crate::bound::BoundType::{self, *};
use crate::bound::FiniteBound;
use crate::bound::Side::{self, *};
use crate::numeric::Domain;
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
        let interval = FiniteInterval::new(FiniteBound::arbitrary(u)?, FiniteBound::arbitrary(u)?);

        Ok(interval)
    }
}

impl<'a, T: Domain + Arbitrary<'a>> Arbitrary<'a> for HalfInterval<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self> {
        let interval = HalfInterval::new(Side::arbitrary(u)?, FiniteBound::arbitrary(u)?);

        Ok(interval)
    }
}

impl<'a, T: Domain + Arbitrary<'a>> Arbitrary<'a> for EnumInterval<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self> {
        let interval = match u.choose_index(3).unwrap() {
            0 => Self::Finite(FiniteInterval::arbitrary(u)?),
            1 => Self::Half(HalfInterval::arbitrary(u)?),
            2 => Self::Unbounded,
            _ => unreachable!(),
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
