//! Linker-time canary for the Tier 1 panic-free claims
//! (`Contains`, `Intersects`, `Connects`).
//!
//! Tier 1 promises "cannot panic on any input." The canary therefore
//! exercises every Tier 1 impl with a representative spread of fixtures,
//! including pathological ones that downstream tiers would reject.
//!
//! Sanity-build (no canary; verifies the example compiles):
//!     cargo build --example panic_free_tier1 --release
//!
//! Actual canary (link fails if any annotated impl can panic):
//!     cargo build --example panic_free_tier1 --features panic-free-check --release
//!
//! Every annotated Tier 1 impl must be invoked below — an impl that is
//! never called is dead-code-eliminated, the `no_panic` symbol never
//! resolves, and a function that would have failed the check passes
//! silently.

use core::hint::black_box;

use intervalsets_core::bound::ord::{FiniteOrdBound, OrdBoundPair};
use intervalsets_core::prelude::*;

fn main() {
    let f_normal = FiniteInterval::<i64>::closed(0, 10);
    let f_disjoint = FiniteInterval::<i64>::closed(20, 30);

    let h_left = HalfInterval::<i64>::closed_unbound(0);
    let h_right = HalfInterval::<i64>::unbound_closed(10);

    let e_finite = EnumInterval::<i64>::closed(0, 10);
    let e_half = EnumInterval::<i64>::closed_unbound(5);

    let elem: i64 = 5;

    let fob: FiniteOrdBound<&i64> = h_left.finite_ord_bound();
    let obp: OrdBoundPair<&i64> = OrdBoundPair::from(&f_normal);

    // Contains<&T> for FiniteInterval / HalfInterval / EnumInterval
    let r1 = f_normal.contains(&elem);
    let r2 = h_left.contains(&elem);
    let r3 = e_finite.contains(&elem);

    // Contains<FiniteOrdBound<&T>> for FiniteInterval / HalfInterval / EnumInterval
    let r4 = f_normal.contains(fob);
    let r5 = h_left.contains(fob);
    let r6 = e_finite.contains(fob);

    // Contains<&T> for OrdBoundPair<&T>
    let r7 = obp.contains(&elem);

    // Contains<&FiniteInterval / &HalfInterval / &EnumInterval> for FiniteInterval
    let r8 = f_normal.contains(&f_disjoint);
    let r9 = f_normal.contains(&h_left);
    let r10 = f_normal.contains(&e_finite);

    // …for HalfInterval
    let r11 = h_left.contains(&f_normal);
    let r12 = h_left.contains(&h_right);
    let r13 = h_left.contains(&e_finite);

    // …for EnumInterval
    let r14 = e_finite.contains(&f_normal);
    let r15 = e_finite.contains(&h_left);
    let r16 = e_finite.contains(&e_half);

    // Intersects: hand-written impls
    let i1 = f_normal.intersects(&f_disjoint);
    let i2 = h_left.intersects(&f_normal);
    let i3 = h_left.intersects(&h_right);
    let i4 = e_finite.intersects(&f_normal);
    let i5 = e_finite.intersects(&h_left);
    let i6 = e_finite.intersects(&e_half);
    // Intersects: commutative_predicate_impl-generated symmetric impls
    let i7 = f_normal.intersects(&h_left);
    let i8 = f_normal.intersects(&e_finite);
    let i9 = h_left.intersects(&e_finite);

    // Connects: hand-written impls
    let c1 = f_normal.connects(&f_disjoint);
    let c2 = h_left.connects(&h_right);
    let c3 = f_normal.connects(&h_left);
    let c5 = e_finite.connects(&f_normal);
    let c7 = e_finite.connects(&h_left);
    let c9 = e_finite.connects(&e_half);
    // Connects: commutative_predicate_impl-generated symmetric impls
    let c4 = h_left.connects(&f_normal);
    let c6 = f_normal.connects(&e_finite);
    let c8 = h_left.connects(&e_finite);

    black_box((
        r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16,
        i1, i2, i3, i4, i5, i6, i7, i8, i9,
        c1, c2, c3, c4, c5, c6, c7, c8, c9,
    ));
}
