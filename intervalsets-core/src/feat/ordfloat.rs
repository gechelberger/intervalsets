use core::convert::Infallible;

use num_traits::float::FloatCore;
use ordered_float::{NotNan, OrderedFloat};

use crate::cast::{CastElement, LossyCastElement, TryCastElement};
use crate::error::MathError;
use crate::measure::Widthable;
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

impl<T: FloatCore + Midpoint<Error = Infallible>> Midpoint for NotNan<T> {
    type Error = Infallible;

    /// Infallible by contract: values stored in any in-tree set type
    /// are validated finite at construction, and `NotNan` further
    /// excludes `NaN` by construction. Delegates to the inner
    /// `T::midpoint`; the resulting midpoint of two finite inputs is
    /// guaranteed finite, so re-wrapping in `NotNan` cannot fail.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        let Ok(mid) = self.into_inner().midpoint(other.into_inner());
        Ok(NotNan::new(mid).expect("midpoint of finite floats is non-NaN"))
    }
}

impl<T: FloatCore + Widthable<Output = T>> Widthable for NotNan<T> {
    type Output = T;

    /// Delegates to the inner float's `Widthable`. Returns `None` if
    /// the diff is non-finite (overflow at extreme inputs).
    fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
        T::width_between(&**left, &**right)
    }
}

impl<T: FloatCore + Widthable<Output = T>> Widthable for OrderedFloat<T> {
    type Output = T;

    /// Delegates to the inner float's `Widthable`. Returns `None` if
    /// the diff is non-finite (overflow at extreme inputs).
    fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
        T::width_between(&left.0, &right.0)
    }
}

impl<T: Midpoint<Error = Infallible>> Midpoint for OrderedFloat<T> {
    type Error = Infallible;

