//! Lightweight generics-based bitflags.
//!
//! This crate provides [`BitFlags`], a `#[repr(transparent)]` wrapper around an
//! unsigned integer that adds a compile-time tag type. The tag type
//! distinguishes flag sets that share the same storage type and defines which
//! bits are valid via the [`BitFlagsSpec`] trait.
//!
//! The crate is `no_std` and `no_alloc`. Optional `serde` and `bytemuck`
//! support can be enabled through features.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(test, not(doc)))]
mod integration_tests_deps {
    use version_sync as _;
}

use core::any::type_name;
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::hint::unreachable_unchecked;
use core::iter::{FromIterator, FusedIterator, IntoIterator};
use core::marker::PhantomData;
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign,
};

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "serde")]
use serde::de::Visitor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod private {
    /// Prevents downstream implementations of [`Bool`](crate::Bool).
    pub trait Sealed {}
}

/// Associates a bit-storage type `T` with the set of bits that are valid for a
/// particular [`BitFlags`] tag.
///
/// Implement this trait for the tag type `U` in `BitFlags<T, U>`.
/// The constants define which bit patterns are considered empty and which bits
/// are valid.
///
/// # Examples
///
/// ```
/// use bitflags_lite::{BitFlags, BitFlagsSpec};
///
/// struct PermissionsTag;
///
/// impl BitFlagsSpec<u32> for PermissionsTag {
///     const VALID_BITS: u32 = (1 << 3) - 1;
///     type AnyBitsAllowed = bitflags_lite::False;
/// }
///
/// type Permissions = BitFlags<u32, PermissionsTag>;
///
/// const READ: Permissions = Permissions::from_bits(1 << 0).unwrap();
/// const WRITE: Permissions = Permissions::from_bits(1 << 1).unwrap();
/// const EXEC: Permissions = Permissions::from_bits(1 << 2).unwrap();
///
/// assert_eq!(Permissions::from_bits(0b1), Some(READ));
/// assert_eq!(Permissions::from_bits(0b11), Some(READ | WRITE));
/// assert_eq!(Permissions::from_bits(0b111), Some(READ | WRITE | EXEC));
/// assert_eq!(Permissions::from_bits(0b1111), None);
/// ```
pub trait BitFlagsSpec<T> {
    /// The bit pattern that contains all valid bits for this flag set.
    const VALID_BITS: T;

    /// Controls whether arbitrary bit patterns may be stored in [`BitFlags`].
    ///
    /// Set this associated type to [`True`] to make
    /// [`BitFlags::from_bits_retain`] available without checking against
    /// [`BitFlagsSpec::VALID_BITS`]; set it to [`False`] otherwise.
    type AnyBitsAllowed: Bool;
}

/// A type-safe set of flags stored in an unsigned integer `T`.
///
/// The second type parameter `U` is a tag type that distinguishes flag sets
/// that share the same storage type. It also implements [`BitFlagsSpec`] to
/// define which bits are valid for this flag set.
///
/// `BitFlags` is `#[repr(transparent)]` over `T`, so it has the same layout as
/// `T`.
///
/// # Examples
///
/// ```
/// use bitflags_lite::{BitFlags, BitFlagsSpec};
///
/// struct PermissionsTag;
///
/// impl BitFlagsSpec<u32> for PermissionsTag {
///     const VALID_BITS: u32 = (1 << 3) - 1;
///     type AnyBitsAllowed = bitflags_lite::False;
/// }
///
/// type Permissions = BitFlags<u32, PermissionsTag>;
///
/// const READ: Permissions = Permissions::from_bits(1 << 0).unwrap();
/// const WRITE: Permissions = Permissions::from_bits(1 << 1).unwrap();
/// const EXEC: Permissions = Permissions::from_bits(1 << 2).unwrap();
///
/// let rw = READ | WRITE;
/// assert!(rw.contains(READ));
/// assert!(rw.contains(WRITE));
/// assert!(!rw.contains(EXEC));
/// ```
#[repr(transparent)]
pub struct BitFlags<T, U>(T, PhantomData<fn() -> *const U>)
where
    U: ?Sized;

