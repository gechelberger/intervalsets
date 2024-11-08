#![feature(sort_floats)]

use std::cmp::Ordering;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use intervalsets::prelude::*;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;

fn random_list<T>(n: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    let mut rng = rand::thread_rng();
    let mut input: Vec<T> = vec![];
    for _ in 0..n {
        input.push(rng.gen());
    }
    input
}

pub fn bench_partial_ord_sort(c: &mut Criterion) {
    let integers: Vec<u64> = random_list(1000);
    let floats: Vec<f32> = integers.iter().map(|x| *x as f32).collect();

    let mut group = c.benchmark_group("partial-ord-sort");
    group.bench_function("slice::sort_floats", |b| {
        b.iter(|| {
            let mut input = black_box(floats.clone());
            input.as_mut_slice().sort_floats();
        })
    });

    group.bench_function("slice::sort-unstable-total-cmp-f32", |b| {
        b.iter(|| {
            let mut input = black_box(floats.clone());
            input.as_mut_slice().sort_unstable_by(|a, b| a.total_cmp(b));
        })
    });
    group.bench_function("slice::sort-unstable-partial-cmp-f32", |b| {
        b.iter(|| {
            let mut input = black_box(floats.clone());
            input
                .as_mut_slice()
                .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        })
    });

    group.bench_function("slice::sort-unstable-u64", |b| {
        b.iter(|| {
            let mut input = black_box(integers.clone());
            input.as_mut_slice().sort_unstable();
        })
    });
}

pub fn bench_interval_intersection(c: &mut Criterion) {
    c.bench_function("interval-intersection", |b| {
        let expected = Interval::closed(5, 10);
        b.iter(|| {
            let intersection =
                black_box(Interval::closed(0, 10)).intersection(black_box(Interval::closed(5, 15)));
            assert_eq!(intersection, expected);
        })
    });
}

pub fn bench_interval_union(c: &mut Criterion) {
    c.bench_function("interval-union-adjacent", |b| {
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

pub fn bench_interval_difference(c: &mut Criterion) {
    c.bench_function("interval-difference", |b| {
        b.iter(|| {
            let (a, b, c, d) = black_box((0, 5, 10, 15));
            let ac = Interval::closed(a, c);
            let bd = Interval::closed(b, d);
            ac.difference(bd)
        })
    });
}

pub fn bench_interval_sym_difference(c: &mut Criterion) {
    c.bench_function("interval-sym-difference", |b| {
        b.iter(|| {
            let (a, b, c, d) = black_box((0, 5, 10, 15));
            let ac = Interval::closed(a, c);
            let bd = Interval::closed(b, d);
            ac.sym_difference(bd)
        })
    });
}

criterion_group!(
    benches,
    bench_partial_ord_sort,
    bench_set_complement,
    bench_set_split,
    bench_interval_hull_by_value,
    bench_interval_hull_by_ref,
    bench_interval_difference,
    bench_interval_sym_difference,
    bench_interval_intersection,
    bench_interval_union,
);
criterion_main!(benches);
