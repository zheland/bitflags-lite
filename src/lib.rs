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

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(test, not(doc)))]
mod integration_tests_deps {
    use serde_test as _;
    use version_sync as _;
}

use core::any::type_name;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter::{FromIterator, FusedIterator, IntoIterator};
use core::marker::PhantomData;
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Sub, SubAssign,
};
use core::str::FromStr;

// TODO: Document sources of invalid bits including: `from_bits_retain`,
// `arbitrary::Arbitrary`, and `bytemuck::Pod`.
// TODO: Document specific behavior of `Serialize` and `Deserialize` (truncating
// and verifying).
// TODO: Document behavior of raw-bit traits.
//

/// Associates a bit-storage type `T` with the set of bits that are valid for a
/// particular [`BitFlags`] tag.
///
/// Implement this trait for the tag type `U` in [`BitFlags<T, U>`].
/// [`VALID_BITS`](Self::VALID_BITS) defines which bits are valid for the flag
/// set.
///
/// Bits not set in `VALID_BITS` are invalid:
/// - [`BitFlags::from_bits`] rejects them.
/// - [`BitFlags::from_bits_truncate`] and [`BitFlags::truncate`] clear them.
/// - [`BitFlags::complement`] and [`Not`](core::ops::Not) clear them.
/// - [`BitFlags::from_bits_retain`] and operations that do not explicitly
///   truncate may preserve them.
///
/// Unlike the `bitflags` crate, valid bits are not declared by a macro. They
/// are part of this trait implementation for the tag type.
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
///     const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
///     const IS_DEFAULT_PARSING_ASCII_CASE_SENSITIVE: bool = true;
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
    /// A bit pattern whose set bits are the valid bits for this flag set.
    ///
    /// Bits not set in this value are invalid for the associated [`BitFlags`]
    /// type.
    /// They are rejected by [`BitFlags::from_bits`] and cleared by
    /// [`BitFlags::from_bits_truncate`], [`BitFlags::truncate`],
    /// [`BitFlags::complement`], and [`Not`](core::ops::Not).
    const VALID_BITS: T;

    /// A slice indexed by bit position, where each element contains the names
    /// associated with that bit.
    ///
    /// The first name in each inner slice is used by [`BitFlags`]'s
    /// [`Display`](core::fmt::Display) implementation.
    /// All names are accepted when parsing.
    /// An empty inner slice means the bit has no name.
    ///
    /// The index `i` corresponds to the bit mask `1 << i`.
    const FLAG_NAMES: &[&[&str]];

    /// Controls whether the default [`FromStr`](core::str::FromStr)
    /// and [`Deserialize`](serde::Deserialize) implementations for [`BitFlags`]
    /// parses named flags case-sensitively.
    ///
    /// If `true`, named flags are matched exactly.
    /// If `false`, named flags are matched using ASCII case-insensitive
    /// comparison.
    ///
    /// This value does not affect [`BitFlags::from_ascii_case_sensitive_str`]
    /// or [`BitFlags::from_ascii_case_insensitive_str`].
    const IS_DEFAULT_PARSING_ASCII_CASE_SENSITIVE: bool;
}

/// A type-safe set of flags stored in an unsigned integer `T`.
///
/// The second type parameter `U` is a tag type that distinguishes flag sets
/// that share the same storage type.
/// When `U` implements `BitFlagsSpec`, it defines which bits are valid.
///
/// `BitFlags` is `#[repr(transparent)]` over `T`, so it has the same layout as
/// `T`.
///
/// # Invalid bits
///
/// A `BitFlags` value is not guaranteed to contain only valid bits.
/// [`BitFlags::from_bits_retain`] can create a value with any bit pattern of
/// `T`, and raw-bit operations such as [`BitFlags::insert`],
/// [`BitFlags::toggle`], [`BitFlags::union`], and bitwise operators preserve
/// invalid bits. Use [`BitFlags::truncate`], [`BitFlags::from_bits`], or
/// [`BitFlags::from_bits_truncate`] when a value must be restricted to
/// [`BitFlagsSpec::VALID_BITS`].
///
/// [`BitFlags::bits`] returns the raw stored bits, including any invalid bits
/// that are present.
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
///     const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
///     const IS_DEFAULT_PARSING_ASCII_CASE_SENSITIVE: bool = true;
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
pub struct BitFlags<T, U>(
    T,
    // PhantomData<fn() -> *const U> is Send + Sync and covariant over `U`,
    // and `U` is not required to be sized as pattern uses only its pointer.
    // See: https://doc.rust-lang.org/nomicon/phantom-data.html#table-of-phantomdata-patterns
    PhantomData<fn() -> *const U>,
)
where
    U: ?Sized;

