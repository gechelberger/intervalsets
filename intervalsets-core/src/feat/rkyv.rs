#[cfg(test)]
mod tests {
    use rkyv::rancor::Error;

    use crate::prelude::*;

    fn check_interval_round_trip(item: EnumInterval<f32>) {
        let encoded = rkyv::to_bytes::<Error>(&item).unwrap();
        let decoded = rkyv::from_bytes::<EnumInterval<f32>, Error>(&encoded).unwrap();
        assert_eq!(item, decoded);
    }

    #[test]
    fn test_intervals() {
        check_interval_round_trip(EnumInterval::<f32>::empty());
        check_interval_round_trip(EnumInterval::closed(0.0, 100.0));
        check_interval_round_trip(EnumInterval::open(0.0, 100.0));
        check_interval_round_trip(EnumInterval::unbound_open(0.0));
        check_interval_round_trip(EnumInterval::closed_unbound(0.0));
        check_interval_round_trip(EnumInterval::<f32>::unbounded());
    }
}
