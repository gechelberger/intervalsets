use crate::traits::empty::MaybeEmpty;

use super::*;

impl<T> MaybeEmpty for Finite<T> {
    fn is_empty(&self) -> bool {
        matches!(self, Finite::Empty)
    }
}

impl<T> MaybeEmpty for BoundCase<T> {
    fn is_empty(&self) -> bool {
        match self {
            Self::Finite(inner) => inner.is_empty(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_maybe_empty() {
        let x = Finite::new(Bound::closed(0), Bound::closed(10));
        assert_eq!(x.is_empty(), false);

        let x = BoundCase::Finite(x);
        assert_eq!(x.is_empty(), false);

        let x = Finite::<i32>::Empty;
        assert_eq!(x.is_empty(), true);

        let x = BoundCase::Finite(x);
        assert_eq!(x.is_empty(), true);

        let x = HalfBounded::new(Side::Left, Bound::closed(0));
        let x = BoundCase::Half(x);
        assert_eq!(x.is_empty(), false);
    }
}