/// An iterator over the individual bits set in a [`BitFlags`] value.
///
/// Each item is a `BitFlags` value containing exactly one set bit.
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

/// An iterator over the named flags in a [`BitFlags`] value.
///
/// The returned iterator only yields bits in contained named flags.
/// Any unnamed flags will not be yielded.
///
/// This type is created by the [`IntoIterator`] implementation for
/// [`BitFlags`].
pub struct BitFlagsIterNamesOnly<T, U>(BitFlags<T, U>)
where
    U: ?Sized;

/// Serde visitor for deserializing [`BitFlags`] from a string.
///
/// The accepted format is the same as [`BitFlags`]'s
/// [`FromStr`](core::str::FromStr) implementation: an empty string, a
/// `|`-separated list of named flags, and/or `0x`-prefixed hexadecimal flags.
/// Named flags are matched according to
/// [`BitFlagsSpec::IS_DEFAULT_PARSING_ASCII_CASE_SENSITIVE`].
///
/// Deserialization does not validate the parsed bits against
/// [`BitFlagsSpec::VALID_BITS`]; the resulting value may contain invalid bits.
#[cfg(feature = "serde")]
struct BitFlagsVisitor<T, U>(PhantomData<T>, PhantomData<fn() -> *const U>)
where
    U: ?Sized;

/// Internal utilities for unsigned integer operations used by [`BitFlags`].
///
/// This module is public to support the generic implementations in this crate,
/// but its items are not intended to be used directly.
pub mod util {
    use core::num::ParseIntError;

    /// An extension trait for unsigned integer operations used by
    /// [`BitFlags`](crate::BitFlags).
    ///
    /// It abstracts over the concrete unsigned integer types so that generic
    /// code can use the same operations.
    pub trait UintOpsExt: Sized {
        /// The value `0`.
        const ZERO: Self;

        /// The value `1`.
        const ONE: Self;

        /// Shifts `self` left by `rhs`, returning `None` if `rhs` is greater
        /// than or equal to the bit width of `Self`.
        #[must_use]
        fn checked_shl(self, rhs: u32) -> Option<Self>;

        /// Returns the number of set bits in `self`.
        #[must_use]
        fn count_ones(self) -> u32;

        /// Returns the index of the highest bit set to one in `self`, or `None`
        /// if `self` is `0`.
        #[must_use]
        fn highest_one(self) -> Option<u32>;

        /// Returns the index of the lowest bit set to one in `self`, or `None`
        /// if `self` is `0`.
        #[must_use]
        fn lowest_one(self) -> Option<u32>;

        /// Returns `self` with only the most significant bit set, or `0` if the
        /// input is `0`.
        #[must_use]
        fn isolate_highest_one(self) -> Self;

        /// Returns `self` with only the least significant bit set, or `0` if
        /// the input is `0`.
        ///
        /// Returns `0` if `self` is `0`.
        #[must_use]
        fn isolate_lowest_one(self) -> Self;

