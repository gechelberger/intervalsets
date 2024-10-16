/// Generic impl of a commutative operation trait, reversing lhs and rhs.
///
/// The operands are swapped to take advantage of an existing
/// implementation. A matching output type must also be provided.
///
/// # Examples
/// commutative_impl!(TraitName, func_name, LeftType, RightType, OutType);
#[macro_export]
macro_rules! commutative_op_impl {
    ($tt:ident, $fn:ident, $t_lhs:ty, $t_rhs:ty, $t_out:ty) => {
        impl<T: $crate::numeric::Domain> $tt<$t_rhs> for $t_lhs {
            type Output = $t_out;

            fn $fn(&self, rhs: &$t_rhs) -> Self::Output {
                rhs.$fn(self)
            }
        }
    };
}

/// Generic impl of a commutative predicate trait, reversing lhs and rhs.
///
/// # Examples
/// commutative_predicate_impl!(TraitName, func_name, LeftType, RightType);
#[macro_export]
macro_rules! commutative_predicate_impl {
    ($tt:ident, $fn:ident, $t_lhs:ty, $t_rhs:ty) => {
        impl<T: $crate::numeric::Domain> $tt<$t_rhs> for $t_lhs {
            fn $fn(&self, rhs: &$t_rhs) -> bool {
                rhs.$fn(self)
            }
        }
    };
}

///
macro_rules! interval_op_passthrough_impl {
    ($tt:ident, $fn:ident, $rhs:ty, $out:ty) => {
        impl<T: $crate::numeric::Domain> $tt<$rhs> for $crate::Interval {
            type Output = $out;
            fn $fn(&self, rhs: &$rhs) -> Self::Output {
                self.0 
            }
        }
    }
}
pub(crate) use interval_op_passthrough_impl;