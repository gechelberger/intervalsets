use arbitrary::Arbitrary;
use intervalsets_core::ops::Intersects;
use intervalsets_core::EnumInterval;

use crate::numeric::{Element, Zero};
use crate::ops::Union;
use crate::{Interval, IntervalSet};

impl<'a, T: Element + Zero + Arbitrary<'a>> Arbitrary<'a> for Interval<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let enum_interval = EnumInterval::arbitrary(u)?;
        Ok(Interval::from(enum_interval))
    }
}

impl<'a, T: Element + Zero + Arbitrary<'a>> Arbitrary<'a> for IntervalSet<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut iset = IntervalSet::empty();
        let n = usize::arbitrary(u)? % 20;
        for _ in 0..n {
            let interval = Interval::<T>::arbitrary(u)?;
            if iset.is_disjoint_from(&interval) {
                iset = iset.union(interval);
            }
        }

        Ok(iset)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;
    use crate::ops::Complement;

    fn unstructured_data(n: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..n).map(|_| rng.gen::<u8>()).collect()
    }

    #[test]
    fn test_interval() {
        // arbitrary skips invalid bound combinations (NaN/INF f32) by
        // returning Err; loop until we hit `iterations` successful runs
        // or exhaust the buffer.
        let data = unstructured_data(65_536);
        let mut u = arbitrary::Unstructured::new(&data);

        let mut iterations = 0;
        while iterations < 100 {
            match Interval::<f32>::arbitrary(&mut u) {
                Ok(interval) => {
                    assert_eq!(
                        interval.complement().complement().expect_interval(),
                        interval
                    );
                    iterations += 1;
                }
                Err(_) => {
                    if u.is_empty() {
                        break;
                    }
                }
            }
        }
        assert!(iterations > 0, "no successful iterations");
    }

    #[test]
    fn test_interval_set() {
        // Larger buffer + skip-on-reject: arbitrary now skips invalid
        // bound combinations (NaN/INF f32 inputs) rather than producing
        // them, which costs entropy.
        let data = unstructured_data(65_536);
        let mut u = arbitrary::Unstructured::new(&data);

        let mut iterations = 0;
        while iterations < 100 {
            match IntervalSet::<f32>::arbitrary(&mut u) {
                Ok(iset) => {
                    assert!(IntervalSet::satisfies_invariants(iset.slice()));
                    assert_eq!(iset.clone().complement().complement(), iset);
                    iterations += 1;
                }
                Err(_) => {
                    if u.is_empty() {
                        break;
                    }
                }
            }
        }
        assert!(iterations > 0, "no successful iterations");
    }
}