        /// Parses `src` as an integer in the given `radix`.
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
}

// Const traits are not stable yet, so the const methods are implemented for
// each integer type separately via a macro.
// See: https://github.com/rust-lang/rust/issues/143874
macro_rules! impl_bit_flags {
    ($ty:ty) => {
        impl<U> BitFlags<$ty, U>
        where
            U: ?Sized,
        {
            /// Creates a flag set with no bits set (zero bits).
            ///
            /// This is equivalent to `from_bits_retain(0)`.
            #[inline]
            #[must_use]
            pub const fn empty() -> Self {
                Self(0, PhantomData)
            }

            /// Creates a flag set containing all valid bits for this tag.
            ///
            /// This returns exactly [`BitFlagsSpec::VALID_BITS`]. Invalid bits
            /// are never set.
            ///
            /// The method is named `all_valid` rather than `all` because a
            /// `BitFlags` value can contain invalid bits after
            /// [`BitFlags::from_bits_retain`].
            /// In that case, "all" would be ambiguous: all bits of the storage
            /// type, all bits currently set, or all valid bits?
            ///
            /// `all_valid` makes the meaning explicit.
            #[inline]
            #[must_use]
            pub const fn all_valid() -> Self
            where
                U: BitFlagsSpec<$ty>,
            {
                Self(U::VALID_BITS, PhantomData)
            }

            /// Creates a flag set from raw bits if all set bits are valid.
            ///
            /// Returns `None` if `bits` contains any bit not present in
            /// [`BitFlagsSpec::VALID_BITS`].
            ///
            /// Use [`BitFlags::from_bits_retain`] to keep invalid bits, or
            /// [`BitFlags::from_bits_truncate`] to clear them.
            #[inline]
            #[must_use]
            pub const fn from_bits(bits: $ty) -> Option<Self>
            where
                U: BitFlagsSpec<$ty>,
            {
                if bits & !U::VALID_BITS == 0 {
                    Some(Self(bits, PhantomData))
                } else {
                    None
                }
            }

            /// Creates a flag set from raw bits, clearing any bits not present in
            /// [`BitFlagsSpec::VALID_BITS`].
            ///
            /// This always succeeds and never stores invalid bits.
            #[inline]
            #[must_use]
            pub const fn from_bits_truncate(bits: $ty) -> Self
            where
                U: BitFlagsSpec<$ty>,
            {
                Self(bits & U::VALID_BITS, PhantomData)
            }

            /// Creates a flag set from raw bits without validation.
            ///
            /// The resulting value may contain bits not present in
            /// [`BitFlagsSpec::VALID_BITS`].
            /// Such invalid bits are preserved by this constructor and by
            /// raw-bit operations unless an operation clears them, such as
            /// [`BitFlags::truncate`], [`BitFlags::complement`],
            /// [`BitFlags::from_bits_truncate`], or [`Not`](core::ops::Not).
            #[inline]
            #[must_use]
            pub const fn from_bits_retain(bits: $ty) -> Self {
                Self(bits, PhantomData)
            }

            /// Clears all bits not present in [`BitFlagsSpec::VALID_BITS`].
            ///
            /// Returns `self` unchanged if it already contains only valid bits.
            #[inline]
            #[must_use]
            pub const fn truncate(self) -> Self
            where
                U: BitFlagsSpec<$ty>,
            {
                Self(self.0 & U::VALID_BITS, PhantomData)
            }

            /// Returns the raw bits stored in this flag set.
            ///
            /// The returned value may contain invalid bits if this value was
            /// created with [`BitFlags::from_bits_retain`] and has not since
            /// been passed through an operation that clears invalid bits, such
            /// as [`BitFlags::truncate`] or [`BitFlags::complement`].
            #[inline]
            #[must_use]
            pub const fn bits(self) -> $ty {
                self.0
            }

            /// Returns `true` if no bits are set.
            ///
            /// This checks the raw stored bits.
            /// A value containing only invalid bits is not empty.
            #[inline]
            #[must_use]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            /// Returns `true` if `self` and `other` have at least one set bit in
            /// common.
            ///
            /// This compares raw bits.
            /// Invalid bits participate in the comparison if they are present.
            #[inline]
            #[must_use]
            pub const fn intersects(&self, other: Self) -> bool {
                (self.0 & other.0) != 0
            }

            /// Returns `true` if all bits set in `other` are also set in `self`.
            ///
            /// This compares raw bits.
            /// Invalid bits participate in the comparison if they are present.
            #[inline]
            #[must_use]
            pub const fn contains(&self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }

            /// Inserts all bits from `other`.
            ///
            /// This is a raw-bit operation.
            /// It does not validate or clear invalid bits.
            /// Inserting `other` can introduce invalid bits into `self`.
            #[inline]
            pub const fn insert(&mut self, other: Self) {
                self.0 |= other.0;
            }

            /// Removes all bits from `other`.
            ///
            /// This is a raw-bit operation.
            /// It cannot introduce new bits, but it does not clear any other
            /// invalid bits that may already be present.
            #[inline]
            pub const fn remove(&mut self, other: Self) {
                self.0 &= !other.0;
            }

            /// Toggles all bits from `other`.
            ///
            /// This is a raw-bit operation.
            /// It does not validate or clear invalid bits.
            /// Toggling `other` can introduce invalid bits into `self`.
            #[inline]
            pub const fn toggle(&mut self, other: Self) {
                self.0 ^= other.0;
            }

            /// Inserts `other` if `value` is `true`, or removes `other` if
            /// `value` is `false`.
            ///
            /// This is a raw-bit operation and follows the same invalid-bit
            /// behavior as [`BitFlags::insert`] and [`BitFlags::remove`].
            #[inline]
            pub const fn set(&mut self, other: Self, value: bool) {
                if value {
                    self.insert(other)
                } else {
                    self.remove(other)
                }
            }

            /// Returns the intersection of `self` and `other`.
            ///
            /// This is a raw-bit operation.
            /// Invalid bits in either operand are preserved when they are set
            /// in both operands.
            #[inline]
            #[must_use]
            pub const fn intersection(self, other: Self) -> Self {
                Self(self.0 & other.0, PhantomData)
            }

            /// Returns the union of `self` and `other`.
            ///
            /// This is a raw-bit operation.
            /// Invalid bits in either operand are preserved.
            #[inline]
            #[must_use]
            pub const fn union(self, other: Self) -> Self {
                Self(self.0 | other.0, PhantomData)
            }

            /// Returns the difference of `self` and `other`.
            ///
            /// The result contains bits that are set in `self` but not in
            /// `other`.
            /// This is a raw-bit operation.
            /// Invalid bits already present in `self` are preserved unless they
            /// are also set in `other`.
            #[inline]
            #[must_use]
            pub const fn difference(self, other: Self) -> Self {
                Self(self.0 & !other.0, PhantomData)
            }

            /// Returns the symmetric difference of `self` and `other`.
            ///
            /// The result contains bits that are set in exactly one of `self`
            /// or `other`.
            /// This is a raw-bit operation.
            /// Invalid bits participate and may be preserved.
            #[inline]
            #[must_use]
            pub const fn symmetric_difference(self, other: Self) -> Self {
                Self(self.0 ^ other.0, PhantomData)
            }

            /// Returns the complement of `self`.
            ///
            /// All valid bits not set in `self` become set, and all invalid
            /// bits become unset.
            /// The result always contains only valid bits.
            #[inline]
            #[must_use]
            pub const fn complement(self) -> Self
            where
                U: BitFlagsSpec<$ty>,
            {
                Self(!self.0 & U::VALID_BITS, PhantomData)
            }
        }

        impl util::UintOpsExt for $ty {
            const ZERO: Self = 0;
            const ONE: Self = 1;

            #[inline]
            fn checked_shl(self, rhs: u32) -> Option<Self> {
                <$ty>::checked_shl(self, rhs)
            }

            #[inline]
            fn count_ones(self) -> u32 {
                <$ty>::count_ones(self)
            }

            #[inline]
            fn isolate_lowest_one(self) -> Self {
                <$ty>::isolate_lowest_one(self)
            }

            #[inline]
            fn isolate_highest_one(self) -> Self {
                <$ty>::isolate_highest_one(self)
            }

            #[inline]
            fn lowest_one(self) -> Option<u32> {
                <$ty>::lowest_one(self)
            }

            #[inline]
            fn highest_one(self) -> Option<u32> {
                <$ty>::highest_one(self)
            }

            #[inline]
            fn from_str_radix(src: &str, radix: u32) -> Result<Self, core::num::ParseIntError> {
                <$ty>::from_str_radix(src, radix)
            }
        }

        impl<U> From<BitFlags<$ty, U>> for $ty
        where
            U: ?Sized,
        {
            #[inline]
            fn from(flags: BitFlags<$ty, U>) -> Self {
                flags.0
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

impl<T, U> BitFlags<T, U>
where
    U: ?Sized,
{
    /// Yield a set of contained flags values.
    #[inline]
    pub const fn iter(self) -> BitFlagsIter<T, U>
    where
        T: Copy + PartialEq + SubAssign + util::UintOpsExt,
    {
        BitFlagsIter(self)
    }

    /// Yield a set of contained named flags values.
    ///
    /// The returned iterator only yields bits in contained named flags.
    /// Any unnamed flags will not be yielded.
    #[inline]
    pub const fn iter_names_only(self) -> BitFlagsIterNamesOnly<T, U>
    where
        T: Copy + PartialEq + Not<Output = T> + BitAndAssign + SubAssign + util::UintOpsExt,
        U: BitFlagsSpec<T>,
    {
        BitFlagsIterNamesOnly(self)
    }

    /// Parses a flag set from a string using ASCII case-sensitive named-flag
    /// matching.
    ///
    /// The input may be empty, which returns [`BitFlags::empty`].
    /// Otherwise it is split on `|`, and each part is trimmed and matched
    /// against the names in [`BitFlagsSpec::FLAG_NAMES`].
    /// A part may also be a hexadecimal flag with a `0x` prefix.
    ///
    /// This method does not validate or truncate the parsed bits against
    /// [`BitFlagsSpec::VALID_BITS`].
    /// The returned value may contain invalid bits.
    ///
    /// Returns [`ParseError`] if a part is not a recognized named flag or a
    /// valid hexadecimal flag.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if an `input` is not recognized as a
    /// `|`-separated list of named flags or hexadecimal flags.
    #[inline]
    pub fn from_ascii_case_sensitive_str(input: &str) -> Result<Self, ParseError>
    where
        T: Default
            + Not<Output = T>
            + BitAndAssign
            + BitOrAssign
            + Shl<usize, Output = T>
            + util::UintOpsExt,
        U: BitFlagsSpec<T>,
    {
        Self::from_str_impl(input, true)
    }

    /// Parses a flag set from a string using ASCII case-insensitive named-flag
    /// matching.
    ///
    /// The input may be empty, which returns [`BitFlags::empty`].
    /// Otherwise it is split on `|`, and each part is trimmed and matched
    /// against the names in [`BitFlagsSpec::FLAG_NAMES`].
    /// A part may also be a hexadecimal flag with a `0x` prefix.
    ///
    /// This method does not validate or truncate the parsed bits against
    /// [`BitFlagsSpec::VALID_BITS`].
    /// The returned value may contain invalid bits.
    ///
    /// Returns [`ParseError`] if a part is not a recognized named flag or a
    /// valid hexadecimal flag.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if an `input` is not recognized as a
    /// `|`-separated list of named flags or hexadecimal flags.
    #[inline]
    pub fn from_ascii_case_insensitive_str(input: &str) -> Result<Self, ParseError>
    where
        T: Default
            + Not<Output = T>
            + BitAndAssign
            + BitOrAssign
            + Shl<usize, Output = T>
            + util::UintOpsExt,
        U: BitFlagsSpec<T>,
    {
        Self::from_str_impl(input, false)
    }

    /// Shared implementation for string parsing.
    ///
    /// If `is_ascii_case_sensitive` is `true`, named flags are matched exactly.
    /// Otherwise they are matched using ASCII case-insensitive comparison.
    #[inline]
    fn from_str_impl(input: &str, is_ascii_case_sensitive: bool) -> Result<Self, ParseError>
    where
        T: Default
            + Not<Output = T>
            + BitAndAssign
            + BitOrAssign
            + Shl<usize, Output = T>
            + util::UintOpsExt,
        U: BitFlagsSpec<T>,
    {
        if input.trim().is_empty() {
            return Ok(Self::default());
        }

        let mut parsed = T::default();
        let flags = input.split('|');
        'a: for flag in flags {
            let flag = flag.trim();

            for (i, variants) in U::FLAG_NAMES.iter().enumerate() {
                #[expect(
                    clippy::as_conversions,
                    clippy::cast_possible_truncation,
                    reason = "would break from the loop before truncation"
                )]
                let Some(mask) = T::ONE.checked_shl(i as u32) else {
                    break;
                };
                for &expected in *variants {
                    let is_matched = if is_ascii_case_sensitive {
                        flag == expected
                    } else {
                        flag.eq_ignore_ascii_case(expected)
                    };
                    if is_matched {
                        parsed |= mask;
                        continue 'a;
                    }
                }
            }

            if let Some(maybe_hex_flag) = flag.strip_prefix("0x") {
                let flag = T::from_str_radix(maybe_hex_flag, 16);
                if let Ok(flag) = flag {
                    parsed |= flag;
                    continue 'a;
                }
            }

            #[cfg(feature = "alloc")]
            let given = alloc::string::ToString::to_string(flag);
            #[cfg(not(feature = "alloc"))]
            let given = ();
            let expected = U::FLAG_NAMES;
            return Err(ParseError { given, expected });
        }
        Ok(Self(parsed, PhantomData))
    }
}

impl<T, U> Copy for BitFlags<T, U>
where
    T: Copy,
    U: ?Sized,
{
}

impl<T, U> Clone for BitFlags<T, U>
where
    T: Clone,
    U: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T, U> PartialEq for BitFlags<T, U>
where
    T: PartialEq,
    U: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, U> Eq for BitFlags<T, U>
where
    T: Eq,
    U: ?Sized,
{
}

impl<T, U> Ord for BitFlags<T, U>
where
    T: Ord,
    U: ?Sized,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T, U> PartialOrd for BitFlags<T, U>
where
    T: PartialOrd,
    U: ?Sized,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T, U> Hash for BitFlags<T, U>
where
    T: Hash,
    U: ?Sized,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T, U> Default for BitFlags<T, U>
where
    T: Default,
    U: ?Sized,
{
    #[inline]
    fn default() -> Self {
        Self(T::default(), PhantomData)
    }
}

impl<T, U> AsRef<T> for BitFlags<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T, U> Borrow<T> for BitFlags<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T, U> BitOr for BitFlags<T, U>
where
    T: BitOr<T, Output = T>,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0, PhantomData)
    }
}

