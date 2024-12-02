use core::ops::Div;

use crate::{EnumInterval, FiniteInterval, HalfInterval};

impl<T> Div for FiniteInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<T> Div for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<T> Div<HalfInterval<T>> for FiniteInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: HalfInterval<T>) -> Self::Output {
        todo!()
    }
}

impl<T> Div<FiniteInterval<T>> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: FiniteInterval<T>) -> Self::Output {
        todo!()
    }
}

impl<T> Div<FiniteInterval<T>> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: FiniteInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs / rhs,
            Self::Half(lhs) => lhs / rhs,
            Self::Unbounded => todo!(),
        }
    }
}

impl<T> Div<HalfInterval<T>> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: HalfInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs / rhs,
            Self::Half(lhs) => lhs / rhs,
            Self::Unbounded => todo!(),
        }
    }
}

impl<T> Div<EnumInterval<T>> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: EnumInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs / rhs,
            Self::Half(lhs) => lhs / rhs,
            Self::Unbounded => match rhs {
                Self::Finite(rhs) => todo!(),
                Self::Half(rhs) => todo!(),
                Self::Unbounded => Self::Unbounded,
            },
        }
    }
}

impl<T> Div<EnumInterval<T>> for FiniteInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: EnumInterval<T>) -> Self::Output {
        match rhs {
            EnumInterval::Finite(rhs) => self / rhs,
            EnumInterval::Half(rhs) => self / rhs,
            EnumInterval::Unbounded => EnumInterval::Unbounded,
        }
    }
}

impl<T> Div<EnumInterval<T>> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn div(self, rhs: EnumInterval<T>) -> Self::Output {
        match rhs {
            EnumInterval::Finite(rhs) => self / rhs,
            EnumInterval::Half(rhs) => self / rhs,
            EnumInterval::Unbounded => EnumInterval::Unbounded,
        }
    }
}

mod impls {}

#[cfg(test)]
mod tests {}
