use serde::{Deserialize, Serialize};

use crate::bound::ord::{OrdBound, OrdBoundFinite, OrdBoundPair};
use crate::bound::{BoundType, FiniteBound, Side};
use crate::sets::EnumInterval;
use crate::Factory;

#[cfg(test)]
mod brief {
    extern crate std;

    use super::*;

    fn round_trip<X>(item: X) -> bool
    where
        X: PartialEq + Serialize + core::fmt::Debug,
        for<'a> X: Deserialize<'a>,
    {
        let mut buffer = [0u8; 128];
        let encoded = serde_brief::to_slice(&item, &mut buffer).unwrap();

        std::println!(
            "brief: {} vs {}: {:?}",
            encoded.len(),
            std::mem::size_of::<X>(),
            item
        );

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
    extern crate std;
    use super::*;

    fn round_trip<X: PartialEq + Serialize + for<'a> Deserialize<'a> + core::fmt::Debug>(
        item: X,
    ) -> bool {
        let encoded = serde_json::to_string(&item).unwrap();

        std::println!(
            "json: {} vs {}: {:?}",
            encoded.len(),
            std::mem::size_of::<X>(),
            item
        );

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
    extern crate std;
    use super::*;

    fn round_trip<X: PartialEq + Serialize + for<'a> Deserialize<'a> + core::fmt::Debug>(
        item: X,
    ) -> bool {
        let encoded = rmp_serde::encode::to_vec(&item).unwrap();

        std::println!(
            "rmp: {} vs {}: {:?}",
            encoded.len(),
            std::mem::size_of::<X>(),
            item
        );

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