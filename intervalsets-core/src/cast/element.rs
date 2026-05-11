//! Element-layer cast support.
//!
//! Defines [`LossyCastElement`] impls for every primitive numeric pair.
//! Uses [`az::SaturatingCast`] where `az` provides the right semantics
//! (int↔int with cross-sign saturation; float→int with saturation),
//! and direct `as` casts for the remaining cases (int→float — only
//! rounding loss; float widening — lossless; f64→f32 — `as` produces
//! `±INF` for out-of-range, so we clamp first).
//!
//! There is intentionally **no** element-layer `Cast` blanket over
//! `T: Into<U>`. Such a blanket would overlap with every set-level
//! [`Cast`](super::Cast) impl (Rust can't rule out
//! `FiniteInterval<T>: Into<FiniteInterval<U>>` as a future user
//! impl). The set-level impls bound on `T: Into<U>` directly.
//!
//! There is also intentionally no element-layer `TryCast` blanket. The
//! set-level [`TryCast`](super::TryCast) impls call
//! [`NumCast::from`](num_traits::NumCast::from) directly inside their
//! method bodies.
//!
//! # NaN-into-int panic caveat
//!
//! `az::SaturatingCast::<i*>::saturating_cast(f64::NAN)` panics. For
//! library float types this is unreachable through the validating API
//! because [`Element::validate`](crate::numeric::Element::validate)
//! rejects NaN at construction time. Misuse via the Tier 4
//! `*_assume_valid` bypass can route NaN into [`LossyCast`](super::LossyCast)
//! and reach the panic; that's documented bypass-misuse, not a
//! contract violation.

use num_traits::{NumCast, ToPrimitive};
// =====================================================================
// Sealed `Primitive` marker
// =====================================================================
//
// Marks the std numeric primitive types. Sealed (`pub(crate)` inside a
// private module) so downstream crates cannot extend it. This unlocks
// a single blanket `TryCastElement` impl over all primitive pairs
// without risking coherence conflicts when feature-gated storage types
// (`BigDecimal`, etc.) add their own `TryCastElement` impls — Rust
// proves the blanket cannot apply to those types because they can
// never become `Primitive`.
pub(crate) use sealed::Primitive;

use super::{LossyCastElement, TryCastElement};

mod sealed {
    pub trait Primitive {}
}

macro_rules! mark_primitive {
    ($($t:ty),* $(,)?) => {
        $( impl sealed::Primitive for $t {} )*
    };
}

mark_primitive!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

// =====================================================================
// TryCastElement: primitive-primitive blanket via NumCast
// =====================================================================
//
// Every primitive pair satisfies `ToPrimitive + NumCast`, so one
// blanket covers the 14×14 cross product. Sealed `Primitive` bound
// prevents the blanket from interfering with feat-type impls.

impl<T, U> TryCastElement<U> for T
where
    T: ToPrimitive + Primitive,
    U: NumCast + Primitive,
{
    #[inline]
    fn try_cast_element(self) -> Option<U> {
        <U as NumCast>::from(self)
    }
}

macro_rules! lossy_via_az {
    ($Src:ty => $($Dst:ty),+ $(,)?) => {
        $(
            impl LossyCastElement<$Dst> for $Src {
                #[inline]
                fn lossy_cast_element(self) -> $Dst {
                    <$Src as az::SaturatingCast<$Dst>>::saturating_cast(self)
                }
            }
        )+
    };
}

macro_rules! lossy_as {
    ($Src:ty => $($Dst:ty),+ $(,)?) => {
        $(
            impl LossyCastElement<$Dst> for $Src {
                #[inline]
                fn lossy_cast_element(self) -> $Dst {
                    self as $Dst
                }
            }
        )+
    };
}

// Int → Int — `az` handles cross-sign narrowing without
// sign-extension footguns. Includes identity (e.g. `i32 → i32`).
lossy_via_az!(i8    => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(i16   => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(i32   => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(i64   => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(i128  => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(isize => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(u8    => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(u16   => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(u32   => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(u64   => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(u128  => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(usize => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

// Float → Int — `az` saturating; NaN panics (see module docs).
lossy_via_az!(f32 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
lossy_via_az!(f64 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

// Int → Float — direct `as` (rounding loss only, no overflow notion).
lossy_as!(i8    => f32, f64);
lossy_as!(i16   => f32, f64);
lossy_as!(i32   => f32, f64);
lossy_as!(i64   => f32, f64);
lossy_as!(i128  => f32, f64);
lossy_as!(isize => f32, f64);
lossy_as!(u8    => f32, f64);
lossy_as!(u16   => f32, f64);
lossy_as!(u32   => f32, f64);
lossy_as!(u64   => f32, f64);
lossy_as!(u128  => f32, f64);
lossy_as!(usize => f32, f64);

// f32 → f64 — lossless widening.
lossy_as!(f32 => f64);

// Float identities — useful for generic call sites where T = U.
impl LossyCastElement<f32> for f32 {
    #[inline]
    fn lossy_cast_element(self) -> f32 {
        self
    }
}

impl LossyCastElement<f64> for f64 {
    #[inline]
    fn lossy_cast_element(self) -> f64 {
        self
    }
}

// f64 → f32 — `as f32` of out-of-range f64 produces `±INF`; clamp to
// `[f32::MIN, f32::MAX]` first so the result is finite.
impl LossyCastElement<f32> for f64 {
    #[inline]
    fn lossy_cast_element(self) -> f32 {
        if self.is_nan() {
            f32::NAN
        } else if self > f32::MAX as f64 {
            f32::MAX
        } else if self < f32::MIN as f64 {
            f32::MIN
        } else {
            self as f32
        }
    }
}