/// An iterator over the individual bits set in a [`BitFlags`] value.
///
/// Each item is a `BitFlags` value containing exactly one set bit.
/// The iterator yields bits from the least significant bit to the most
/// significant bit.
///
/// This type is created by the [`IntoIterator`] implementation for
/// [`BitFlags`].
pub struct BitFlagsIter<T, U>(BitFlags<T, U>)
where
    U: ?Sized;

/// A compile-time boolean used by [`BitFlagsSpec::AnyBitsAllowed`].
///
/// This is a workaround for currently unstable:
/// - `generic_const_exprs`, see <https://github.com/rust-lang/rust/issues/76560>;
/// - `gca_min_const_items`, see <https://github.com/rust-lang/rust/issues/132980>.
pub trait Bool: private::Sealed {
    /// The boolean value represented by this type.
    const VALUE: bool;
}

/// [`Bool`] value representing `true`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct True;

/// [`Bool`] value representing `false`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct False;

impl Bool for True {
    const VALUE: bool = true;
}

impl Bool for False {
    const VALUE: bool = false;
}

impl private::Sealed for True {}
impl private::Sealed for False {}

/// Serde visitor for deserializing [`BitFlags`] from a binary string.
#[cfg(feature = "serde")]
struct BitFlagsVisitor<T, U>(PhantomData<T>, PhantomData<fn() -> *const U>)
where
    U: ?Sized;

