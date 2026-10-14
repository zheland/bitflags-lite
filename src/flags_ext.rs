use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::{BitFlag, BitFlags, BitFlagsSpec, util};

/// Extension trait providing generic, non-const equivalents of the
/// type-specific inherent methods on [`BitFlags`].
///
/// The inherent methods are implemented for each concrete integer type so they
/// can be `const`.
/// This trait exposes the same operations for generic `T`.
pub trait BitFlagsExt<T, U>: Sized
where
    U: ?Sized,
{
    /// Creates a flag set from a single [`BitFlag`].
    ///
    /// This retains the bit index from `bit_flag` without validating it against
    /// the width of `T` or [`BitFlagsSpec::VALID_BITS_MASK`].
    #[must_use]
    fn from_flag(bit_flag: BitFlag<T, U>) -> Self
    where
        T: util::IntOpsExt;

    /// Creates a flag set with only the bit at index `bit` set.
    ///
    /// The bit index is not validated against
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    /// The resulting value may contain an invalid bit. Use
    /// [`BitFlags::try_from_bit`] or [`BitFlags::from_bits_truncate`] when
    /// validation or truncation is required.
    #[must_use]
    fn from_bit_retain(bit: u8) -> Self
    where
        T: util::IntOpsExt;

    /// Creates a flag set from bit index `bit` if that bit is valid.
    ///
    /// Returns `None` if `bit` is outside the width of `T` or if the resulting
    /// bit is not present in [`BitFlagsSpec::VALID_BITS_MASK`].
    #[must_use]
    fn try_from_bit(bit: u8) -> Option<Self>
    where
        T: Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>;

    /// Creates a flag set from raw bits if all set bits are valid.
    ///
    /// Returns `None` if `bits` contains any bit not present in
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// Use [`BitFlags::from_bits_retain`] to keep invalid bits, or
    /// [`BitFlags::from_bits_truncate`] to clear them.
    #[must_use]
    fn try_from_bits(bits: T) -> Option<Self>
    where
        T: Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>;

    /// Creates a flag set from raw bits, clearing any bits not present in
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// This always succeeds and never stores invalid bits.
    #[must_use]
    fn from_bits_truncate(bits: T) -> Self
    where
        T: BitAnd<T, Output = T>,
        U: BitFlagsSpec<T>;

    /// Clears all bits not present in [`BitFlagsSpec::VALID_BITS_MASK`].
    ///
    /// This leaves `self` unchanged if it already contains only valid bits.
    fn truncate(&mut self)
    where
        T: BitAndAssign<T>,
        U: BitFlagsSpec<T>;

    /// Returns `true` if no bits are set.
    ///
    /// This checks the raw stored bits.
    /// A value containing only invalid bits is not empty.
    #[must_use]
    fn is_empty(&self) -> bool
    where
        T: Copy + PartialEq + util::IntOpsExt;

    #[must_use]
    fn is_valid(&self) -> bool
    where
        T: Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>;

    /// Returns `true` if `self` and `other` have at least one set bit in
    /// common.
    ///
    /// This compares raw bits.
    /// Invalid bits participate in the comparison if they are present.
    #[must_use]
    fn intersects(&self, other: Self) -> bool
    where
        T: Copy + PartialEq + BitAnd<T, Output = T> + util::IntOpsExt;

    /// Returns `true` if all bits set in `other` are also set in `self`.
    ///
    /// This compares raw bits.
    /// Invalid bits participate in the comparison if they are present.
    #[must_use]
    fn contains(&self, other: Self) -> bool
    where
        T: Copy + PartialEq + BitAnd<T, Output = T>;

    /// Inserts all bits from `other`.
    ///
    /// This is a raw-bit operation.
    /// It does not validate or clear invalid bits.
    /// Inserting `other` can introduce invalid bits into `self`.
    fn insert(&mut self, other: Self)
    where
        T: Copy + BitOrAssign<T>;

    /// Removes all bits from `other`.
    ///
    /// This is a raw-bit operation.
    /// It cannot introduce new bits, but it does not clear any other
    /// invalid bits that may already be present.
    fn remove(&mut self, other: Self)
    where
        T: Copy + Not<Output = T> + BitAndAssign<T>;

    /// Toggles all bits from `other`.
    ///
    /// This is a raw-bit operation.
    /// It does not validate or clear invalid bits.
    /// Toggling `other` can introduce invalid bits into `self`.
    fn toggle(&mut self, other: Self)
    where
        T: Copy + BitXorAssign<T>;

    /// Inserts `other` if `value` is `true`, or removes `other` if
    /// `value` is `false`.
    ///
    /// This is a raw-bit operation and follows the same invalid-bit
    /// behavior as [`BitFlags::insert`] and [`BitFlags::remove`].
    fn set(&mut self, other: Self, value: bool)
    where
        T: Copy + Not<Output = T> + BitAndAssign<T> + BitOrAssign<T>;

    /// Returns the intersection of `self` and `other`.
    ///
    /// This is a raw-bit operation.
    /// Invalid bits in either operand are preserved when they are set
    /// in both operands.
    #[must_use]
    fn intersection(self, other: Self) -> Self
    where
        T: Copy + BitAnd<T, Output = T>;

    /// Returns the union of `self` and `other`.
    ///
    /// This is a raw-bit operation.
    /// Invalid bits in either operand are preserved.
    #[must_use]
    fn union(self, other: Self) -> Self
    where
        T: Copy + BitOr<T, Output = T>;

    /// Returns the difference of `self` and `other`.
    ///
    /// The result contains bits that are set in `self` but not in
    /// `other`.
    /// This is a raw-bit operation.
    /// Invalid bits already present in `self` are preserved unless they
    /// are also set in `other`.
    #[must_use]
    fn difference(self, other: Self) -> Self
    where
        T: Copy + Not<Output = T> + BitAnd<T, Output = T>;

    /// Returns the symmetric difference of `self` and `other`.
    ///
    /// The result contains bits that are set in exactly one of `self`
    /// or `other`.
    /// This is a raw-bit operation.
    /// Invalid bits participate and may be preserved.
    #[must_use]
    fn symmetric_difference(self, other: Self) -> Self
    where
        T: Copy + BitXor<T, Output = T>;

    /// Returns the complement of `self`.
    ///
    /// All valid bits not set in `self` become set, and all invalid
    /// bits become unset.
    /// The result always contains only valid bits.
    #[must_use]
    fn complement(self) -> Self
    where
        T: Copy + Not<Output = T> + BitAnd<T, Output = T>,
        U: BitFlagsSpec<T>;
}

