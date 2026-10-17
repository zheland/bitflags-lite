use core::ops::{BitAnd, BitOr, BitXor, Not};

use crate::{BitFlag, BitFlags, BitFlagsExt, BitFlagsSpec, util};

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
    /// Creates a bit flag from a [`BitFlags`] set.
    ///
    /// Returns `Some` if `flags` contains exactly one set bit and that bit is
    /// valid according to [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// Returns `None` if `flags` is empty, has more than one bit set, or the
    /// single set bit is not valid.
    ///
    /// This is equivalent to [`BitFlag::try_from_bits`] on `flags.bits()`.
    #[must_use]
    fn try_from_flags(flags: BitFlags<T, U>) -> Option<Self>
    where
        T: 'static + Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>;

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
        T: 'static + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
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
        T: 'static + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>;

    /// Creates a single-bit flag from `bits` if exactly one bit is set.
    ///
    /// The single set bit is not checked against
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// Returns `None` if `bits` is `0` or has more than one bit set.
    #[must_use]
    fn try_from_bits_retain(bits: T) -> Option<Self>
    where
        T: util::IntOpsExt;

    /// Returns `true` if this flag refers to a valid bit.
    ///
    /// A flag is valid when its bit index is within the width of `T` and the
    /// corresponding bit is present in [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// [`BitFlag::from_bit_retain`] does not validate the bit index, so a
    /// `BitFlag` constructed through it may be invalid.
    #[must_use]
    fn is_valid(&self) -> bool
    where
        T: 'static + Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>;

    /// Returns the single-bit mask for this flag, or `None` if the bit index is
    /// outside the width of `T`.
    ///
    /// The mask is `1 << bit`. It is not checked against
    /// [`BitFlagsSpec::VALID_BITS_MASK`]; use [`BitFlag::is_valid`] when tag
    /// validity is required.
    #[must_use]
    fn mask(&self) -> Option<T>
    where
        T: util::IntOpsExt;

    /// Returns the single-bit mask for this flag, or `0` if the bit index is
    /// outside the width of `$ty`.
    ///
    /// This is the const-friendly equivalent of `self.mask().unwrap_or(0)`.
    #[must_use]
    fn mask_or_empty(&self) -> T
    where
        T: util::IntOpsExt;

    /// Returns the intersection of `self` and `other`.
    ///
    /// This is a raw-bit operation.
    /// Invalid bits in either operand are preserved when they are set
    /// in both operands.
    #[must_use]
    fn intersection(self, other: Self) -> BitFlags<T, U>
    where
        T: Copy + BitAnd<T, Output = T> + util::IntOpsExt;

    /// Returns the union of `self` and `other`.
    ///
    /// This is a raw-bit operation.
    /// Invalid bits in either operand are preserved.
    #[must_use]
    fn union(self, other: Self) -> BitFlags<T, U>
    where
        T: Copy + BitOr<T, Output = T> + util::IntOpsExt;

    /// Returns the difference of `self` and `other`.
    ///
    /// The result contains bits that are set in `self` but not in
    /// `other`.
    /// This is a raw-bit operation.
    /// Invalid bits already present in `self` are preserved unless they
    /// are also set in `other`.
    #[must_use]
    fn difference(self, other: Self) -> BitFlags<T, U>
    where
        T: Copy + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt;

    /// Returns the symmetric difference of `self` and `other`.
    ///
    /// The result contains bits that are set in exactly one of `self`
    /// or `other`.
    /// This is a raw-bit operation.
    /// Invalid bits participate and may be preserved.
    #[must_use]
    fn symmetric_difference(self, other: Self) -> BitFlags<T, U>
    where
        T: Copy + BitXor<T, Output = T> + util::IntOpsExt;

    /// Returns the complement of `self`.
    ///
    /// All valid bits not set in `self` become set, and all invalid
    /// bits become unset.
    /// The result always contains only valid bits.
    #[must_use]
    fn complement(self) -> BitFlags<T, U>
    where
        T: 'static + Copy + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>;
}

impl<T, U> BitFlagExt<T, U> for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn try_from_flags(flags: BitFlags<T, U>) -> Option<Self>
    where
        T: 'static + Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        Self::try_from_bits(flags.bits())
    }

    #[inline]
    fn try_from_bit(i: u8) -> Option<Self>
    where
        T: 'static + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        let bits = T::ONE.checked_shl(u32::from(i))?;
        ((i < T::BITS) && (bits & !U::VALID_BITS_MASK) == T::ZERO).then(|| Self::from_bit_retain(i))
    }

    #[inline]
    fn try_from_bits(bits: T) -> Option<Self>
    where
        T: 'static + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
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

    #[inline]
    fn try_from_bits_retain(bits: T) -> Option<Self>
    where
        T: util::IntOpsExt,
    {
        if bits.count_ones() == 1
            && let Some(bit) = bits.lowest_one()
            && let Ok(bit) = u8::try_from(bit)
        {
            Some(Self::from_bit_retain(bit))
        } else {
            None
        }
    }

    #[inline]
    fn is_valid(&self) -> bool
    where
        T: 'static + Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        (self.bit() < T::BITS) && BitFlags::<T, U>::from_flag(*self).is_valid()
    }

    #[inline]
    fn mask(&self) -> Option<T>
    where
        T: util::IntOpsExt,
    {
        T::ONE.checked_shl(u32::from(self.bit()))
    }

    #[inline]
    fn mask_or_empty(&self) -> T
    where
        T: util::IntOpsExt,
    {
        self.mask().unwrap_or(T::ZERO)
    }

    #[inline]
    fn intersection(self, other: Self) -> BitFlags<T, U>
    where
        T: Copy + BitAnd<T, Output = T> + util::IntOpsExt,
    {
        <BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(self)
            .intersection(<BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(other))
    }

    #[inline]
    fn union(self, other: Self) -> BitFlags<T, U>
    where
        T: Copy + BitOr<T, Output = T> + util::IntOpsExt,
    {
        <BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(self)
            .union(<BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(other))
    }

    #[inline]
    fn difference(self, other: Self) -> BitFlags<T, U>
    where
        T: Copy + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
    {
        <BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(self)
            .difference(<BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(other))
    }

    #[inline]
    fn symmetric_difference(self, other: Self) -> BitFlags<T, U>
    where
        T: Copy + BitXor<T, Output = T> + util::IntOpsExt,
    {
        <BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(self)
            .symmetric_difference(<BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(other))
    }

    #[inline]
    fn complement(self) -> BitFlags<T, U>
    where
        T: 'static + Copy + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        <BitFlags<T, U> as BitFlagsExt<T, U>>::from_flag(self).complement()
    }
}
