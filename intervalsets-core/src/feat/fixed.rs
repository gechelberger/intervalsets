// lint doesn't detect usage inside macro
#[allow(unused_imports)]
use num_traits::{CheckedAdd, CheckedSub};

/// private macro for Domain on fixed crate types.
macro_rules! fixed_domain {
    ($($t:ty,) +) => {
        $(
            impl<N: typenum::Unsigned> crate::numeric::Domain for $t {
                fn try_adjacent(&self, side: crate::bound::Side) -> Option<Self> {
                    let bits = self.to_bits();
                    let next = match side {
                        crate::bound::Side::Left => bits.checked_sub(1)?,
                        crate::bound::Side::Right => bits.checked_add(1)?,
                    };
                    Some(Self::from_bits(next))
                }
            }
        )+
    }
}

fixed_domain!(
    fixed::FixedI8<N>,
    fixed::FixedU8<N>,
    fixed::FixedI16<N>,
    fixed::FixedU16<N>,
    fixed::FixedI32<N>,
    fixed::FixedU32<N>,
    fixed::FixedI64<N>,
    fixed::FixedU64<N>,
    fixed::FixedI128<N>,
    fixed::FixedU128<N>,
);

#[cfg(test)]
mod tests {
    use fixed::types::{I6F2, U6F2};

    use crate::bound::Side::*;
    use crate::numeric::Domain;

    #[test]
    fn test_adjacent() {
        let x = I6F2::from_num(5.50);

        let left = x.try_adjacent(Left);
        assert_eq!(left, Some(I6F2::from_num(5.25)));

        let right = x.try_adjacent(Right);
        assert_eq!(right, Some(I6F2::from_num(5.75)));
    }

    #[test]
    fn test_adjacent_uint() {
        let x = U6F2::from_num(0.0);

        let left = x.try_adjacent(Left);
        assert_eq!(left, None);

        let right = x.try_adjacent(Right);
        assert_eq!(right, Some(U6F2::from_num(0.25)));
    }
}
