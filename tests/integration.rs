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
