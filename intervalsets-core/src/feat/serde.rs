use serde::{Deserialize, Serialize};

use crate::bound::ord::{OrdBound, OrdBoundFinite, OrdBoundPair};
use crate::bound::{BoundType, FiniteBound, Side};
use crate::sets::EnumInterval;
use crate::Factory;

#[cfg(test)]
mod brief {
    use super::*;

    fn round_trip<X>(item: X) -> bool
    where
        X: PartialEq + Serialize,
        for<'a> X: Deserialize<'a>,
    {
        let mut buffer = [0u8; 128];
        let trimmed = serde_brief::to_slice(&item, &mut buffer).unwrap();
        let inflated = serde_brief::from_slice(&trimmed).unwrap();
        item == inflated
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

    fn round_trip<X: PartialEq + Serialize + for<'a> Deserialize<'a>>(item: X) -> bool {
        let deflated = serde_json::to_string(&item).unwrap();
        let inflated = serde_json::from_str(&deflated).unwrap();
        item == inflated
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
