#[cfg(feature = "rust_decimal")]
mod decimal;

#[cfg(feature = "arbitrary")]
mod arbitrary;

#[cfg(feature = "ordered-float")]
mod ordfloat;

#[cfg(feature = "num-bigint")]
mod bigint;

#[cfg(feature = "bigdecimal")]
mod bigdecimal;

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "rkyv")]
mod rkyv;

#[cfg(feature = "quickcheck")]
mod quickcheck;

#[cfg(feature = "fixed")]
mod fixed;
