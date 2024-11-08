use super::adjacent::Adjacent;
use super::RefMerged;
use crate::bound::{FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::numeric::Domain;
use crate::ops::{Contains, Intersects, Merged};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl<T: Domain> Merged<Self> for FiniteInterval<T> {
    type Output = Self;

    fn merged(self, rhs: Self) -> Option<Self::Output> {
        if self.is_disjoint_from(&rhs) && !self.is_adjacent_to(&rhs) {
            if self.is_empty() {
                return Some(rhs);
            } else if rhs.is_empty() {
                return Some(self);
            } else {
                return None;
            }
        }

        let merged = self
            .map(|a_left, a_right| {
                rhs.map(|b_left, b_right| {
                    FiniteInterval::Bounded(
                        FiniteBound::take_min(Side::Left, a_left, b_left),
                        FiniteBound::take_max(Side::Right, a_right, b_right),
                    )
                })
                .expect("Expected to merge sets")
            })
            .expect("Expected to merge sets");

        Some(merged)
    }
}

impl<T: Domain + Clone> RefMerged<Self> for FiniteInterval<T> {
    fn ref_merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.is_disjoint_from(rhs) && !self.is_adjacent_to(rhs) {
            if self.is_empty() {
                return Some(rhs.clone());
            } else if rhs.is_empty() {
                return Some(self.clone());
            } else {
                return None;
            }
        }

        let merged = self
            .ref_map(|a_left, a_right| {
                rhs.ref_map(|b_left, b_right| {
                    FiniteInterval::Bounded(
                        FiniteBound::min(Side::Left, a_left, b_left).clone(),
                        FiniteBound::max(Side::Right, a_right, b_right).clone(),
                    )
                })
                .expect("Expected to merge sets")
            })
            .expect("Expected to merge sets");

        Some(merged)
    }
}

impl<T: Domain> Merged<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn merged(self, rhs: Self) -> Option<Self::Output> {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                Some(self.into())
            } else {
                Some(rhs.into())
            }
        } else {
            // <----](---->
            // <----][---->
            // <----)[---->
            // but not <----)(---->
            if self.contains(rhs.bound.value())
                || rhs.contains(self.bound.value())
                || self.is_adjacent_to(&rhs)
            {
                Some(EnumInterval::Unbounded)
            } else {
                None // disjoint
            }
        }
    }
}

impl<T: Domain + Clone> RefMerged<Self> for HalfInterval<T> {
    fn ref_merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                Some(self.clone().into())
            } else {
                Some(rhs.clone().into())
            }
        } else {
            // <----](---->
            // <----][---->
            // <----)[---->
            // but not <----)(---->
            if self.contains(rhs.bound.value())
                || rhs.contains(self.bound.value())
                || self.is_adjacent_to(rhs)
            {
                Some(EnumInterval::Unbounded)
            } else {
                None // disjoint
            }
        }
    }
}

impl<T: Domain> Merged<FiniteInterval<T>> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn merged(self, rhs: FiniteInterval<T>) -> Option<Self::Output> {
        let n_seen = rhs
            .ref_map(|left, right| {
                [left, right]
                    .into_iter()
                    .filter(|b| self.contains(b.value()))
                    .count()
            })
            .unwrap_or(2);

        if n_seen == 2 {
            Some(self.into())
        } else if n_seen == 0 && !self.is_adjacent_to(&rhs) {
            None
        } else {
            rhs.map(|left, right| {
                let merged = match self.side {
                    Side::Left => HalfInterval::new(self.side, left),
                    Side::Right => HalfInterval::new(self.side, right),
                };

                merged.into()
            })
            .ok()
        }
    }
}

impl<T: Domain + Clone> RefMerged<FiniteInterval<T>> for HalfInterval<T> {
    fn ref_merged(&self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        let n_seen = rhs
            .ref_map(|left, right| {
                [left, right]
                    .into_iter()
                    .filter(|b| self.contains(b.value()))
                    .count()
            })
            .unwrap_or(2);

        if n_seen == 2 {
            Some(self.clone().into())
        } else if n_seen == 0 && !self.is_adjacent_to(rhs) {
            None
        } else {
            rhs.ref_map(|left, right| {
                let merged = match self.side {
                    Side::Left => HalfInterval::new(self.side, left.clone()),
                    Side::Right => HalfInterval::new(self.side, right.clone()),
                };

                merged.into()
            })
            .ok()
        }
    }
}

impl<T: Domain> Merged<HalfInterval<T>> for FiniteInterval<T> {
    type Output = EnumInterval<T>;

    fn merged(self, rhs: HalfInterval<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain + Clone> RefMerged<HalfInterval<T>> for FiniteInterval<T> {
    fn ref_merged(&self, rhs: &HalfInterval<T>) -> Option<Self::Output> {
        rhs.ref_merged(self)
    }
}

impl<T: Domain> Merged<FiniteInterval<T>> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn merged(self, rhs: FiniteInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => lhs.merged(rhs).map(|itv| itv.into()),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain + Clone> RefMerged<FiniteInterval<T>> for EnumInterval<T> {
    fn ref_merged(&self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => lhs.ref_merged(rhs).map(|itv| itv.into()),
            Self::Half(lhs) => lhs.ref_merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<HalfInterval<T>> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn merged(self, rhs: HalfInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => rhs.merged(lhs), // todo: check
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain + Clone> RefMerged<HalfInterval<T>> for EnumInterval<T> {
    fn ref_merged(&self, rhs: &HalfInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => lhs.ref_merged(rhs), // todo: check
            Self::Half(lhs) => lhs.ref_merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<Self> for EnumInterval<T> {
    type Output = Self;

    fn merged(self, rhs: Self) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => rhs.merged(lhs),
            Self::Half(lhs) => rhs.merged(lhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain + Clone> RefMerged<Self> for EnumInterval<T> {
    fn ref_merged(&self, rhs: &Self) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => rhs.ref_merged(lhs),
            Self::Half(lhs) => rhs.ref_merged(lhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<EnumInterval<T>> for FiniteInterval<T> {
    type Output = EnumInterval<T>;

    fn merged(self, rhs: EnumInterval<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain + Clone> RefMerged<EnumInterval<T>> for FiniteInterval<T> {
    fn ref_merged(&self, rhs: &EnumInterval<T>) -> Option<Self::Output> {
        rhs.ref_merged(self)
    }
}

impl<T: Domain> Merged<EnumInterval<T>> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn merged(self, rhs: EnumInterval<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain + Clone> RefMerged<EnumInterval<T>> for HalfInterval<T> {
    fn ref_merged(&self, rhs: &EnumInterval<T>) -> Option<Self::Output> {
        rhs.ref_merged(self)
    }
}
