use core::ops::{BitAnd, Not};

use crate::{BitFlag, BitFlagsSpec, util};

/// Extension trait providing generic, non-const equivalents of the
/// type-specific inherent methods on [`BitFlag`].
///
/// The inherent methods are implemented for each concrete integer type so they
/// can be `const`.
/// This trait exposes the same operations for generic `T`.
pub trait BitFlagExt<T, U>: Sized
where
    U: ?Sized,
{
    /// Creates a validated bit flag for bit index `i`.
    ///
    /// Returns `None` if `i` is not set in
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// The index `i` must be less than the number of bits in the
    /// storage type.
    #[must_use]
    fn try_from_bit(i: u8) -> Option<Self>
    where
        T: PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::UintOpsExt,
        U: BitFlagsSpec<T>;

    /// Creates a validated bit flag from `bits` if exactly one bit is
    /// set and that bit is valid according to
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// Returns `None` if `bits` is zero, has more than one bit set, or
    /// the single set bit is not valid.
    #[must_use]
    fn try_from_bits(bits: T) -> Option<Self>
    where
        T: PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::UintOpsExt,
        U: BitFlagsSpec<T>;

    /// Creates a single-bit flag from `bits` if exactly one bit is set.
    ///
    /// The single set bit is not checked against
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// Returns `None` if `bits` is `0` or has more than one bit set.
    #[must_use]
    fn try_maybe_invalid_from_bits(bits: T) -> Option<Self>
    where
        T: util::UintOpsExt;
}

impl<T, U> BitFlagExt<T, U> for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn try_from_bit(i: u8) -> Option<Self>
    where
        T: PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::UintOpsExt,
        U: BitFlagsSpec<T>,
    {
        let bits = T::ONE.checked_shl(u32::from(i))?;
        if (bits & !U::VALID_BITS_MASK) == T::ZERO {
            Self::try_maybe_invalid_from_bit(i)
        } else {
            None
        }
    }

    #[inline]
    fn try_from_bits(bits: T) -> Option<Self>
    where
        T: PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::UintOpsExt,
        U: BitFlagsSpec<T>,
    {
        if bits.count_ones() == 1
            && let Some(bit) = bits.lowest_one()
            && let Ok(bit) = u8::try_from(bit)
        {
            Self::try_from_bit(bit)
        } else {
            None
        }
    }

    /// Creates a single-bit flag from `bits` if exactly one bit is set.
    ///
    /// Returns `None` if `bits` is `0` or has more than one bit set.
    #[inline]
    fn try_maybe_invalid_from_bits(bits: T) -> Option<Self>
    where
        T: util::UintOpsExt,
    {
        if bits.count_ones() == 1
            && let Some(bit) = bits.lowest_one()
            && let Ok(bit) = u8::try_from(bit)
        {
            Self::try_maybe_invalid_from_bit(bit)
        } else {
            None
        }
    }
}