    /// Infallible by contract: values stored in any in-tree set type
    /// are validated finite at construction (`Element::validate` rejects
    /// `NaN` and `±INF` for `OrderedFloat`). Delegates to the inner
    /// `T::midpoint`.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        let Ok(mid) = self.0.midpoint(other.0);
        Ok(OrderedFloat(mid))
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

// === Cast support (LossyCastElement) ===
//
// `az::SaturatingCast` is not implemented for the `ordered_float`
// wrappers (neither az nor ordered-float depend on each other). We
// provide `LossyCastElement` directly by delegating to the inner
// float's impl and re-wrapping. The result is always finite (inner
// `LossyCast` for floats clamps to `[U::MIN, U::MAX]`), so re-wrapping
// in `NotNan` is infallible.
//
// `TryCast` works for these wrappers out of the box via the existing
// `T: ToPrimitive, U: NumCast + Element` bound (ordered-float provides
// `ToPrimitive`/`NumCast` impls on both wrappers).
//
// `Cast` (infallible widening) works automatically for
// `NotNan<f32> -> NotNan<f64>` (ordered-float provides
// `From<NotNan<f32>> for NotNan<f64>`). For `OrderedFloat<f32> ->
// OrderedFloat<f64>` no analogous `Into` exists upstream and the
// orphan rule prevents us from adding one; users should use
// `TryCast` (always `Ok` for widening) or unwrap/rewrap manually.

impl<T, U> LossyCastElement<OrderedFloat<U>> for OrderedFloat<T>
where
    T: LossyCastElement<U>,
{
    #[inline]
    fn lossy_cast_element(self) -> OrderedFloat<U> {
        OrderedFloat(self.0.lossy_cast_element())
    }
}

impl<T, U> LossyCastElement<NotNan<U>> for NotNan<T>
where
    T: LossyCastElement<U> + FloatCore,
    U: FloatCore,
{
    #[inline]
    fn lossy_cast_element(self) -> NotNan<U> {
        // The inner `T -> U` is saturating + clamping, so the result
        // is finite (and therefore non-NaN). Re-wrap via
        // `new_unchecked` would also work; `new(...).expect(...)`
        // keeps the safety floor without a measurable cost.
        let raw = self.into_inner().lossy_cast_element();
        NotNan::new(raw).expect("LossyCast of finite NotNan produces non-NaN")
    }
}

// `CastElement` for `NotNan<f32> → NotNan<f64>`: ordered-float
// provides `From<NotNan<f32>> for NotNan<f64>` upstream (f32 → f64 is
// lossless and finite-preserving). No analogous upstream `From` exists
// for `OrderedFloat`, so users widen `OrderedFloat` via `TryCast`.

impl CastElement<NotNan<f64>> for NotNan<f32> {
    #[inline]
    fn cast_element(self) -> NotNan<f64> {
        self.into()
    }
}

// `TryCastElement` for wrapper-pair narrowing/widening. Delegates to
// the wrapper's `NumCast` impl (ordered-float provides
// `NumCast for OrderedFloat<T> where T: NumCast` and
// `NumCast for NotNan<T> where T: FloatCore`).

impl<T, U> TryCastElement<OrderedFloat<U>> for OrderedFloat<T>
where
    T: num_traits::ToPrimitive,
    U: num_traits::NumCast,
{
    #[inline]
    fn try_cast_element(self) -> Option<OrderedFloat<U>> {
        <OrderedFloat<U> as num_traits::NumCast>::from(self)
    }
}

impl<T, U> TryCastElement<NotNan<U>> for NotNan<T>
where
    T: FloatCore,
    U: FloatCore,
{
    #[inline]
    fn try_cast_element(self) -> Option<NotNan<U>> {
        <NotNan<U> as num_traits::NumCast>::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::SetBounds;
    use crate::factory::traits::*;
    use crate::EnumInterval;

    #[test]
    fn test_not_nan_factory_construction() {
        let x = EnumInterval::closed(
            NotNan::new(0.0_f32).unwrap(),
            NotNan::new(10.0_f32).unwrap(),
        );
        assert_eq!(x.left().unwrap().value(), &NotNan::new(0.0).unwrap());
        assert_eq!(x.right().unwrap().value(), &NotNan::new(10.0).unwrap());
    }

    #[test]
    fn test_ord_float_factory_construction() {
        let x = EnumInterval::closed(OrderedFloat(0.0_f32), OrderedFloat(10.0_f32));
        assert_eq!(x.left().unwrap().value(), &OrderedFloat(0.0));
        assert_eq!(x.right().unwrap().value(), &OrderedFloat(10.0));
    }

    #[test]
    fn test_validate_rejects_inf_for_ordfloat_wrappers() {
        // NotNan refuses NaN at its own constructor, so the only path
        // for non-finite to reach a `FiniteBound` is via ±INF — which
        // `Element::validate` then rejects.
        use crate::error::Error;

        let inf = NotNan::new(f32::INFINITY).unwrap();
        let zero = NotNan::new(0.0_f32).unwrap();
        let r = EnumInterval::try_closed(zero, inf);
        assert!(matches!(r, Err(Error::InvalidBoundLimit)));

        // OrderedFloat admits any T; both NaN and ±INF reach validate.
        let r = EnumInterval::try_closed(OrderedFloat(0.0_f32), OrderedFloat(f32::NAN));
        assert!(matches!(r, Err(Error::InvalidBoundLimit)));

        let r = EnumInterval::try_closed(OrderedFloat(0.0_f32), OrderedFloat(f32::INFINITY));
        assert!(matches!(r, Err(Error::InvalidBoundLimit)));
    }

    #[test]
    fn test_midpoint_not_nan() {
        let a = NotNan::new(2.0_f32).unwrap();
        let b = NotNan::new(4.0_f32).unwrap();
        assert_eq!(a.midpoint(b).unwrap(), NotNan::new(3.0_f32).unwrap());
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
    }

    // === Cast trait coverage ===

    mod cast {
        use super::*;
        use crate::bound::FiniteBound;
        use crate::cast::{Cast, LossyCast, TryCast};
        use crate::error::Error;
        use crate::sets::FiniteInterval;

        // ---------- TryCast (works out of the box via NumCast / ToPrimitive) ----------

        #[test]
        fn try_cast_ordered_float_widening() {
            let x = FiniteInterval::closed(OrderedFloat(0.0_f32), OrderedFloat(10.0_f32));
            let y: FiniteInterval<OrderedFloat<f64>> = x.try_cast().unwrap();
            assert_eq!(
                y,
                FiniteInterval::closed(OrderedFloat(0.0_f64), OrderedFloat(10.0_f64))
            );
        }

        #[test]
        fn try_cast_ordered_float_narrowing_overflow() {
            // f64::MAX rounds to f32::INFINITY via NumCast; the
            // post-cast `Element::validate` rejects non-finite.
            let x = FiniteInterval::closed(OrderedFloat(0.0_f64), OrderedFloat(f64::MAX));
            let y: Result<FiniteInterval<OrderedFloat<f32>>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        #[test]
        fn try_cast_not_nan_widening() {
            let lo = NotNan::new(0.0_f32).unwrap();
            let hi = NotNan::new(10.0_f32).unwrap();
            let x = FiniteInterval::closed(lo, hi);
            let y: FiniteInterval<NotNan<f64>> = x.try_cast().unwrap();
            assert_eq!(
                y,
                FiniteInterval::closed(
                    NotNan::new(0.0_f64).unwrap(),
                    NotNan::new(10.0_f64).unwrap()
                )
            );
        }

        // ---------- Cast (only NotNan widening — see module docs) ----------

        #[test]
        fn cast_not_nan_widening_via_into() {
            let lo = NotNan::new(0.0_f32).unwrap();
            let hi = NotNan::new(10.0_f32).unwrap();
            let x = FiniteInterval::closed(lo, hi);
            let y: FiniteInterval<NotNan<f64>> = x.cast();
            assert_eq!(
                y,
                FiniteInterval::closed(
                    NotNan::new(0.0_f64).unwrap(),
                    NotNan::new(10.0_f64).unwrap()
                )
            );
        }

        // ---------- LossyCast (via new LossyCastElement impls) ----------

        #[test]
        fn lossy_cast_ordered_float_narrowing_clamps() {
            let x = FiniteInterval::closed(OrderedFloat(0.0_f64), OrderedFloat(f64::MAX));
            let y: FiniteInterval<OrderedFloat<f32>> = x.lossy_cast();
            // Right bound saturates to f32::MAX (not INF — the inner
            // `f64 -> f32` impl clamps, and `Element::validate` for
            // `OrderedFloat<f32>` would have rejected INF).
            let (_, r) = y.view_raw().unwrap();
            assert_eq!(r.value(), &OrderedFloat(f32::MAX));
        }

        #[test]
        fn lossy_cast_not_nan_narrowing_clamps() {
            let lo = NotNan::new(0.0_f64).unwrap();
            let hi = NotNan::new(f64::MAX).unwrap();
            let x = FiniteInterval::closed(lo, hi);
            let y: FiniteInterval<NotNan<f32>> = x.lossy_cast();
            let (_, r) = y.view_raw().unwrap();
            assert_eq!(*r.value(), NotNan::new(f32::MAX).unwrap());
        }

        #[test]
        fn lossy_cast_ordered_float_widening_is_lossless() {
            let x = FiniteInterval::closed(OrderedFloat(0.0_f32), OrderedFloat(10.5_f32));
            let y: FiniteInterval<OrderedFloat<f64>> = x.lossy_cast();
            assert_eq!(
                y,
                FiniteInterval::closed(OrderedFloat(0.0_f64), OrderedFloat(10.5_f64))
            );
        }

        #[test]
        fn lossy_cast_saturation_snaps_to_closed_for_ord_float() {
            // open(-f64::MAX, f64::MAX) → [f32::MIN, f32::MAX] with
            // both bounds closed (snap-to-closed at saturation).
            use crate::factory::TryFiniteFactory;
            let x =
                FiniteInterval::try_open(OrderedFloat(-f64::MAX), OrderedFloat(f64::MAX)).unwrap();
            let y: FiniteInterval<OrderedFloat<f32>> = x.lossy_cast();
            let (l, r) = y.view_raw().unwrap();
            assert!(l.is_closed());
            assert!(r.is_closed());
            assert_eq!(l.value(), &OrderedFloat(f32::MIN));
            assert_eq!(r.value(), &OrderedFloat(f32::MAX));
        }

        #[test]
        fn lossy_cast_element_finite_bound_ordered_float() {
            // Bound-level LossyCast goes through the new
            // `LossyCastElement<OrderedFloat<U>> for OrderedFloat<T>`
            // impl.
            let b = FiniteBound::closed(OrderedFloat(f64::MAX));
            let c: FiniteBound<OrderedFloat<f32>> = b.lossy_cast();
            assert_eq!(c.value(), &OrderedFloat(f32::MAX));
        }
    }
}