impl<T, U> BitOrAssign for BitFlags<T, U>
where
    T: BitOrAssign,
    U: ?Sized,
{
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl<T, U> BitAnd for BitFlags<T, U>
where
    T: BitAnd<T, Output = T>,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0, PhantomData)
    }
}

impl<T, U> BitAndAssign for BitFlags<T, U>
where
    T: BitAndAssign,
    U: ?Sized,
{
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl<T, U> BitXor for BitFlags<T, U>
where
    T: BitXor<T, Output = T>,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0, PhantomData)
    }
}

impl<T, U> BitXorAssign for BitFlags<T, U>
where
    T: BitXorAssign,
    U: ?Sized,
{
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl<T, U> Sub for BitFlags<T, U>
where
    T: Not<Output = T> + BitAnd<T, Output = T>,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0, PhantomData)
    }
}

impl<T, U> SubAssign for BitFlags<T, U>
where
    T: Not<Output = T> + BitAndAssign,
    U: ?Sized,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.0;
    }
}

impl<T, U> Not for BitFlags<T, U>
where
    T: Not<Output = T> + BitAnd<T, Output = T>,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Self(!self.0 & U::VALID_BITS, PhantomData)
    }
}

impl<T, U> fmt::Display for BitFlags<T, U>
where
    T: Copy
        + PartialEq
        + Not<Output = T>
        + BitAnd<T, Output = T>
        + Shl<usize, Output = T>
        + fmt::LowerHex
        + util::UintOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut separator = "";
        let mut this = self.0;
        for (i, variants) in U::FLAG_NAMES.iter().enumerate() {
            #[expect(
                clippy::as_conversions,
                clippy::cast_possible_truncation,
                reason = "would break from the loop before truncation"
            )]
            let Some(mask) = T::ONE.checked_shl(i as u32) else {
                break;
            };
            if let Some(flag) = variants.first() {
                let new_this = this & !mask;
                if this != new_this {
                    write!(f, "{separator}{flag}")?;
                    separator = " | ";
                }
                this = new_this;
            }
        }
        if this != T::ZERO {
            write!(f, "{separator}{this:#x}")?;
        }
        Ok(())
    }
}

