/*

use chrono::{DateTime, TimeDelta, Utc};
use intervalsets::{continuous_domain_impl, Interval};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct MyDT(DateTime<Utc>);

continuous_domain_impl!(MyDT);

#[test]
fn test_foo() {
    let a = Utc::now();
    let delta = TimeDelta::new(1000, 0).unwrap();
    let b = a + delta;

    let i = Interval::closed(MyDT(a), MyDT(b));
}
 */

use intervalsets::numeric::Zero;
use intervalsets::prelude::*;

#[test]
fn from_readme() {
    let reserved = Interval::closed_open(0, 100)
        .union(Interval::closed_open(200, 300))
        .union(Interval::closed_open(400, 500));

    let requests: Vec<Interval<_>> = vec![
        [10, 20].into(),
        (150..160).into(),
        [200, 210].into(),
        (300, 400).into(),
    ];

    let (acceptable, rejected): (Vec<_>, Vec<_>) = requests
        .into_iter()
        .partition(|interval| !reserved.intersects(interval));

    assert_eq!(
        acceptable,
        vec![Interval::closed_open(150, 160), Interval::open(300, 400),]
    );

    assert_eq!(
        rejected,
        vec![Interval::closed(10, 20), Interval::closed(200, 210),]
    )
}

#[test]
fn test_pseudo_empty() {
    let x = Interval::<u8>::unbound_open(0);
    for i in 0..=255u8 {
        assert!(!x.contains(&i));
    }
    assert_ne!(x, Interval::<u8>::empty());
}

#[test]
fn test_restricted_universe() {
    fn natural_numbers<T: Domain + Zero>() -> Interval<T> {
        Interval::<T>::closed_unbound(T::zero())
    }

    let x = Interval::<u8>::closed(0, 10);
    let y = x.complement().intersection(natural_numbers());
    assert_eq!(y.expect_interval(), Interval::open_unbound(10));
}

#[test]
fn test_unsigned_edge_case() {
    let x = Interval::<u8>::unbound_open(0); // (<-, 0)
    let all_valid_u8_values = Interval::<u8>::closed(0, 255);

    assert_eq!(x.intersection(all_valid_u8_values), Interval::empty());
    assert!(x.count().is_infinite());
}
