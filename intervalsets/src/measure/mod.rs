//! A Measure is a function of a set that gives a comparable size between sets.
//!
//! They must obey the following invariants:
//!
//! ```text
//! Let m(S) be our measure.
//!
//! 1) Monotonicity:
//!     If A is subset of B then m(A) <= m(B)
//!
//! 2) Subadditivity:
//!     If A0, A1, .. An is a countable set of possibly intersecting sets:
//!         m(A0 U A1 .. An) <= Sum { m(Ai) for i in 0..n }
//! ```
//!
//! Some common measures are Cardinality and the Lebesgue measure
//! (which is Width in R1).

pub use intervalsets_core::measure::{Cardinality, Countable, Extent, Width, Widthable};

mod cardinality;
mod width;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extent_ord() {
        assert!(Extent::Finite(10) < Extent::Infinite,);
    }
}