impl<T, U> fmt::Debug for BitFlags<T, U>
where
    T: fmt::LowerHex,
    U: ?Sized,
{
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitFlags({:#x}, PhantomData<{}>)",
            self.0,
            type_name::<U>()
        )
    }
}

impl<T, U> fmt::Binary for BitFlags<T, U>
where
    T: fmt::Binary,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl<T, U> fmt::Octal for BitFlags<T, U>
where
    T: fmt::Octal,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl<T, U> fmt::LowerHex for BitFlags<T, U>
where
    T: fmt::LowerHex,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl<T, U> fmt::UpperHex for BitFlags<T, U>
where
    T: fmt::UpperHex,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl<T, U> FromStr for BitFlags<T, U>
where
    T: Default
        + Not<Output = T>
        + BitAndAssign
        + BitOrAssign
        + Shl<usize, Output = T>
        + FromStr
        + util::UintOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Err = ParseError;

    #[inline]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::from_str_impl(input, U::IS_DEFAULT_PARSING_ASCII_CASE_SENSITIVE)
    }
}

impl<T, U> Extend<Self> for BitFlags<T, U>
where
    T: BitOrAssign,
    U: ?Sized,
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

impl<T, U> IntoIterator for BitFlags<T, U>
where
    T: Copy + PartialEq + SubAssign + util::UintOpsExt,
    U: ?Sized,
{
    type IntoIter = BitFlagsIter<T, U>;
    type Item = Self;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitFlagsIter(self)
    }
}

