use crate::numeric::Element;
use crate::ops::Connects;
use crate::{EnumInterval, FiniteInterval, HalfInterval, MaybeEmpty};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaybeDisjoint<T> {
    Consumed,
    Connected(EnumInterval<T>),
    Disjoint(EnumInterval<T>, EnumInterval<T>),
}

impl<T> Iterator for MaybeDisjoint<T> {
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut inst = Self::Consumed;
        core::mem::swap(self, &mut inst);
        match inst {
            Self::Consumed => None,
            Self::Connected(interval) => Some(interval),
            Self::Disjoint(lhs, rhs) => {
                let mut put_back = Self::Connected(rhs);
                core::mem::swap(self, &mut put_back);
                Some(lhs)
            }
        }
    }
}

impl<T> MaybeDisjoint<T> {
    pub fn empty() -> Self {
        Self::Consumed
    }

    pub fn expect_interval(self) -> EnumInterval<T> {
        match self {
            Self::Consumed => EnumInterval::empty(),
            Self::Connected(interval) => interval,
            Self::Disjoint(_, _) => panic!("expcted a single connected interval"),
        }
    }
}

impl<T> From<FiniteInterval<T>> for MaybeDisjoint<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::from(EnumInterval::from(value))
    }
}

impl<T> From<HalfInterval<T>> for MaybeDisjoint<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::from(EnumInterval::from(value))
    }
}

impl<T> From<EnumInterval<T>> for MaybeDisjoint<T> {
    fn from(interval: EnumInterval<T>) -> Self {
        if interval.is_empty() {
            Self::Consumed
        } else {
            Self::Connected(interval)
        }
    }
}

impl<T: Element> From<(EnumInterval<T>, EnumInterval<T>)> for MaybeDisjoint<T> {
    fn from(value: (EnumInterval<T>, EnumInterval<T>)) -> Self {
        debug_assert!(!value.0.is_empty());
        debug_assert!(!value.1.is_empty());
        debug_assert!(value.0 < value.1);
        debug_assert!(!value.0.connects(&value.1));
        Self::Disjoint(value.0, value.1)
    }
}
