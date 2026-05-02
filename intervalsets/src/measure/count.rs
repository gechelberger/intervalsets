use core::ops::Add;

use super::{Count, Countable, Measurement};
use crate::error::Error;
use crate::numeric::Zero;
use crate::{Interval, IntervalSet};

impl<T> Count for Interval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;

    fn try_count(&self) -> Result<Measurement<Self::Output>, Error> {
        self.0.try_count()
    }
}

impl<T, Out> Count for IntervalSet<T>
where
    T: Countable<Output = Out>,
    Out: Zero + Clone + Add<Out, Output = Out>,
{
    type Output = Out;

    fn try_count(&self) -> Result<Measurement<Self::Output>, Error> {
        self.iter()
            .try_fold(Measurement::Finite(Out::zero()), |accum, subset| {
                Ok(accum + subset.try_count()?)
            })
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