impl<T, U> FromIterator<Self> for BitFlags<T, U>
where
    T: Default + BitOrAssign,
    U: ?Sized,
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
    T: Default,
    U: ?Sized,
{
    #[inline]
    fn default() -> Self {
        Self(BitFlags::default())
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
    T: Copy + PartialEq + SubAssign + util::UintOpsExt,
    U: ?Sized,
{
    type Item = BitFlags<T, U>;

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "never result in side-effects"
    )]
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let lowest = self.0.0.isolate_lowest_one();
        if lowest == T::ZERO {
            None
        } else {
            self.0.0 -= lowest;
            Some(BitFlags(lowest, PhantomData))
        }
    }

    #[expect(clippy::as_conversions, reason = "T::count_ones never exceeds `usize`")]
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.0.count_ones() as usize;
        (len, Some(len))
    }
}

impl<T, U> DoubleEndedIterator for BitFlagsIter<T, U>
where
    T: Copy + PartialEq + SubAssign + util::UintOpsExt,
    U: ?Sized,
{
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "never result in side-effects"
    )]
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let lowest = self.0.0.isolate_highest_one();
        if lowest == T::ZERO {
            None
        } else {
            self.0.0 -= lowest;
            Some(BitFlags(lowest, PhantomData))
        }
    }
}

