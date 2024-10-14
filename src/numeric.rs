use num_traits::{One, Zero};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumericSet {
    Natural,
    Integer,
    // Rational,
    Real,
}

impl NumericSet {
    #[inline]
    pub fn in_real(self) -> bool {
        true
    }

    #[inline]
    pub fn in_integer(self) -> bool {
        self != Self::Real
    }

    #[inline]
    pub fn in_natural(self) -> bool {
        self == Self::Natural
    }
}

pub trait Numeric<Rhs = Self, Output = Self>:
    Sized
    + Copy
    + PartialOrd
    + PartialEq
    + core::ops::Add<Rhs, Output = Output>
    + core::ops::Sub<Rhs, Output = Output>
    + core::ops::Mul<Rhs, Output = Output>
    + core::ops::Div<Rhs, Output = Output>
    + core::ops::Rem<Rhs, Output = Output>
    + Zero
    + One
{
    fn numeric_set() -> NumericSet;

    fn try_finite_add(self, rhs: Rhs) -> Option<Self>;

    fn try_finite_sub(self, rhs: Rhs) -> Option<Self>;
}

macro_rules! numeric_integer_impl {
    ($t:ty, $v:expr) => {
        impl Numeric for $t {
            #[inline]
            fn numeric_set() -> NumericSet {
                $v
            }

            #[inline]
            fn try_finite_add(self, rhs: Self) -> Option<Self> {
                <$t>::checked_add(self, rhs)
            }

            #[inline]
            fn try_finite_sub(self, rhs: Self) -> Option<Self> {
                <$t>::checked_sub(self, rhs)
            }
        }
    };
}

macro_rules! numeric_float_impl {
    ($t:ty) => {
        impl Numeric for $t {
            fn numeric_set() -> NumericSet {
                NumericSet::Real
            }

            fn try_finite_add(self, rhs: Self) -> Option<Self> {
                match self + rhs {
                    <$t>::INFINITY => None,
                    result => Some(result),
                }
            }

            fn try_finite_sub(self, rhs: Self) -> Option<Self> {
                match self - rhs {
                    <$t>::NEG_INFINITY => None,
                    result => Some(result),
                }
            }
        }
    };
}

numeric_integer_impl!(usize, NumericSet::Natural);
numeric_integer_impl!(u8, NumericSet::Natural);
numeric_integer_impl!(u16, NumericSet::Natural);
numeric_integer_impl!(u32, NumericSet::Natural);
numeric_integer_impl!(u64, NumericSet::Natural);
numeric_integer_impl!(u128, NumericSet::Natural);

numeric_integer_impl!(isize, NumericSet::Integer);
numeric_integer_impl!(i8, NumericSet::Integer);
numeric_integer_impl!(i16, NumericSet::Integer);
numeric_integer_impl!(i32, NumericSet::Integer);
numeric_integer_impl!(i64, NumericSet::Integer);
numeric_integer_impl!(i128, NumericSet::Integer);

numeric_float_impl!(f32);
numeric_float_impl!(f64);
