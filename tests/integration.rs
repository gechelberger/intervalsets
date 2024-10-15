use chrono::{DateTime, TimeDelta, Utc};
use intervalsets::{Contains, Interval, Union};

#[test]
fn itest_contains() {
    let interval = Interval::open_closed(0, 10);
    assert!(!interval.contains(&0));
    assert!(interval.contains(&5));
    assert!(interval.contains(&10));
    assert!(!interval.contains(&11));
}

#[test]
fn itest_format() {
    assert_eq!(format!("{}", Interval::<i32>::unbound()), "(<-, ->)");
    assert_eq!(format!("{}", Interval::closed(0.5, 1.5)), "[0.5, 1.5]");
    assert_eq!(format!("{}", Interval::open(-0.5, 0.5)), "(-0.5, 0.5)");
    assert_eq!(format!("{}", Interval::open_unbound(0.5)), "(0.5, ->)");

    let set = Interval::closed(0.0, 10.0).union(&Interval::open(100.0, 110.0));

    assert_eq!(format!("{}", set), "{[0, 10], (100, 110)}")
}

/*
use intervalsets::Domain;
use intervalsets::Side;
intervalsets::continuous_domain_impl!(DateTime<Utc>);

#[test]
fn test_finite_chrono_width() {
    let dist = TimeDelta::new(100000, 0).unwrap();
    let a = Utc::now();
    let b = a + dist;

    let interval = Interval::open(a, b);
}*/
