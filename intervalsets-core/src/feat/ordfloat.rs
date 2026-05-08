use num_traits::float::FloatCore;
use ordered_float::{NotNan, OrderedFloat};

use crate::error::{MathError, MidpointError};
use crate::factory::Converter;
use crate::numeric::{Element, Midpoint};
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

impl<T: FloatCore + Element> Element for NotNan<T> {
    fn try_adjacent(&self, _: crate::bound::Side) -> Option<Self> {
        None
    }

    /// Rejects ±INF. `NotNan<T>` already excludes NaN by construction,
    /// but its inner `T` can still be infinite — and `Element::validate`
    /// must reject non-finite for the `FiniteBound` chokepoint to hold.
    #[inline]
    fn validate(self) -> Option<Self> {
        self.into_inner().is_finite().then_some(self)
    }
}

impl<T: FloatCore + Element> Element for OrderedFloat<T> {
    fn try_adjacent(&self, _: crate::bound::Side) -> Option<Self> {
        None
    }

    /// Rejects NaN and ±INF. `OrderedFloat<T>` admits NaN under its
    /// total order, but NaN is never a valid finite-bound limit.
    #[inline]
    fn validate(self) -> Option<Self> {
        self.into_inner().is_finite().then_some(self)
    }
}

impl<T: FloatCore + Element> Converter<T> for NotNan<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        NotNan::new(value).unwrap()
    }
}

impl<T: FloatCore + Element> Converter<T> for OrderedFloat<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        OrderedFloat(value)
    }
}

impl<T: FloatCore + Midpoint<Error = MidpointError>> Midpoint for NotNan<T> {
    type Error = MidpointError;

    /// Delegates to the inner `T::midpoint`. Returns
    /// [`MidpointError`](crate::error::MidpointError) when either
    /// input is non-finite (per the inner float impl's contract);
    /// the resulting midpoint is guaranteed finite, so re-wrapping
    /// in `NotNan` cannot fail.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        let mid = self.into_inner().midpoint(other.into_inner())?;
        Ok(NotNan::new(mid).expect("midpoint of finite floats is non-NaN"))
    }
}

impl<T: Midpoint<Error = MidpointError>> Midpoint for OrderedFloat<T> {
    type Error = MidpointError;

    /// Delegates to the inner `T::midpoint`. Returns
    /// [`MidpointError`](crate::error::MidpointError) when either
    /// input is non-finite — even though `OrderedFloat` admits NaN
    /// under its total order, NaN has no well-defined midpoint.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        Ok(OrderedFloat(self.0.midpoint(other.0)?))
    }
}

// === Value-level TryOp impls (E3) ===
//
// Both wrappers report any non-finite result (INF or NaN) as
// `MathError::Domain`, mirroring the bare `f32`/`f64` impls.
//
// `OrderedFloat::add/sub/mul/div` do not panic on NaN (the wrapper
// admits NaN under its total order) — we just check `is_finite()` on
// the result. `NotNan`'s ops panic if the result would be NaN, so we
// route through the inner `T` first and only re-wrap once `is_finite`
// confirms the result is non-NaN.

macro_rules! ordfloat_impl_try {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: FloatCore> $trait for OrderedFloat<T> {
            type Output = OrderedFloat<T>;
            type Error = MathError;

            #[inline]
            fn $method(self, rhs: Self) -> Result<Self, MathError> {
                let r = OrderedFloat(self.0 $op rhs.0);
                if r.0.is_finite() {
                    Ok(r)
                } else {
                    Err(MathError::Domain)
                }
            }
        }

        impl<T: FloatCore> $trait for NotNan<T> {
            type Output = NotNan<T>;
            type Error = MathError;

            #[inline]
            fn $method(self, rhs: Self) -> Result<Self, MathError> {
                let r = self.into_inner() $op rhs.into_inner();
                if r.is_finite() {
                    Ok(NotNan::new(r).expect("finite result is non-NaN by definition"))
                } else {
                    Err(MathError::Domain)
                }
            }
        }
    };
}

