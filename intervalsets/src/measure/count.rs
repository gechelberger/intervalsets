use core::ops::Add;

use super::{Count, Countable, Measurement};
use crate::numeric::Zero;
use crate::{Interval, IntervalSet};

impl<T> Count for Interval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;

    fn count(&self) -> Measurement<Self::Output> {
        self.0.count()
    }
}

impl<T, Out> Count for IntervalSet<T>
where
    T: Countable<Output = Out>,
    Out: Zero + Clone + Add<Out, Output = Out>,
{
    type Output = Out;

    fn count(&self) -> Measurement<Self::Output> {
        self.iter()
            .map(|subset| subset.count())
            .fold(Measurement::Finite(Out::zero()), |accum, item| accum + item)
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn test_count() {
        //let x: Interval<i64> = Interval::closed(0.0, 10.0);
        //assert_eq!(x.count().finite(), 11);
    }
}
