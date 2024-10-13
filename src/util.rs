/// Generic impl of a commutative trait, reversing lhs and rhs.
///
/// # Examples
/// commutative_impl!(TraitName, func_name, LeftType, RightType, OutType);
macro_rules! commutative_impl {
    ($tt:ident, $fn:ident, $t_lhs:ty, $t_rhs:ty, $t_out:ty) => {
        impl<T: Copy + PartialOrd> $tt<$t_rhs> for $t_lhs {
            type Output = $t_out;

            fn $fn(&self, rhs: &$t_rhs) -> Self::Output {
                rhs.$fn(self)
            }
        }
    };
}

pub(crate) use commutative_impl;