impl<T, U> BitFlagsExt<T, U> for BitFlags<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn from_flag(bit_flag: BitFlag<T, U>) -> Self
    where
        T: util::IntOpsExt,
    {
        Self::from_bit_retain(bit_flag.bit())
    }

    #[inline]
    fn from_bit_retain(bit: u8) -> Self
    where
        T: util::IntOpsExt,
    {
        T::ONE
            .checked_shl(u32::from(bit))
            .map_or_else(Self::empty, Self::from_bits_retain)
    }

    #[inline]
    fn try_from_bit(bit: u8) -> Option<Self>
    where
        T: Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        T::ONE
            .checked_shl(u32::from(bit))
            .and_then(Self::try_from_bits)
    }

    #[inline]
    fn try_from_bits(bits: T) -> Option<Self>
    where
        T: Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        (bits & !U::VALID_BITS_MASK == T::ZERO).then(|| Self::from_bits_retain(bits))
    }

    #[inline]
    fn from_bits_truncate(bits: T) -> Self
    where
        T: BitAnd<T, Output = T>,
        U: BitFlagsSpec<T>,
    {
        Self::from_bits_retain(bits & U::VALID_BITS_MASK)
    }

    #[inline]
    fn truncate(&mut self)
    where
        T: BitAndAssign<T>,
        U: BitFlagsSpec<T>,
    {
        *self.bits_mut() &= U::VALID_BITS_MASK;
    }

    #[inline]
    fn is_empty(&self) -> bool
    where
        T: Copy + PartialEq + util::IntOpsExt,
    {
        self.bits() == T::ZERO
    }

    #[inline]
    fn is_valid(&self) -> bool
    where
        T: Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        self.bits() & !U::VALID_BITS_MASK == T::ZERO
    }

    #[inline]
    fn intersects(&self, other: Self) -> bool
    where
        T: Copy + PartialEq + BitAnd<T, Output = T> + util::IntOpsExt,
    {
        (self.bits() & other.bits()) != T::ZERO
    }

    #[inline]
    fn contains(&self, other: Self) -> bool
    where
        T: Copy + PartialEq + BitAnd<T, Output = T>,
    {
        (self.bits() & other.bits()) == other.bits()
    }

    #[inline]
    fn insert(&mut self, other: Self)
    where
        T: Copy + BitOrAssign<T>,
    {
        *self.bits_mut() |= other.bits();
    }

    #[inline]
    fn remove(&mut self, other: Self)
    where
        T: Copy + Not<Output = T> + BitAndAssign<T>,
    {
        *self.bits_mut() &= !other.bits();
    }

    #[inline]
    fn toggle(&mut self, other: Self)
    where
        T: Copy + BitXorAssign<T>,
    {
        *self.bits_mut() ^= other.bits();
    }

    #[inline]
    fn set(&mut self, other: Self, value: bool)
    where
        T: Copy + Not<Output = T> + BitAndAssign<T> + BitOrAssign<T>,
    {
        if value {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }

    #[inline]
    fn intersection(self, other: Self) -> Self
    where
        T: Copy + BitAnd<T, Output = T>,
    {
        Self::from_bits_retain(self.bits() & other.bits())
    }

    #[inline]
    fn union(self, other: Self) -> Self
    where
        T: Copy + BitOr<T, Output = T>,
    {
        Self::from_bits_retain(self.bits() | other.bits())
    }

    #[inline]
    fn difference(self, other: Self) -> Self
    where
        T: Copy + Not<Output = T> + BitAnd<T, Output = T>,
    {
        Self::from_bits_retain(self.bits() & !other.bits())
    }

    #[inline]
    fn symmetric_difference(self, other: Self) -> Self
    where
        T: Copy + BitXor<T, Output = T>,
    {
        Self::from_bits_retain(self.bits() ^ other.bits())
    }

    #[inline]
    fn complement(self) -> Self
    where
        T: Copy + Not<Output = T> + BitAnd<T, Output = T>,
        U: BitFlagsSpec<T>,
    {
        Self::from_bits_retain(!self.bits() & U::VALID_BITS_MASK)
    }
}
