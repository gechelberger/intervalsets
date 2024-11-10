use core::cmp::Ordering::*;

pub trait TryMin: Sized {
    fn try_min(self, rhs: Self) -> Option<Self>;
}

pub trait TryMax: Sized {
    fn try_max(self, rhs: Self) -> Option<Self>;
}

impl<T: PartialOrd> TryMin for T {
    fn try_min(self, rhs: Self) -> Option<Self> {
        match self.partial_cmp(&rhs)? {
            Less | Equal => Some(self),
            Greater => Some(rhs),
        }
    }
}

impl<T: PartialOrd> TryMax for T {
    fn try_max(self, rhs: Self) -> Option<Self> {
        match self.partial_cmp(&rhs)? {
            Greater | Equal => Some(self),
            Less => Some(rhs),
        }
    }
}