macro_rules! impl_bit_flags {
    ($ty:ty) => {
        impl private::Sealed for $ty {}

        impl<U> BitFlags<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty, AnyBitsAllowed = True>,
        {
            /// Creates a flag set from raw bits without validation.
            ///
            /// Available only when `U` allows arbitrary bits, i.e. when its
            /// [`AnyBitsAllowed`](BitFlagsSpec::AnyBitsAllowed) associated type
            /// is [`True`].
            #[inline]
            pub const fn from_bits_retain(bits: $ty) -> Self {
                Self(bits, PhantomData)
            }
        }

        impl<U> BitFlags<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            /// Creates a flag set with no bits set (zero bits).
            #[inline]
            pub const fn empty() -> Self {
                Self(0, PhantomData)
            }

            /// Creates a flag set containing all valid bits for this tag.
            #[inline]
            #[must_use]
            pub const fn all_valid() -> Self {
                Self(U::VALID_BITS, PhantomData)
            }

            /// Creates a flag set from raw bits if all set bits are valid.
            ///
            /// Returns `None` if `bits` contains any bit not present in
            /// [`BitFlagsSpec::VALID_BITS`].
            #[inline]
            pub const fn from_bits(bits: $ty) -> Option<Self> {
                if bits & !U::VALID_BITS == 0 {
                    Some(Self(bits, PhantomData))
                } else {
                    None
                }
            }

            /// Creates a flag set from raw bits, clearing any bits not present in
            /// [`BitFlagsSpec::VALID_BITS`].
            #[inline]
            pub const fn from_bits_truncate(bits: $ty) -> Self {
                Self(bits & U::VALID_BITS, PhantomData)
            }

            /// Creates a flag set from raw bits, without checking if the bits
            /// are valid.
            ///
            /// # Safety
            ///
            /// If `AnyBitsAllowed` if `False`, than the bits passes must be
            /// only subset of [`BitFlagsSpec::VALID_BITS`].
            #[inline]
            pub const unsafe fn from_bits_unchecked(bits: $ty) -> Self {
                Self(bits, PhantomData)
            }

            /// Clears all bits not present in [`BitFlagsSpec::VALID_BITS`].
            #[inline]
            #[must_use]
            pub const fn truncate(self) -> Self {
                if U::AnyBitsAllowed::VALUE {
                    Self(self.0 & U::VALID_BITS, PhantomData)
                } else {
                    self
                }
            }

            /// Returns the raw bits stored in this flag set.
            #[inline]
            pub const fn bits(self) -> $ty {
                self.hint_possible_invariants();
                self.0
            }

            /// Returns the mutable reference to raw bits stored in this flag
            /// set.
            ///
            /// # Safety
            ///
            /// If `AnyBitsAllowed` if `False`, than the bits must always remain
            /// a subset of [`BitFlagsSpec::VALID_BITS`].
            #[inline]
            pub const unsafe fn bits_mut(&mut self) -> &mut $ty {
                &mut self.0
            }

            /// Returns `true` if no bits are set.
            #[inline]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            /// Returns `true` if `self` and `other` have at least one set bit in
            /// common.
            #[inline]
            pub const fn intersects(&self, other: Self) -> bool {
                (self.0 & other.0) != 0
            }

            /// Returns `true` if all bits set in `other` are also set in `self`.
            #[inline]
            pub const fn contains(&self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }

            /// Inserts all bits from `other`.
            #[inline]
            pub const fn insert(&mut self, other: Self) {
                self.0 |= other.0;
            }

            /// Removes all bits from `other`.
            #[inline]
            pub const fn remove(&mut self, other: Self) {
                self.0 &= !other.0;
            }

            /// Toggles all bits from `other`.
            #[inline]
            pub const fn toggle(&mut self, other: Self) {
                self.0 ^= other.0;
            }

            /// Inserts `other` if `value` is `true`, or removes `other` if
            /// `value` is `false`.
            #[inline]
            pub const fn set(&mut self, other: Self, value: bool) {
                if value {
                    self.insert(other)
                } else {
                    self.remove(other)
                }
            }

            /// Returns the intersection of `self` and `other`.
            #[inline]
            #[must_use]
            pub const fn intersection(self, other: Self) -> Self {
                Self(self.0 & other.0, PhantomData)
            }

            /// Returns the union of `self` and `other`.
            #[inline]
            #[must_use]
            pub const fn union(self, other: Self) -> Self {
                Self(self.0 | other.0, PhantomData)
            }

            /// Returns the difference of `self` and `other`.
            ///
            /// The result contains bits that are set in `self` but not in
            /// `other`.
            #[inline]
            #[must_use]
            pub const fn difference(self, other: Self) -> Self {
                Self(self.0 & !other.0, PhantomData)
            }

            /// Returns the symmetric difference of `self` and `other`.
            ///
            /// The result contains bits that are set in exactly one of `self`
            /// or `other`.
            #[inline]
            #[must_use]
            pub const fn symmetric_difference(self, other: Self) -> Self {
                Self(self.0 ^ other.0, PhantomData)
            }

            /// Returns the complement of `self`.
            ///
            /// All bits not set in `self` become set.
            #[inline]
            #[must_use]
            pub const fn complement(self) -> Self {
                Self(!self.0 & U::VALID_BITS, PhantomData)
            }

            // Informs the compiler that some values are impossible,
            // possibly enabling further optimizations.
            #[inline]
            const fn hint_possible_invariants(&self) {
                if !U::AnyBitsAllowed::VALUE && (self.0 & !U::VALID_BITS != 0) {
                    // SAFETY: `BitFlags` must not contain invalid invariants.
                    unsafe { unreachable_unchecked() };
                }
            }
        }

        impl<U> Default for BitFlags<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            #[inline]
            fn default() -> Self {
                Self(0, PhantomData)
            }
        }

        impl<U> From<$ty> for BitFlags<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty, AnyBitsAllowed = True>,
        {
            #[inline]
            fn from(other: $ty) -> Self {
                Self(other, PhantomData)
            }
        }

        impl<U> From<BitFlags<$ty, U>> for $ty
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            #[inline]
            fn from(flags: BitFlags<$ty, U>) -> Self {
                flags.hint_possible_invariants();
                flags.0
            }
        }

        impl<U> IntoIterator for BitFlags<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            type IntoIter = BitFlagsIter<$ty, U>;
            type Item = BitFlags<$ty, U>;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                BitFlagsIter(self)
            }
        }

        impl<U> Iterator for BitFlagsIter<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            type Item = BitFlags<$ty, U>;

            #[expect(
                clippy::arithmetic_side_effects,
                reason = "never result in side-effects"
            )]
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.hint_possible_invariants();
                let lowest = self.0.0.isolate_lowest_one();
                if lowest == 0 {
                    None
                } else {
                    self.0.0 -= lowest;
                    Some(BitFlags(lowest, PhantomData))
                }
            }

            #[expect(clippy::as_conversions, reason = "T::count_ones never exceeds `usize`")]
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0.hint_possible_invariants();
                let len = self.0.0.count_ones() as usize;
                (len, Some(len))
            }
        }

        impl<U> DoubleEndedIterator for BitFlagsIter<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "never result in side-effects"
            )]
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.hint_possible_invariants();
                let highest = self.0.0.isolate_highest_one();
                if highest == 0 {
                    None
                } else {
                    self.0.0 -= highest;
                    Some(BitFlags(highest, PhantomData))
                }
            }
        }

        impl<U> ExactSizeIterator for BitFlagsIter<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            #[expect(clippy::as_conversions, reason = "T::count_ones never exceeds `usize`")]
            #[inline]
            fn len(&self) -> usize {
                self.0.hint_possible_invariants();
                self.0.0.count_ones() as usize
            }
        }

        impl<U> FusedIterator for BitFlagsIter<$ty, U> where U: ?Sized + BitFlagsSpec<$ty> {}

        #[cfg(feature = "serde")]
        impl<U> Serialize for BitFlags<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.collect_str(&self)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de, U> Deserialize<'de> for BitFlags<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(BitFlagsVisitor::<$ty, U>(PhantomData, PhantomData))
            }
        }

        #[cfg(feature = "serde")]
        impl<U> Visitor<'_> for BitFlagsVisitor<$ty, U>
        where
            U: ?Sized + BitFlagsSpec<$ty>,
        {
            type Value = BitFlags<$ty, U>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a binary string like \"0b100101\" or \"100101\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value = value.strip_prefix("0b").unwrap_or(value);
                let bits = <$ty>::from_str_radix(value, 2).map_err(E::custom)?;
                if U::AnyBitsAllowed::VALUE || bits & !U::VALID_BITS == 0 {
                    Ok(BitFlags(bits, PhantomData))
                } else {
                    Err(E::custom("invalid bit pattern"))
                }
            }
        }
    };
}

