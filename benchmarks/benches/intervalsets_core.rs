//#![feature(sort_floats)]

use std::cmp::Ordering;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use intervalsets_core::bound::{FiniteBound, Side};
use intervalsets_core::ops::*;
use intervalsets_core::sets::{FiniteInterval, HalfInterval};
use intervalsets_core::{EnumInterval, Factory as _};
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

pub fn bench_core_intersects(c: &mut Criterion) {
    let x = FiniteInterval::closed(0, 100);
    let y = FiniteInterval::closed(100, 200);

    let mut group = c.benchmark_group("core::intersects");
    group.bench_function("finite-finite", |b| {
        b.iter(|| {
            let x = black_box(x.clone());
            let y = black_box(y.clone());
            //x.intersects(&y);
            y.intersects(&x);
            //intervalsets_core::ops::intersects::intersects_ord(&x, &y);
        })
    });

    let z = HalfInterval::left(FiniteBound::closed(50));

    group.bench_function("finite-half", |b| {
        b.iter(|| {
            let x = black_box(x.clone());
            let z = black_box(z.clone());
            //x.intersects(&z);
            z.intersects(&x);
        })
    });

    group.bench_function("enum-enum", |b| {
        b.iter(|| {
            let x: EnumInterval<_> = black_box(x.clone()).into();
            let y: EnumInterval<_> = black_box(y.clone()).into();
            x.intersects(&y);
        })
    });

    /*group.bench_function("contains-intersects", |b| {
        b.iter(|| {
            let x = black_box(x.clone());
            let y = black_box(y.clone());
            intervalsets_core::ops::intersects::intersects_contains(&x, &y)
        })
    });*/
}

pub fn bench_partial_ord_sort(c: &mut Criterion) {
    let integers: Vec<u64> = random_list(1000);
    let floats: Vec<f32> = integers.iter().map(|x| *x as f32).collect();

    let mut group = c.benchmark_group("partial-ord-sort");
    //group.bench_function("slice::sort_floats", |b| {
    //    b.iter(|| {
    //        let mut input = black_box(floats.clone());
    //        input.as_mut_slice().sort_floats();
    //    })
    //});

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

pub fn bench_core_intersection(c: &mut Criterion) {
    let x = FiniteInterval::closed(0, 100);
    let y = FiniteInterval::closed(50, 150);
    let z = FiniteInterval::closed(150, 200);

    let ex = EnumInterval::from(x.clone());
    let ey = EnumInterval::from(y.clone());
    let ez = EnumInterval::from(z.clone());

    let mut group = c.benchmark_group("core::intersection");
    group.bench_function("finite-finite", |b| {
        b.iter(|| {
            let x = black_box(x.clone());
            let y = black_box(y.clone());
            x.intersection(y);
        })
    });

    group.bench_function("finite-finite-disjoint", |b| {
        b.iter(|| {
            let x = black_box(x.clone());
            let z = black_box(z.clone());
            x.intersection(z);
        });
    });

    group.bench_function("enum-enum", |b| {
        b.iter(|| {
            let ex = black_box(ex.clone());
            let ey = black_box(ey.clone());
            ex.intersection(ey);
        })
    });

    group.bench_function("enum-enum-disjoint", |b| {
        b.iter(|| {
            let ex = black_box(ex.clone());
            let ez = black_box(ez.clone());
            ex.intersection(ez);
        })
    });
}

pub fn bench_core_merged(c: &mut Criterion) {
    let x = FiniteInterval::<f32>::closed(0.0, 100.0);
    let y = FiniteInterval::closed(50.0, 150.0);
    let z = FiniteInterval::closed(200.0, 300.0);
    let a = FiniteInterval::closed(100.0, 150.0);
    let mut group = c.benchmark_group("core::merged");
    group.bench_function("finite-finite-overlapped", |b| {
        b.iter(|| {
            let x = black_box(x.clone());
            let y = black_box(y.clone());
            x.try_merge(y)
        });
    });

    group.bench_function("finite-finite-disjoint", |b| {
        b.iter(|| {
            let x = black_box(x.clone());
            let z = black_box(z.clone());
            x.try_merge(z);
        })
    });

    group.bench_function("finite-finite-adjacent", |b| {
        b.iter(|| {
            let x = black_box(x.clone());
            let a = black_box(a.clone());
            x.try_merge(a);
        })
    });
}

pub fn bench_core_split(c: &mut Criterion) {
    let x = FiniteInterval::closed(0.0, 100.0);
    let mut group = c.benchmark_group("core::split");
    group.bench_function("finite-bisect", |b| {
        b.iter(|| {
            let (a, b) = black_box(x.clone()).split(50.0, Side::Right);
        })
    });
    group.bench_function("finite-left", |b| {
        b.iter(|| {
            let (a, b) = black_box(x.clone()).split(-50.0, Side::Right);
        })
    });
    group.bench_function("finite-right", |b| {
        b.iter(|| {
            let (a, b) = black_box(x.clone()).split(150.0, Side::Right);
        })
    });
}

/*
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
*/

pub fn bench_interval_hull(c: &mut Criterion) {
    let points = [
        5, 300, -300, 32, 44, 83, 93, -1000, 20, 84, 74, -33, 49, 400, 55, 32, -2000, 100, 22, 73,
        1000, 3000, 30, -200, 432, 4000, 300, -3000, 12,
    ];

    let mut group = c.benchmark_group("core::hull");
    group.bench_function("finite-by-value", |b| {
        b.iter(|| {
            let x = black_box(points.clone());
            FiniteInterval::convex_hull(x)
        })
    });

    group.bench_function("finite-by-ref", |b| {
        b.iter(|| {
            let x = black_box(&points);
            FiniteInterval::convex_hull(x)
        })
    });
}

/*
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
*/

criterion_group!(
    benches,
    bench_core_intersects,
    bench_core_merged,
    bench_partial_ord_sort,
    bench_core_split,
    bench_interval_hull,
    //bench_interval_difference,
    //bench_interval_sym_difference,
    bench_core_intersection,
);
criterion_main!(benches);
