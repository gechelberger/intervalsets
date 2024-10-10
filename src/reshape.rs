use std::ops::Add;

use crate::interval::Interval;

impl<T: Ord + Copy + Add<Output=T>> Interval<T> {

    pub fn shifted(&self, offset: T) -> Self {
        Self::new_unchecked(
            self.left() + offset,
            self.right() + offset,
        )
    }

    pub fn padded(&self, left: T, right: T) -> Self {
        Self::new_unchecked(
            self.left() - left,
            self.right() + right
        )
    }
}