impl_bit_flags!(u8);
impl_bit_flags!(u16);
impl_bit_flags!(u32);
impl_bit_flags!(u64);
impl_bit_flags!(u128);
impl_bit_flags!(usize);

impl<T, U> BitOr for BitFlags<T, U>
where
    T: BitOr<T, Output = T> + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0, PhantomData)
    }
}

impl<T, U> BitOrAssign for BitFlags<T, U>
where
    T: BitOrAssign + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl<T, U> BitAnd for BitFlags<T, U>
where
    T: BitAnd<T, Output = T> + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0, PhantomData)
    }
}

impl<T, U> BitAndAssign for BitFlags<T, U>
where
    T: BitAndAssign + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl<T, U> BitXor for BitFlags<T, U>
where
    T: BitXor<T, Output = T> + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0, PhantomData)
    }
}

impl<T, U> BitXorAssign for BitFlags<T, U>
where
    T: BitXorAssign + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl<T, U> Sub for BitFlags<T, U>
where
    T: Not<Output = T> + BitAnd<T, Output = T> + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0, PhantomData)
    }
}

impl<T, U> SubAssign for BitFlags<T, U>
where
    T: Not<Output = T> + BitAndAssign + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.0;
    }
}

impl<T, U> Not for BitFlags<T, U>
where
    T: Not<Output = T> + BitAnd<T, Output = T> + private::Sealed,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Self(!self.0 & U::VALID_BITS, PhantomData)
    }
}

