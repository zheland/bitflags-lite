//! Internal utilities for integer operations used by [`BitFlags`].
//!
//! This module is public to support the generic implementations in this crate,
//! but its items are not intended to be used directly.

use core::num::ParseIntError;

/// An extension trait for integer operations used by
/// [`BitFlags`](crate::BitFlags).
///
/// It abstracts over the concrete integer types so that generic code can use
/// the same operations.
///
/// # Bit width limit
///
/// Bit indices are stored as `u8`, so implementations must report a
/// [`BITS`](Self::BITS) value of at most `u8::MAX`.
/// This library therefore supports storage types of up to 256 bits.
/// All primitive integers satisfy this constraint.
pub trait IntOpsExt: Sized {
    /// The value `0`.
    const ZERO: Self;

    /// The value `1`.
    const ONE: Self;

    /// The number of bits in this integer type.
    ///
    /// For primitive integers this equals `size_of::<Self>() * 8`, but
    /// [`IntOpsExt`] implementations must return the number of bits the type
    /// can represent as a bit set, which may be smaller for wrapper types.
    ///
    /// This value is used to bound valid bit indices for
    /// [`BitFlag`](crate::BitFlag) and [`BitFlags`](crate::BitFlags).
    /// It must not exceed `u8::MAX`, because bit indices are stored as `u8`.
    const BITS: u8;

    /// Checked shift left.
    /// Computes `self << rhs`, returning `None` if `rhs` is larger than or
    /// equal to the number of bits in `self`.
    #[must_use]
    fn checked_shl(&self, rhs: u32) -> Option<Self>;

    /// Returns the number of ones in the binary representation of `self`.
    #[must_use]
    fn count_ones(&self) -> u32;

    /// Returns the index of the highest bit set to one in `self`, or `None`
    /// if `self` is `0`.
    #[must_use]
    fn highest_one(&self) -> Option<u32>;

    /// Returns the index of the lowest bit set to one in `self`, or `None`
    /// if `self` is `0`.
    #[must_use]
    fn lowest_one(&self) -> Option<u32>;

    /// Parses an integer from a string slice with digits in a given base.
    ///
    /// This is equivalent to the corresponding inherent method on the
    /// concrete integer type.
    ///
    /// # Errors
    ///
    /// If `src` is not a valid representation of an integer in the given
    /// radix, returns `Err(ParseIntError)`.
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseIntError>;
}

// Const traits are not stable yet, so the const methods are implemented for
// each integer type separately via a macro.
// See: https://github.com/rust-lang/rust/issues/143874
macro_rules! impl_uint_opt_ext {
    ($ty:ty) => {
        impl IntOpsExt for $ty {
            const ZERO: Self = 0;
            const ONE: Self = 1;

            #[expect(
                clippy::as_conversions,
                clippy::cast_possible_truncation,
                reason = "supported in const context"
            )]
            const BITS: u8 = <$ty>::BITS as u8;

            #[inline]
            fn checked_shl(&self, rhs: u32) -> Option<Self> {
                <$ty>::checked_shl(*self, rhs)
            }

            #[inline]
            fn count_ones(&self) -> u32 {
                <$ty>::count_ones(*self)
            }

            #[inline]
            fn lowest_one(&self) -> Option<u32> {
                <$ty>::lowest_one(*self)
            }

            #[inline]
            fn highest_one(&self) -> Option<u32> {
                <$ty>::highest_one(*self)
            }

            #[inline]
            fn from_str_radix(src: &str, radix: u32) -> Result<Self, core::num::ParseIntError> {
                <$ty>::from_str_radix(src, radix)
            }
        }
    };
}

impl_uint_opt_ext!(u8);
impl_uint_opt_ext!(u16);
impl_uint_opt_ext!(u32);
impl_uint_opt_ext!(u64);
impl_uint_opt_ext!(u128);
impl_uint_opt_ext!(usize);

impl_uint_opt_ext!(i8);
impl_uint_opt_ext!(i16);
impl_uint_opt_ext!(i32);
impl_uint_opt_ext!(i64);
impl_uint_opt_ext!(i128);
impl_uint_opt_ext!(isize);
