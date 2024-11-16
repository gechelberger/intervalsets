#[cfg(feature = "rust_decimal")]
pub mod decimal;

#[cfg(feature = "arbitrary")]
pub mod arbitrary;

#[cfg(feature = "ordered-float")]
pub mod ordfloat;

#[cfg(feature = "num-bigint")]
pub mod bigint;

#[cfg(feature = "bigdecimal")]
pub mod bigdecimal;
