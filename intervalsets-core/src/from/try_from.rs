use core::fmt;

use crate::{EnumInterval, FiniteInterval, HalfInterval};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ConversionError {
    msg: &'static str,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConversionError: {}", self.msg)
    }
}

impl core::error::Error for ConversionError {}

impl<T> TryFrom<EnumInterval<T>> for FiniteInterval<T> {
    type Error = ConversionError;

    fn try_from(value: EnumInterval<T>) -> Result<Self, Self::Error> {
        match value {
            EnumInterval::Finite(inner) => Ok(inner),
            _ => Err(ConversionError {
                msg: "EnumInterval => FiniteInterval",
            }),
        }
    }
}

impl<T> TryFrom<EnumInterval<T>> for HalfInterval<T> {
    type Error = ConversionError;

    fn try_from(value: EnumInterval<T>) -> Result<Self, Self::Error> {
        match value {
            EnumInterval::Half(inner) => Ok(inner),
            _ => Err(ConversionError {
                msg: "EnumInterval => HalfInterval",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_try_from_enum_interval() -> Result<(), ConversionError> {
        let finite = EnumInterval::closed(0, 10);
        assert_eq!(FiniteInterval::closed(0, 10), finite.try_into()?);
        assert!(matches!(HalfInterval::try_from(finite), Err(_)));

        let half = EnumInterval::closed_unbound(0.0);
        assert_eq!(HalfInterval::closed_unbound(0.0), half.try_into()?);
        assert!(matches!(FiniteInterval::try_from(half), Err(_)));

        Ok(())
    }
}
