use crate::bound::Side;

pub trait Domain: Sized + Clone + PartialOrd + PartialEq {
    fn try_adjacent(&self, side: Side) -> Option<Self>;
}

#[macro_export]
macro_rules! continuous_domain_impl {
    ($t:ty) => {
        impl $crate::Domain for $t {
            #[inline]
            fn try_adjacent(&self, side: $crate::Side) -> Option<Self> {
                None
            }
        }
    };
}

continuous_domain_impl!(f32);
continuous_domain_impl!(f64);

macro_rules! integer_domain_impl {
    ($t:ty) => {
        impl Domain for $t {
            #[inline]
            fn try_adjacent(&self, side: Side) -> Option<Self> {
                match side {
                    Side::Right => <$t>::checked_add(*self, 1),
                    Side::Left => <$t>::checked_sub(*self, 1),
                }
            }
        }
    };
}

integer_domain_impl!(usize);
integer_domain_impl!(u8);
integer_domain_impl!(u16);
integer_domain_impl!(u32);
integer_domain_impl!(u64);
integer_domain_impl!(u128);

integer_domain_impl!(isize);
integer_domain_impl!(i8);
integer_domain_impl!(i16);
integer_domain_impl!(i32);
integer_domain_impl!(i64);
integer_domain_impl!(i128);
