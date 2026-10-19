use core::fmt;
use core::iter::FusedIterator;
use core::ops::{BitAndAssign, Not};

use crate::{BitFlag, BitFlags, util};

/// An iterator over the individual bits set in a [`BitFlags`] value.
///
/// Each item is a [`BitFlag`] value containing exactly one set bit.
/// The iterator yields bits from the least significant bit to the most
/// significant bit, and also supports double-ended iteration.
///
/// The iterator yields all set bits, including invalid bits if the source
/// [`BitFlags`] was created with [`BitFlags::from_bits_retain`] or otherwise
/// contains invalid bits.
///
/// This type is created by the [`IntoIterator`] implementation for
/// [`BitFlags`].
pub struct BitFlagsIter<T, U>(BitFlags<T, U>)
where
    U: ?Sized;

impl<T, U> BitFlagsIter<T, U>
where
    T: Copy + Not<Output = T> + BitAndAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    /// Creates an iterator over the set bits in `flags`.
    #[inline]
    #[must_use]
    pub const fn new(flags: BitFlags<T, U>) -> Self {
        Self(flags)
    }
}

// The iterators should not implement `Copy`.
impl<T, U> Clone for BitFlagsIter<T, U>
where
    T: Clone,
    U: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, U> Default for BitFlagsIter<T, U>
where
    T: util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn default() -> Self {
        Self(BitFlags::empty())
    }
}

impl<T, U> fmt::Debug for BitFlagsIter<T, U>
where
    T: fmt::LowerHex,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BitFlagsIter").field(&self.0).finish()
    }
}

impl<T, U> Iterator for BitFlagsIter<T, U>
where
    T: Copy + Not<Output = T> + BitAndAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    type Item = BitFlag<T, U>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let lowest = self.0.bits().lowest_one()?;

        let Some(mask) = T::ONE.checked_shl(lowest) else {
            #[cfg(debug_assertions)]
            #[expect(clippy::panic, reason = "enabled only for `debug_assertions`")]
            (panic!(
                "invalid implementation of `{}`, checked_shl failed with bit from lowest_one",
                core::any::type_name::<T>()
            ));
            #[cfg(not(debug_assertions))]
            return None;
        };
        *self.0.bits_mut() &= !mask;

        #[expect(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            reason = "types longer than 256 bits are not supported"
        )]
        Some(BitFlag::from_bit_retain(lowest as u8))
    }

    #[expect(clippy::as_conversions, reason = "T::count_ones never exceeds `usize`")]
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.bits().count_ones() as usize;
        (len, Some(len))
    }
}

impl<T, U> DoubleEndedIterator for BitFlagsIter<T, U>
where
    T: Copy + Not<Output = T> + BitAndAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let lowest = self.0.bits().highest_one()?;

        let Some(mask) = T::ONE.checked_shl(lowest) else {
            #[cfg(debug_assertions)]
            #[expect(clippy::panic, reason = "enabled only for `debug_assertions`")]
            (panic!(
                "invalid implementation of `{}`, checked_shl failed with bit from lowest_one",
                core::any::type_name::<T>()
            ));
            #[cfg(not(debug_assertions))]
            return None;
        };
        *self.0.bits_mut() &= !mask;

        #[expect(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            reason = "types longer than 256 bits are not supported"
        )]
        Some(BitFlag::from_bit_retain(lowest as u8))
    }
}

impl<T, U> ExactSizeIterator for BitFlagsIter<T, U>
where
    T: Copy + Not<Output = T> + BitAndAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[expect(clippy::as_conversions, reason = "T::count_ones never exceeds `usize`")]
    #[inline]
    fn len(&self) -> usize {
        self.0.bits().count_ones() as usize
    }
}

impl<T, U> FusedIterator for BitFlagsIter<T, U>
where
    T: Copy + Not<Output = T> + BitAndAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
}
