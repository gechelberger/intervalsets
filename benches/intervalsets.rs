use criterion::{black_box, criterion_group, criterion_main, Criterion};
use intervalsets::prelude::*;

// use lib::euler1; // function to profile

pub fn bench_intersection(c: &mut Criterion) {
    c.bench_function("intersect", |b| {
        let expected = Interval::closed(5, 10);
        b.iter(|| {
            let intersection =
                black_box(Interval::closed(0, 10)).intersection(black_box(Interval::closed(5, 15)));
            assert_eq!(intersection, expected);
        })
    });
}

pub fn bench_union(c: &mut Criterion) {
    c.bench_function("union-adjacent", |b| {
        let expected = IntervalSet::from((0, 20));
        b.iter(|| {
            let (a, b, c, d) = black_box((0, 10, 11, 20));
            let lhs = Interval::closed(a, b);
            let rhs = Interval::closed(c, d);
            assert_eq!(lhs.union(rhs), expected);
        })
    });
}

pub fn bench_set_complement(c: &mut Criterion) {
    c.bench_function("set-complement", |b| {
        let expected = Interval::unbounded()
            .difference(Interval::closed(100, 110))
            .difference(Interval::closed(1000, 1100))
            .difference(Interval::closed(10000, 11000))
            .difference(Interval::closed(100000, 110000));
        b.iter(|| {
            let (a, b) = black_box((100, 110));
            let i1 = Interval::closed(a, b);
            let (a, b) = black_box((1000, 1100));
            let i2 = Interval::closed(a, b);
            let (a, b) = black_box((10000, 11000));
            let i3 = Interval::closed(a, b);
            let (a, b) = black_box((100000, 110000));
            let i4 = Interval::closed(a, b);
            let set = IntervalSet::from_iter([i1, i2, i3, i4]);
            assert_eq!(set.complement(), expected);
        })
    });
}

pub fn bench_set_split(c: &mut Criterion) {
    c.bench_function("set-split", |b| {
        let set = Interval::closed(0, 10)
            .union(Interval::closed(100, 110))
            .union(Interval::closed(1000, 1100))
            .union(Interval::closed(10000, 11000))
            .union(Interval::closed(100000, 110000))
            .union(Interval::closed(1000000, 1100000))
            .union(Interval::closed(10000000, 11000000))
            .union(Interval::closed(100000900, 110000000))
            .union(Interval::closed(1000009000, 1100000000));
        b.iter(|| {
            let x = black_box(set.clone());
            let (left, right) = x.split(10500, Side::Left);
        })
    });
}

pub fn bench_interval_hull_by_value(c: &mut Criterion) {
    c.bench_function("interval-hull-by-value", |b| {
        let points = [
            5, 300, -300, 32, 44, 83, 93, -1000, 20, 84, 74, -33, 49, 400, 55, 32, -2000, 100, 22,
            73, 1000, 3000, 30, -200, 432, 4000, 300, -3000, 12,
        ];
        b.iter(|| {
            let x = black_box(points.clone());
            Interval::convex_hull(x)
        })
    });
}

pub fn bench_interval_hull_by_ref(c: &mut Criterion) {
    c.bench_function("interval-hull-by-ref", |b| {
        let points = [
            5, 300, -300, 32, 44, 83, 93, -1000, 20, 84, 74, -33, 49, 400, 55, 32, -2000, 100, 22,
            73, 1000, 3000, 30, -200, 432, 4000, 300, -3000, 12,
        ];
        b.iter(|| Interval::convex_hull(black_box(&points)))
    });
}

criterion_group!(
    benches,
    bench_intersection,
    bench_union,
    bench_set_complement,
    bench_set_split,
    bench_interval_hull_by_value,
    bench_interval_hull_by_ref,
);
criterion_main!(benches);
