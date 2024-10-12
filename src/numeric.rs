
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumericSet {
    Natural,
    Integer,
    // Rational,
    Real,
}

impl NumericSet {

    pub fn in_real(self) -> bool {
        true
    }

    pub fn in_integer(self) -> bool {
        self != Self::Real
    }

    pub fn in_natural(self) -> bool {
        self == Self::Natural
    }
}


pub trait Numeric: Sized + PartialOrd + PartialEq + num::traits::NumOps + num::Zero + num::One  {
    fn numeric_set() -> NumericSet;
}

macro_rules! numeric_impl {
    ($t:ty, $v:expr) => {
        impl Numeric for $t {
            #[inline]
            fn numeric_set() -> NumericSet {
                $v
            }
        }
    };
}

numeric_impl!(usize, NumericSet::Natural);
numeric_impl!(u8, NumericSet::Natural);
numeric_impl!(u16, NumericSet::Natural);
numeric_impl!(u32, NumericSet::Natural);
numeric_impl!(u64, NumericSet::Natural);
numeric_impl!(u128, NumericSet::Natural);

numeric_impl!(isize, NumericSet::Integer);
numeric_impl!(i8, NumericSet::Integer);
numeric_impl!(i16, NumericSet::Integer);
numeric_impl!(i32, NumericSet::Integer);
numeric_impl!(i64, NumericSet::Integer);
numeric_impl!(i128, NumericSet::Integer);

numeric_impl!(f32, NumericSet::Real);
numeric_impl!(f64, NumericSet::Real);