ordfloat_impl_try!(TryAdd, try_add, +);
ordfloat_impl_try!(TrySub, try_sub, -);
ordfloat_impl_try!(TryMul, try_mul, *);
ordfloat_impl_try!(TryDiv, try_div, /);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{EIFactory, FiniteFactory};
    use crate::EnumInterval;

    #[test]
    fn test_not_nan_converter() {
        type F = EIFactory<f32, NotNan<f32>>;

        let x = F::closed(0.0, 10.0);

        assert_eq!(
            x,
            EnumInterval::closed(NotNan::new(0.0).unwrap(), NotNan::new(10.0).unwrap())
        );
    }

    #[test]
    fn test_ord_float_converter() {
        type F = EIFactory<f32, OrderedFloat<f32>>;

        let x = F::closed(0.0, 10.0);
        assert_eq!(
            x,
            EnumInterval::closed(OrderedFloat(0.0), OrderedFloat(10.0),)
        );
    }

    #[test]
    fn test_midpoint_not_nan() {
        let a = NotNan::new(2.0_f32).unwrap();
        let b = NotNan::new(4.0_f32).unwrap();
        assert_eq!(a.midpoint(b).unwrap(), NotNan::new(3.0_f32).unwrap());

        // Infinity is admitted by NotNan but rejected as a midpoint
        // endpoint -- inf.midpoint(-inf) would otherwise produce NaN.
        let inf = NotNan::new(f32::INFINITY).unwrap();
        let zero = NotNan::new(0.0_f32).unwrap();
        assert!(inf.midpoint(zero).is_err());
    }

    #[test]
    fn test_try_ops_ord_float() {
        let two = OrderedFloat(2.0_f64);
        let three = OrderedFloat(3.0_f64);
        let zero = OrderedFloat(0.0_f64);

        assert_eq!(two.try_add(three).unwrap(), OrderedFloat(5.0));
        assert_eq!(two.try_sub(three).unwrap(), OrderedFloat(-1.0));
        assert_eq!(two.try_mul(three).unwrap(), OrderedFloat(6.0));
        assert_eq!(OrderedFloat(6.0_f64).try_div(two).unwrap(), three);

        // Non-finite results: all surface as `Domain`.
        assert_eq!(
            OrderedFloat(f64::MAX).try_add(OrderedFloat(f64::MAX)),
            Err(MathError::Domain)
        );
        // 1.0 / 0.0 = INF → Domain
        assert_eq!(OrderedFloat(1.0_f64).try_div(zero), Err(MathError::Domain));
        // 0.0 / 0.0 = NaN → Domain
        assert_eq!(zero.try_div(zero), Err(MathError::Domain));
    }

    #[test]
    fn test_try_ops_not_nan() {
        let two = NotNan::new(2.0_f64).unwrap();
        let three = NotNan::new(3.0_f64).unwrap();
        let zero = NotNan::new(0.0_f64).unwrap();

        assert_eq!(two.try_add(three).unwrap(), NotNan::new(5.0).unwrap());
        assert_eq!(two.try_sub(three).unwrap(), NotNan::new(-1.0).unwrap());
        assert_eq!(two.try_mul(three).unwrap(), NotNan::new(6.0).unwrap());
        assert_eq!(NotNan::new(6.0_f64).unwrap().try_div(two).unwrap(), three);

        // INF + (-INF) would yield NaN — must surface as `Domain`, not panic.
        let inf = NotNan::new(f64::INFINITY).unwrap();
        let neg_inf = NotNan::new(f64::NEG_INFINITY).unwrap();
        assert_eq!(inf.try_add(neg_inf), Err(MathError::Domain));

        // 1.0 / 0.0 = INF → Domain
        assert_eq!(
            NotNan::new(1.0_f64).unwrap().try_div(zero),
            Err(MathError::Domain)
        );
        // 0.0 / 0.0 would be NaN → Domain (no panic)
        assert_eq!(zero.try_div(zero), Err(MathError::Domain));

        // Overflow → INF → Domain
        assert_eq!(
            NotNan::new(f64::MAX)
                .unwrap()
                .try_add(NotNan::new(f64::MAX).unwrap()),
            Err(MathError::Domain)
        );
    }

    #[test]
    fn test_validate_rejects_non_finite() {
        use crate::bound::{BoundType, FiniteBound};
        use crate::error::Error;

        // OrderedFloat: validate rejects ±INF and NaN.
        assert_eq!(OrderedFloat(f64::INFINITY).validate(), None);
        assert_eq!(OrderedFloat(f64::NEG_INFINITY).validate(), None);
        assert_eq!(OrderedFloat(f64::NAN).validate(), None);
        assert_eq!(
            OrderedFloat(1.5_f64).validate(),
            Some(OrderedFloat(1.5_f64))
        );

        // NotNan: still rejects ±INF post-validate (NotNan only blocks NaN
        // by construction).
        let inf = NotNan::new(f64::INFINITY).unwrap();
        let neg_inf = NotNan::new(f64::NEG_INFINITY).unwrap();
        let one = NotNan::new(1.0_f64).unwrap();
        assert_eq!(inf.validate(), None);
        assert_eq!(neg_inf.validate(), None);
        assert_eq!(one.validate(), Some(one));

        // FiniteBound chokepoint: factory-style construction surfaces
        // the rejection as `Error::InvalidBoundLimit`.
        assert!(matches!(
            FiniteBound::try_new(BoundType::Closed, OrderedFloat(f64::INFINITY)),
            Err(Error::InvalidBoundLimit)
        ));
        assert!(matches!(
            FiniteBound::try_new(BoundType::Closed, NotNan::new(f64::INFINITY).unwrap()),
            Err(Error::InvalidBoundLimit)
        ));
    }

    #[test]
    fn test_midpoint_ord_float() {
        let a = OrderedFloat(2.0_f32);
        let b = OrderedFloat(4.0_f32);
        assert_eq!(a.midpoint(b).unwrap(), OrderedFloat(3.0_f32));

        // NaN is admitted by OrderedFloat's total order but rejected
        // here -- NaN has no well-defined midpoint.
        let nan = OrderedFloat(f32::NAN);
        let zero = OrderedFloat(0.0_f32);
        assert!(nan.midpoint(zero).is_err());

        // Infinity is rejected for the same reason as the float impl.
        let inf = OrderedFloat(f32::INFINITY);
        assert!(inf.midpoint(zero).is_err());
    }
}