impl<T, U> ExactSizeIterator for BitFlagsIter<T, U>
where
    T: Copy + PartialEq + SubAssign + util::UintOpsExt,
    U: ?Sized,
{
    #[expect(clippy::as_conversions, reason = "T::count_ones never exceeds `usize`")]
    #[inline]
    fn len(&self) -> usize {
        self.0.0.count_ones() as usize
    }
}

impl<T, U> FusedIterator for BitFlagsIter<T, U>
where
    T: Copy + PartialEq + SubAssign + util::UintOpsExt,
    U: ?Sized,
{
}

impl<T, U> fmt::Debug for BitFlagsIterNamesOnly<T, U>
where
    T: fmt::LowerHex,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BitFlagsIterNamesOnly")
            .field(&self.0)
            .finish()
    }
}

impl<T, U> Iterator for BitFlagsIterNamesOnly<T, U>
where
    T: Copy + PartialEq + Not<Output = T> + BitAndAssign + SubAssign + util::UintOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Item = &'static str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(lowest) = self.0.0.lowest_one() {
            self.0.0 &= !T::ONE.checked_shl(lowest).expect("just found");
            #[expect(clippy::as_conversions, reason = "never overflows")]
            let lowest = lowest as usize;
            if let Some(flag) = U::FLAG_NAMES.get(lowest).and_then(|opts| opts.first()) {
                return Some(*flag);
            }
        }
        None
    }

    #[expect(clippy::as_conversions, reason = "T::count_ones never exceeds `usize`")]
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.0.count_ones() as usize;
        (0, Some(len))
    }
}

impl<T, U> DoubleEndedIterator for BitFlagsIterNamesOnly<T, U>
where
    T: Copy + PartialEq + Not<Output = T> + BitAndAssign + SubAssign + util::UintOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(lowest) = self.0.0.highest_one() {
            self.0.0 &= !T::ONE.checked_shl(lowest).expect("just found");
            #[expect(clippy::as_conversions, reason = "never overflows")]
            let lowest = lowest as usize;
            if let Some(flag) = U::FLAG_NAMES.get(lowest).and_then(|opts| opts.first()) {
                return Some(*flag);
            }
        }
        None
    }
}

