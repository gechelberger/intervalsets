use std::hash::Hash;

use crate::bound::Bound;
use crate::numeric::Domain;
use crate::{Interval, IntervalSet};

impl<T: Hash> Hash for Bound<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "Bound".hash(state);
        self.bound_type().hash(state);
        self.value().hash(state);
    }
}

impl<T: Hash + Domain> Hash for Interval<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "Interval".hash(state);
        self.0.hash(state);
    }
}

impl<T: Hash + Domain> Hash for IntervalSet<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "IntervalSet".hash(state);
        self.intervals().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::hash::{DefaultHasher, Hash, Hasher};
    fn do_hash<T: Hash>(item: T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }

    pub(super) fn check_hash<T: Hash + PartialEq>(a: &T, b: &T) {
        if a == b {
            assert_eq!(do_hash(a), do_hash(b));
        } else {
            // hash collissions are allowed, but highly unlikely
            assert_ne!(do_hash(a), do_hash(b));
        }
    }

    #[test]
    fn test_hash_stable_interval() {
        check_hash(&Interval::<i8>::empty(), &Interval::<i8>::empty());
        check_hash(&Interval::<i8>::unbounded(), &Interval::<i8>::unbounded());
        check_hash(
            &Interval::<i8>::closed(0, 10),
            &Interval::<i8>::closed(0, 10),
        );

        // f32 & f64 are not Hash
        //check_hash(
        //    &Interval::<f64>::open(0.0, 10.0),
        //    &Interval::<f64>::open(0.0, 10.0),
        //)
    }

    #[test]
    fn test_hash_stable_set() {
        check_hash(&IntervalSet::<i8>::empty(), &Interval::<i8>::empty().into());
    }

    #[quickcheck]
    fn check_hash_interval_set(a: i8, b: i8) {
        let set = IntervalSet::from_iter([Interval::closed(-50, 50)]);

        let other: IntervalSet<_> = Interval::closed(a, b).into();
        check_hash(&set, &other);
    }

    #[quickcheck]
    fn check_hash_stable_interval(a: i8, b: i8) {
        let interval = Interval::closed(-50, 50);
        check_hash(&interval, &Interval::closed(a, b));
    }
}

#[cfg(feature = "rust_decimal")]
#[cfg(test)]
mod decimal_tests {
    use super::*;
    use rust_decimal::Decimal;

    #[quickcheck]
    fn check_hash_decimal_interval(a: f32, b: f32) {
        let a = Decimal::from_f32_retain(a);
        let b = Decimal::from_f32_retain(b);
        if a.is_none() || b.is_none() {
            return;
        }
        let a = a.unwrap();
        let b = b.unwrap();

        let interval = Interval::open(a, b);
        super::tests::check_hash(&interval, &Interval::open(a, b));
        super::tests::check_hash(&interval, &Interval::closed(a, b));
        super::tests::check_hash(&interval, &Interval::open_closed(a, b));
        super::tests::check_hash(&interval, &Interval::closed_open(a, b));
    }
}
