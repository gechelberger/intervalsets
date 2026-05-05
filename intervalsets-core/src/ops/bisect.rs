use core::convert::Infallible;

use crate::FiniteInterval;

pub enum BisectionError {
    InfiniteBoundError, // todo thiserror
}

pub trait Bisect<T>: Sized {
    type Output;

    type Error: core::error::Error;

    fn bisect_left(self) -> Result<Self::Output, Self::Error>;

    fn bisect_right(self) -> Result<Self::Output, Self::Error>;
}

impl<T> Bisect<T> for FiniteInterval<T> {
    type Output = Self;
    type Error = Infallible;

    fn bisect_left(self) -> Result<Self::Output, Self::Error> {
        match self.into_raw() {
            None => Ok(Self::empty()),
            Some((lhs, rhs)) => {
                //let mid = lhs + (rhs - lhs) / 2;
                unreachable!()
            }
        }
    }

    fn bisect_right(self) -> Result<Self::Output, Self::Error> {
        unreachable!()
    }
}
