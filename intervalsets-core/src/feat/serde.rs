#[allow(unused)]
use serde::{Deserialize, Serialize};

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
        let decoded = serde_brief::from_slice(encoded).unwrap();
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
}

/// Negative tests — confirm `Deserialize` rejects malformed input. We patch
/// the wire output of valid intervals to construct payloads that the
/// serializer would never emit (NaN limits, swapped-order Bounded, etc.) and
/// assert the strict deserialize path errors. JSON is the easiest format to
/// hand-edit; we reuse the same hand-crafted shapes against rmp via a
/// "serialize a sentinel, mutate the bytes" helper.
#[cfg(test)]
mod malformed {
    use super::*;

    #[test]
    fn json_rejects_swapped_bounded() {
        // Build a payload by serializing a valid Bounded(closed(0), closed(10))
        // then swapping the two limit values so the wire shape is identical
        // but lhs > rhs. The strict deserialize path must reject it.
        let canonical =
            serde_json::to_string(&EnumInterval::<f32>::closed(0.0, 10.0)).unwrap();
        // Sanity-check the wire format before we mutate it.
        assert!(
            canonical.contains("Bounded"),
            "unexpected serialized form: {canonical}"
        );
        let swapped = canonical.replacen("0.0", "TMP", 1);
        let swapped = swapped.replacen("10.0", "0.0", 1);
        let swapped = swapped.replace("TMP", "10.0");

        let result: Result<EnumInterval<f32>, _> = serde_json::from_str(&swapped);
        assert!(
            result.is_err(),
            "expected error for swapped-order Bounded, got: {:?}\npayload: {swapped}",
            result
        );
    }

    #[test]
    fn rmp_rejects_nan_in_half_interval() {
        // Build a valid HalfInterval, then patch the f32 bytes to a NaN.
        let valid = EnumInterval::<f32>::unbound_open(0.0);
        let mut bytes = rmp_serde::encode::to_vec(&valid).unwrap();

        // Find the f32 0.0 = [0,0,0,0] and replace with NaN bits.
        let nan_bits = f32::NAN.to_be_bytes();
        let zero_bytes = [0u8; 4];
        let pos = bytes
            .windows(4)
            .position(|w| w == zero_bytes)
            .expect("valid encoding should contain the zero limit value");
        bytes[pos..pos + 4].copy_from_slice(&nan_bits);

        let result: Result<EnumInterval<f32>, _> = rmp_serde::decode::from_slice(&bytes);
        assert!(
            result.is_err(),
            "expected error for NaN in HalfInterval bound, got: {:?}",
            result
        );
    }

    #[test]
    fn rmp_rejects_nan_in_finite_interval() {
        let valid = EnumInterval::<f32>::closed(0.0, 1.0);
        let mut bytes = rmp_serde::encode::to_vec(&valid).unwrap();

        // Replace the first f32 (0.0) with NaN.
        let nan_bits = f32::NAN.to_be_bytes();
        let zero_bytes = [0u8; 4];
        let pos = bytes
            .windows(4)
            .position(|w| w == zero_bytes)
            .expect("valid encoding should contain the zero limit value");
        bytes[pos..pos + 4].copy_from_slice(&nan_bits);

        let result: Result<EnumInterval<f32>, _> = rmp_serde::decode::from_slice(&bytes);
        assert!(
            result.is_err(),
            "expected error for NaN in FiniteInterval bound, got: {:?}",
            result
        );
    }

    #[test]
    fn json_normalizes_discrete_open_bounds() {
        // Discrete `Bounded(open(0), open(10))` is silently normalized to
        // `Bounded(closed(1), closed(9))` by `try_new`. Deserialize matches
        // that semantic — no error, value-preserving.
        // Build the payload by serializing the canonical-form interval
        // and then re-shape it to the open-bound variant the user might
        // have hand-crafted.
        let canonical = serde_json::to_string(&EnumInterval::<i32>::closed(1, 9)).unwrap();
        // Sanity: confirm the serializer emits the shape we expect.
        assert!(
            canonical.contains("Closed"),
            "unexpected serialized form: {canonical}"
        );
        let open_payload = canonical
            .replace("\"Closed\":1", "\"Open\":0")
            .replace("\"Closed\":9", "\"Open\":10");
        let parsed: EnumInterval<i32> = serde_json::from_str(&open_payload).unwrap();
        assert_eq!(parsed, EnumInterval::<i32>::closed(1, 9));
    }
}