impl<T, U> FusedIterator for BitFlagsIterNamesOnly<T, U>
where
    T: Copy + PartialEq + Not<Output = T> + BitAndAssign + SubAssign + util::UintOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<T, U> serde::Serialize for BitFlags<T, U>
where
    T: Copy
        + PartialEq
        + Not<Output = T>
        + BitAnd<T, Output = T>
        + Shl<usize, Output = T>
        + fmt::LowerHex
        + util::UintOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Replicates the behavior of `bitflags`-based implementation.
        serializer.collect_str(&self)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T, U> serde::Deserialize<'de> for BitFlags<T, U>
where
    T: Default
        + Not<Output = T>
        + BitAndAssign
        + BitOrAssign
        + Shl<usize, Output = T>
        + FromStr
        + util::UintOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(BitFlagsVisitor::<T, U>(PhantomData, PhantomData))
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<T, U> serde::de::Visitor<'_> for BitFlagsVisitor<T, U>
where
    T: Default
        + Not<Output = T>
        + BitAndAssign
        + BitOrAssign
        + Shl<usize, Output = T>
        + FromStr
        + util::UintOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Value = BitFlags<T, U>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_expected(f, U::FLAG_NAMES)
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::Value::from_str(value).map_err(E::custom)
    }
}

#[cfg(feature = "arbitrary")]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary")))]
impl<'a, T, U> arbitrary::Arbitrary<'a> for BitFlags<T, U>
where
    T: arbitrary::Arbitrary<'a>,
    U: ?Sized,
{
    #[inline]
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        T::arbitrary(u).map(|u| Self(u, PhantomData))
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        T::size_hint(depth)
    }
}

#[cfg(feature = "bytemuck")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytemuck")))]
// SAFETY: All fields are valid when their memory is entirely zeroed out.
unsafe impl<T, U> bytemuck::Zeroable for BitFlags<T, U>
where
    T: bytemuck::Zeroable,
    U: ?Sized,
{
}

#[cfg(feature = "bytemuck")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytemuck")))]
// SAFETY: The type is Copy, Zeroable, has no padding bytes, and any
// bit pattern of the correct size is a valid instance of this type.
unsafe impl<T, U> bytemuck::Pod for BitFlags<T, U>
where
    T: bytemuck::Pod,
    U: 'static + ?Sized,
{
}

/// An error encountered while parsing flags from text.
#[derive(Clone, Debug)]
pub struct ParseError {
    #[cfg(not(feature = "alloc"))]
    given: (),
    #[cfg(feature = "alloc")]
    given: alloc::string::String,

    expected: &'static [&'static [&'static str]],
}

impl fmt::Display for ParseError {
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let named_flags = self
            .expected
            .iter()
            .filter(|variants| !variants.is_empty())
            .count();

        if named_flags == 0 {
            write!(f, "unrecognized hex flag, ")?;
        } else {
            write!(f, "unrecognized named flag or hex flag, ")?;
        }
        #[cfg(feature = "alloc")]
        write!(f, "given: \"{}\", expected ", self.given)?;
        #[cfg(not(feature = "alloc"))]
        write!(f, "expected ")?;
        fmt_expected(f, self.expected)?;

        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl core::error::Error for ParseError {}

/// Writes the expected flag format for [`ParseError`] and serde visitors.
#[inline]
fn fmt_expected(
    f: &mut fmt::Formatter<'_>,
    expected: &'static [&'static [&'static str]],
) -> fmt::Result {
    let named_flags = expected
        .iter()
        .filter(|variants| !variants.is_empty())
        .count();

    if named_flags == 0 {
        write!(f, "a valid hex flag with \"0x\" prefix")?;
    } else {
        write!(f, "\"\", or a \"|\"-separated set of")?;

        let mut separator = " ";
        for variants in expected {
            if let Some(flag) = variants.first() {
                write!(f, "{separator}\"{flag}\"")?;
                separator = ", ";
            }
        }
        write!(f, ", or a valid hex flag")?;
    }
    Ok(())
}