impl<T, U> Copy for BitFlags<T, U>
where
    T: Copy,
    U: ?Sized + BitFlagsSpec<T>,
{
}

impl<T, U> Clone for BitFlags<T, U>
where
    T: Clone,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T, U> PartialEq for BitFlags<T, U>
where
    T: PartialEq,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, U> Eq for BitFlags<T, U>
where
    T: Eq,
    U: ?Sized + BitFlagsSpec<T>,
{
}

impl<T, U> Ord for BitFlags<T, U>
where
    T: Ord,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T, U> PartialOrd for BitFlags<T, U>
where
    T: PartialOrd,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T, U> Hash for BitFlags<T, U>
where
    T: Hash,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T, U> fmt::Display for BitFlags<T, U>
where
    T: fmt::Binary,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#b}", self.0)?;
        Ok(())
    }
}

impl<T, U> fmt::Debug for BitFlags<T, U>
where
    T: fmt::Binary,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitFlags({:#b}, PhantomData<{}>)",
            self.0,
            type_name::<U>()
        )?;
        Ok(())
    }
}

impl<T, U> fmt::Binary for BitFlags<T, U>
where
    T: fmt::Binary,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl<T, U> fmt::Octal for BitFlags<T, U>
where
    T: fmt::Octal,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl<T, U> fmt::LowerHex for BitFlags<T, U>
where
    T: fmt::LowerHex,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl<T, U> fmt::UpperHex for BitFlags<T, U>
where
    T: fmt::UpperHex,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl<T, U> AsRef<T> for BitFlags<T, U>
where
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T, U> AsMut<T> for BitFlags<T, U>
where
    U: ?Sized + BitFlagsSpec<T, AnyBitsAllowed = True>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T, U> Borrow<T> for BitFlags<T, U>
where
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T, U> BorrowMut<T> for BitFlags<T, U>
where
    U: ?Sized + BitFlagsSpec<T, AnyBitsAllowed = True>,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T, U> Extend<Self> for BitFlags<T, U>
where
    T: BitOrAssign,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Self>,
    {
        for other in iter {
            self.0 |= other.0;
        }
    }
}

impl<T, U> FromIterator<Self> for BitFlags<T, U>
where
    T: Default + BitOrAssign,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        let mut this = Self(T::default(), PhantomData);
        for other in iter {
            this.0 |= other.0;
        }
        this
    }
}

impl<T, U> Clone for BitFlagsIter<T, U>
where
    T: Clone,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, U> Default for BitFlagsIter<T, U>
where
    BitFlags<T, U>: Default,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn default() -> Self {
        Self(BitFlags::default())
    }
}

impl<T, U> fmt::Debug for BitFlagsIter<T, U>
where
    T: fmt::Binary,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BitFlagsIter").field(&self.0).finish()
    }
}

#[cfg(feature = "bytemuck")]
// SAFETY: All fields are valid when their memory is entirely zeroed out.
unsafe impl<T, U> Zeroable for BitFlags<T, U>
where
    T: Zeroable,
    U: ?Sized + BitFlagsSpec<T, AnyBitsAllowed = True>,
{
}

#[cfg(feature = "bytemuck")]
// SAFETY: The type is Copy, Zeroable, has no padding bytes, and any
// bit pattern of the correct size is a valid instance of this type.
unsafe impl<T, U> Pod for BitFlags<T, U>
where
    T: Pod,
    U: 'static + ?Sized + BitFlagsSpec<T, AnyBitsAllowed = True>,
{
}
