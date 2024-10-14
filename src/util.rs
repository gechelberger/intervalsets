/// Generic impl of a commutative operation trait, reversing lhs and rhs.
///
/// The operands are swapped to take advantage of an existing
/// implementation. A matching output type must also be provided.
///
/// # Examples
/// commutative_impl!(TraitName, func_name, LeftType, RightType, OutType);
macro_rules! commutative_op_impl {
    ($tt:ident, $fn:ident, $t_lhs:ty, $t_rhs:ty, $t_out:ty) => {
        impl<T: Numeric> $tt<$t_rhs> for $t_lhs {
            type Output = $t_out;

            fn $fn(&self, rhs: &$t_rhs) -> Self::Output {
                rhs.$fn(self)
            }
        }
    };
}

pub(crate) use commutative_op_impl;

/// Generic impl of a commutative predicate trait, reversing lhs and rhs.
///
/// # Examples
/// commutative_predicate_impl!(TraitName, func_name, LeftType, RightType);
macro_rules! commutative_predicate_impl {
    ($tt:ident, $fn:ident, $t_lhs:ty, $t_rhs:ty) => {
        impl<T: Numeric> $tt<$t_rhs> for $t_lhs {
            fn $fn(&self, rhs: &$t_rhs) -> bool {
                rhs.$fn(self)
            }
        }
    };
}

pub(crate) use commutative_predicate_impl;
