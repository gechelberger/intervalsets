use super::{mergeable, Merged, Union};
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, StackSet};

impl<T: Domain + Ord> Union<Self> for FiniteInterval<T> {
    type Output = StackSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        if mergeable(&self, &rhs) {
            let merged = self.merged(rhs).expect("Intervals should have merged");
            StackSet::new([merged.into()])
        } else {
            StackSet::new([self.into(), rhs.into()])
        }
    }
}

impl<T: Domain + Ord> Union<Self> for HalfInterval<T> {
    type Output = StackSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        if mergeable(&self, &rhs) {
            let merged = self.merged(rhs).expect("Intervals should have merged");
            StackSet::new([merged])
        } else {
            StackSet::new([self.into(), rhs.into()])
        }
    }
}

impl<T: Domain + Ord> Union<HalfInterval<T>> for FiniteInterval<T> {
    type Output = StackSet<T>;

    fn union(self, rhs: HalfInterval<T>) -> Self::Output {
        if mergeable(&self, &rhs) {
            let merged = self.merged(rhs).expect("Intervals should have merged");
            StackSet::new([merged])
        } else {
            StackSet::new([self.into(), rhs.into()])
        }
    }
}

impl<T: Domain + Ord> Union<FiniteInterval<T>> for EnumInterval<T> {
    type Output = StackSet<T>;

    fn union(self, rhs: FiniteInterval<T>) -> Self::Output {
        match self {
            Self::Unbounded => Self::Unbounded.into(),
            Self::Half(lhs) => lhs.union(rhs),
            Self::Finite(lhs) => lhs.union(rhs),
        }
    }
}

impl<T: Domain + Ord> Union<HalfInterval<T>> for EnumInterval<T> {
    type Output = StackSet<T>;

    fn union(self, rhs: HalfInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.union(rhs),
            Self::Half(lhs) => lhs.union(rhs),
            Self::Unbounded => Self::Unbounded.into(),
        }
    }
}

impl<T: Domain + Ord> Union<Self> for EnumInterval<T> {
    type Output = StackSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        match self {
            Self::Unbounded => Self::Unbounded.into(),
            Self::Half(lhs) => rhs.union(lhs),
            Self::Finite(lhs) => rhs.union(lhs),
        }
    }
}

impl<T: Domain + Ord> Union<FiniteInterval<T>> for HalfInterval<T> {
    type Output = StackSet<T>;
    fn union(self, rhs: FiniteInterval<T>) -> Self::Output {
        rhs.union(self)
    }
}

impl<T: Domain + Ord> Union<EnumInterval<T>> for HalfInterval<T> {
    type Output = StackSet<T>;
    fn union(self, rhs: EnumInterval<T>) -> Self::Output {
        rhs.union(self)
    }
}

impl<T: Domain + Ord> Union<EnumInterval<T>> for FiniteInterval<T> {
    type Output = StackSet<T>;
    fn union(self, rhs: EnumInterval<T>) -> Self::Output {
        rhs.union(self)
    }
}

impl<T: Domain + Ord> Union<FiniteInterval<T>> for StackSet<T> {
    type Output = Self;
    fn union(self, rhs: FiniteInterval<T>) -> Self::Output {
        self.union(EnumInterval::from(rhs))
    }
}

impl<T: Domain + Ord> Union<HalfInterval<T>> for StackSet<T> {
    type Output = Self;
    fn union(self, rhs: HalfInterval<T>) -> Self::Output {
        self.union(EnumInterval::from(rhs))
    }
}

impl<T: Domain + Ord> Union<EnumInterval<T>> for StackSet<T> {
    type Output = Self;
    fn union(self, rhs: EnumInterval<T>) -> Self::Output {
        unsafe {
            Self::new_unchecked(MergeSorted::new(itertools::merge(
                self.into_raw(),
                core::iter::once(rhs),
            )))
        }
    }
}

impl<T: Domain + Ord> Union<Self> for StackSet<T> {
    type Output = Self;
    fn union(self, rhs: Self) -> Self::Output {
        unsafe {
            Self::new_unchecked(MergeSorted::new(itertools::merge(
                self.into_raw(),
                rhs.into_raw(),
            )))
        }
    }
}

impl<T: Domain + Ord> Union<StackSet<T>> for FiniteInterval<T> {
    type Output = StackSet<T>;
    fn union(self, rhs: StackSet<T>) -> Self::Output {
        rhs.union(self)
    }
}

impl<T: Domain + Ord> Union<StackSet<T>> for HalfInterval<T> {
    type Output = StackSet<T>;
    fn union(self, rhs: StackSet<T>) -> Self::Output {
        rhs.union(self)
    }
}

impl<T: Domain + Ord> Union<StackSet<T>> for EnumInterval<T> {
    type Output = StackSet<T>;
    fn union(self, rhs: StackSet<T>) -> Self::Output {
        rhs.union(self)
    }
}

/// todo ...
pub struct MergeSorted<T, I: Iterator<Item = EnumInterval<T>>> {
    sorted: itertools::PutBack<I>,
}

impl<T, I> MergeSorted<T, I>
where
    I: Iterator<Item = EnumInterval<T>>,
{
    fn new<U>(sorted: U) -> Self
    where
        U: IntoIterator<Item = EnumInterval<T>, IntoIter = I>,
    {
        Self {
            sorted: itertools::put_back(sorted),
        }
    }
}

impl<T, I> Iterator for MergeSorted<T, I>
where
    T: Domain,
    I: Iterator<Item = EnumInterval<T>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.sorted.next()?;

        while let Some(candidate) = self.sorted.next() {
            if mergeable(&current, &candidate) {
                current = current.merged(candidate).unwrap();
            } else {
                self.sorted.put_back(candidate);
                break;
            }
        }

        Some(current)
    }
}
