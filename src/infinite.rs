use crate::finite::Interval;
use crate::ival::{Bound, IVal, Side};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HalfInterval<T> {
    side: Side,
    ival: IVal<T>,
}

/// (a, ->) = Left  { x in T | a <  x      }
/// [a, ->) = Left  { x in T | a <= x      }
/// (<-, b) = Right { x in T |      x < b  }
/// (<-, b] = Right { x in T |      x <= b }
impl<T: Ord> HalfInterval<T> {
    fn new(side: Side, ival: IVal<T>) -> Self {
        Self { side, ival }
    }

    fn contains(&self, value: T) -> bool {
        self.ival.contains(self.side, value)
    }
}

pub enum IntervalExt<T> {
    Finite(Interval<T>),
    HalfFinite(Side, IVal<T>),
    Infinite,
}
