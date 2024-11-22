#[allow(unused)]
use serde::{Deserialize, Serialize};

#[allow(unused)]
use crate::bound::ord::{FiniteOrdBound, FiniteOrdBoundKind, OrdBound, OrdBoundPair};
#[allow(unused)]
use crate::bound::{BoundType, FiniteBound, Side};
#[allow(unused)]
use crate::factory::{EmptyFactory, FiniteFactory, HalfBoundedFactory, UnboundedFactory};
#[allow(unused)]
use crate::sets::EnumInterval;

#[cfg(test)]
mod brief {
    use super::*;

    fn round_trip<X>(item: X) -> bool
    where
        X: PartialEq + Serialize + core::fmt::Debug,
        for<'a> X: Deserialize<'a>,
    {
        let mut buffer = [0u8; 128];
        let encoded = serde_brief::to_slice(&item, &mut buffer).unwrap();
        let decoded = serde_brief::from_slice(&encoded).unwrap();
        item == decoded
    }

    #[test]
    fn test_intervals() {
        assert!(round_trip(EnumInterval::<f32>::empty()));
        assert!(round_trip(EnumInterval::<i32>::empty()));
        assert!(round_trip(EnumInterval::closed(0, 100)));
        assert!(round_trip(EnumInterval::open(0.0, 100.0)));
        assert!(round_trip(EnumInterval::unbound_open(0.0)));
        assert!(round_trip(EnumInterval::closed_unbound(0)));
        assert!(round_trip(EnumInterval::<f32>::unbounded()));
    }

    #[test]
    fn test_ord_bounds() {
        assert!(round_trip(OrdBoundPair::from(EnumInterval::<f32>::empty())));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::<i32>::empty())));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::closed(0, 100))));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::open(
            0.0, 100.0
        ))));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::unbound_open(
            0.0
        ))));
        assert!(round_trip(OrdBoundPair::from(
            EnumInterval::closed_unbound(0)
        )));
        assert!(round_trip(OrdBoundPair::from(
            EnumInterval::<f32>::unbounded()
        )));
    }
}

#[cfg(test)]
mod json {
    use super::*;

    fn round_trip<X: PartialEq + Serialize + for<'a> Deserialize<'a> + core::fmt::Debug>(
        item: X,
    ) -> bool {
        let encoded = serde_json::to_string(&item).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();
        item == decoded
    }

    #[test]
    fn test_intervals() {
        assert!(round_trip(EnumInterval::<f32>::empty()));
        assert!(round_trip(EnumInterval::<i32>::empty()));
        assert!(round_trip(EnumInterval::closed(0, 100)));
        assert!(round_trip(EnumInterval::open(0.0, 100.0)));
        assert!(round_trip(EnumInterval::unbound_open(0.0)));
        assert!(round_trip(EnumInterval::closed_unbound(0)));
        assert!(round_trip(EnumInterval::<f32>::unbounded()));
    }

    #[test]
    fn test_ord_bounds() {
        assert!(round_trip(OrdBoundPair::from(EnumInterval::<f32>::empty())));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::<i32>::empty())));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::closed(0, 100))));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::open(
            0.0, 100.0
        ))));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::unbound_open(
            0.0
        ))));
        assert!(round_trip(OrdBoundPair::from(
            EnumInterval::closed_unbound(0)
        )));
        assert!(round_trip(OrdBoundPair::from(
            EnumInterval::<f32>::unbounded()
        )));
    }
}

#[cfg(test)]
mod rmp {
    use super::*;

    fn round_trip<X: PartialEq + Serialize + for<'a> Deserialize<'a> + core::fmt::Debug>(
        item: X,
    ) -> bool {
        let encoded = rmp_serde::encode::to_vec(&item).unwrap();
        let decoded = rmp_serde::decode::from_slice(&encoded).unwrap();
        item == decoded
    }

    #[test]
    fn test_intervals() {
        assert!(round_trip(EnumInterval::<f32>::empty()));
        assert!(round_trip(EnumInterval::<i32>::empty()));
        assert!(round_trip(EnumInterval::closed(0, 100)));
        assert!(round_trip(EnumInterval::open(0.0, 100.0)));
        assert!(round_trip(EnumInterval::unbound_open(0.0)));
        assert!(round_trip(EnumInterval::closed_unbound(0)));
        assert!(round_trip(EnumInterval::<f32>::unbounded()));
    }

    #[test]
    fn test_ord_bounds() {
        assert!(round_trip(OrdBoundPair::from(EnumInterval::<f32>::empty())));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::<i32>::empty())));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::closed(0, 100))));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::open(
            0.0, 100.0
        ))));
        assert!(round_trip(OrdBoundPair::from(EnumInterval::unbound_open(
            0.0
        ))));
        assert!(round_trip(OrdBoundPair::from(
            EnumInterval::closed_unbound(0)
        )));
        assert!(round_trip(OrdBoundPair::from(
            EnumInterval::<f32>::unbounded()
        )));
    }
}
