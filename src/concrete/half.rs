use crate::ival::{Bound, IVal, Side};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HalfInterval<T> {
    pub(crate) side: Side,
    pub(crate) ival: IVal<T>,
}

impl<T: Copy> HalfInterval<T> {
    pub fn new(side: Side, ival: IVal<T>) -> Self {
        Self { side, ival }
    }

    /// (<-, b) = { x in T | x < b }
    pub fn unbound_open(right: T) -> Self {
        Self::new(Side::Right, IVal::new(Bound::Open, right))
    }

    /// (<-, b] = { x in T | x <= b }
    pub fn unbound_closed(right: T) -> Self {
        Self::new(Side::Right, IVal::new(Bound::Closed, right))
    }

    /// (a, ->) = { x in T | a < x }
    pub fn open_unbound(left: T) -> Self {
        Self::new(Side::Left, IVal::new(Bound::Open, left))
    }

    /// [a, ->) = {x in T | a <= x }
    pub fn closed_unbound(left: T) -> Self {
        Self::new(Side::Left, IVal::new(Bound::Closed, left))
    }

    pub fn lval_unchecked(&self) -> T {
        match self.side {
            Side::Left => self.ival.value,
            Side::Right => panic!("right half interval has no left bound"),
        }
    }

    pub fn rval_unchecked(&self) -> T {
        match self.side {
            Side::Left => panic!("left half interval has no right bound"),
            Side::Right => self.ival.value,
        }
    }
}
