use super::math::TrySub;
use crate::bound::{SetBounds, Side};
use crate::measure::Extent;
use crate::numeric::Zero;
use crate::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

pub trait Span {
    type Output;
    type Error;

    fn span(&self) -> Result<Extent<Self::Output>, Self::Error>;
}

impl<T> Span for FiniteInterval<T>
where
    T: TrySub + Clone,
    <T as TrySub>::Output: Zero,
{
    type Output = <T as TrySub>::Output;
    type Error = <T as TrySub>::Error;

    fn span(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self.view_raw() {
            None => Ok(Extent::Finite(Self::Output::zero())),
            Some((min, max)) => max
                .value()
                .clone()
                .try_sub(min.value().clone())
                .map(Extent::Finite),
        }
    }
}

impl<T> Span for HalfInterval<T>
where
    T: TrySub + Clone,
    <T as TrySub>::Output: Zero,
{
    type Output = <T as TrySub>::Output;
    type Error = <T as TrySub>::Error;

    fn span(&self) -> Result<Extent<Self::Output>, Self::Error> {
        Ok(Extent::Infinite)
    }
}

impl<T> Span for EnumInterval<T>
where
    T: TrySub + Clone,
    <T as TrySub>::Output: Zero,
{
    type Output = <T as TrySub>::Output;
    type Error = <T as TrySub>::Error;

    fn span(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self {
            Self::Finite(inner) => inner.span(),
            Self::Half(inner) => inner.span(),
            Self::Unbounded => Ok(Extent::Infinite),
        }
    }
}

impl<T> Span for MaybeDisjoint<T>
where
    T: TrySub + Clone,
    <T as TrySub>::Output: Zero,
{
    type Output = <T as TrySub>::Output;
    type Error = <T as TrySub>::Error;

    fn span(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self {
            Self::Connected(inner) => inner.span(),
            Self::Disjoint(left, right) => {
                match (left.bound(Side::Left), right.bound(Side::Right)) {
                    (Some(lo), Some(hi)) => hi
                        .value()
                        .clone()
                        .try_sub(lo.value().clone())
                        .map(Extent::Finite),
                    _ => Ok(Extent::Infinite),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MathError;
    use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};

    #[test]
    fn closed_integer_span() {
        let x = FiniteInterval::closed(3_i32, 10);
        assert_eq!(x.span(), Ok(Extent::Finite(7)));
    }

    #[test]
    fn empty_span_is_zero() {
        let x = FiniteInterval::<i32>::empty();
        assert_eq!(x.span(), Ok(Extent::Finite(0)));
    }

    #[test]
    fn span_overflow_surfaces_err() {
        // i32::MAX - i32::MIN overflows i32 — span uses the same storage type.
        let x = FiniteInterval::closed(i32::MIN, i32::MAX);
        assert_eq!(x.span(), Err(MathError::Range));
    }

    #[test]
    fn half_interval_span_is_infinite() {
        let x = HalfInterval::<i32>::closed_unbound(0);
        assert_eq!(x.span(), Ok(Extent::Infinite));
    }

    #[test]
    fn enum_interval_span_dispatches_by_variant() {
        let finite = EnumInterval::closed(3_i32, 10);
        assert_eq!(finite.span(), Ok(Extent::Finite(7)));

        let half = EnumInterval::closed_unbound(0_i32);
        assert_eq!(half.span(), Ok(Extent::Infinite));

        let unbounded = EnumInterval::<i32>::unbounded();
        assert_eq!(unbounded.span(), Ok(Extent::Infinite));
    }

    #[test]
    fn maybe_disjoint_consumed_span_is_zero() {
        let x = MaybeDisjoint::<i32>::empty();
        assert_eq!(x.span(), Ok(Extent::Finite(0)));
    }

    #[test]
    fn maybe_disjoint_connected_delegates() {
        let x = MaybeDisjoint::from_interval(EnumInterval::closed(3_i32, 10));
        assert_eq!(x.span(), Ok(Extent::Finite(7)));
    }

    #[test]
    fn maybe_disjoint_pair_spans_outer_hull() {
        // [0, 1] ∪ [5, 10] — width is 6, span is 10.
        let a = EnumInterval::closed(0_i32, 1);
        let b = EnumInterval::closed(5_i32, 10);
        let x = MaybeDisjoint::from_pair(a, b);
        assert_eq!(x.span(), Ok(Extent::Finite(10)));
    }

    #[test]
    fn maybe_disjoint_pair_with_infinite_hull() {
        // (<-, 0] ∪ [5, ->) — outer hull is unbounded on both sides.
        let a = EnumInterval::unbound_closed(0_i32);
        let b = EnumInterval::closed_unbound(5_i32);
        let x = MaybeDisjoint::from_pair(a, b);
        assert_eq!(x.span(), Ok(Extent::Infinite));
    }
}
