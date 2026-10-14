use core::any::type_name;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter::{FromIterator, IntoIterator};
use core::marker::PhantomData;
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Sub, SubAssign,
};
use core::str::FromStr;

use crate::{BitFlag, BitFlagsIter, BitFlagsSpec, NameMatcher, util};

/// A type-safe set of flags stored in an integer `T`.
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
/// invalid bits. Use [`BitFlags::truncate`], [`BitFlags::try_from_bits`], or
/// [`BitFlags::from_bits_truncate`] when a value must be restricted to
/// [`BitFlagsSpec::VALID_BITS_MASK`].
///
/// [`BitFlags::bits`] returns the raw stored bits, including any invalid bits
/// that are present.
///
/// # Examples
///
/// ```
/// use bitflags_lite::{BitFlags, BitFlagsSpec, NameMatcher};
///
/// struct PermissionsTag;
///
/// impl BitFlagsSpec<u32> for PermissionsTag {
///     const VALID_BITS_MASK: u32 = (1 << 3) - 1;
///     const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
///     const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
/// }
///
/// type Permissions = BitFlags<u32, PermissionsTag>;
///
/// const READ: Permissions = Permissions::try_from_bits(1 << 0).unwrap();
/// const WRITE: Permissions = Permissions::try_from_bits(1 << 1).unwrap();
/// const EXEC: Permissions = Permissions::try_from_bits(1 << 2).unwrap();
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

/// Serde visitor for deserializing [`BitFlags`] from a string.
///
/// The accepted format is the same as [`BitFlags`]'s
/// [`FromStr`](core::str::FromStr) implementation: an empty string, a
/// `|`-separated list of named flags, and/or `0x`-prefixed hexadecimal flags.
/// Named flags are matched according to
/// [`BitFlagsSpec::DEFAULT_NAME_MATCHER`].
///
/// Deserialization does not validate the parsed bits against
/// [`BitFlagsSpec::VALID_BITS_MASK`].
/// The resulting value may contain invalid bits.
#[cfg(feature = "serde")]
struct BitFlagsVisitor<T, U>(PhantomData<T>, PhantomData<fn() -> *const U>)
where
    U: ?Sized;

impl<T, U> BitFlags<T, U>
where
    U: ?Sized,
{
    /// Creates a flag set with no bits set (zero bits).
    ///
    /// This is equivalent to `from_bits_retain(0)`.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self
    where
        T: util::IntOpsExt,
    {
        Self(T::ZERO, PhantomData)
    }

    /// Creates a flag set containing all valid bits for this tag.
    ///
    /// This returns exactly [`BitFlagsSpec::VALID_BITS_MASK`]. Invalid bits
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
        U: BitFlagsSpec<T>,
    {
        Self(U::VALID_BITS_MASK, PhantomData)
    }

    /// Creates a flag set from raw bits without validation.
    ///
    /// The resulting value may contain bits not present in
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    /// Such invalid bits are preserved by this constructor and by
    /// raw-bit operations unless an operation clears them, such as
    /// [`BitFlags::truncate`], [`BitFlags::complement`],
    /// [`BitFlags::from_bits_truncate`], or [`Not`](core::ops::Not).
    #[inline]
    #[must_use]
    pub const fn from_bits_retain(bits: T) -> Self {
        Self(bits, PhantomData)
    }

    /// Returns the raw bits stored in this flag set.
    ///
    /// The returned value may contain invalid bits if this value was
    /// created with [`BitFlags::from_bits_retain`] and has not since
    /// been passed through an operation that clears invalid bits, such
    /// as [`BitFlags::truncate`] or [`BitFlags::complement`].
    #[inline]
    #[must_use]
    pub const fn bits(&self) -> T
    where
        T: Copy,
    {
        self.0
    }

    /// Returns a mutable reference to the raw bits stored in this flag set.
    ///
    /// Mutating the returned value can introduce invalid bits.
    /// Use [`BitFlags::truncate`] afterwards if the value must contain only
    /// valid bits.
    #[inline]
    #[must_use]
    pub const fn bits_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Returns an iterator over the individual set bits in this flag set.
    ///
    /// The iterator yields [`BitFlag`] values from the least significant bit to
    /// the most significant bit.
    /// It includes invalid bits if they are present.
    #[inline]
    pub const fn iter(self) -> BitFlagsIter<T, U>
    where
        T: Copy + Not<Output = T> + BitAndAssign<T> + util::IntOpsExt,
    {
        BitFlagsIter::new(self)
    }

    /// Parses a flag set from a string.
    ///
    /// The input may be empty, which returns [`BitFlags::empty`].
    /// Otherwise it is split on given `separator`, and each part is trimmed and
    /// matched against the names in [`BitFlagsSpec::FLAG_NAMES`].
    /// Match behavior is specified by `name_matcher`.
    /// A part may also be a hexadecimal flag with a `0x` prefix.
    ///
    /// This method does not validate or truncate the parsed bits against
    /// [`BitFlagsSpec::VALID_BITS_MASK`].
    /// The returned value may contain invalid bits.
    ///
    /// Returns [`BitFlagsParseError`] if a part is not a recognized named flag
    /// or a valid hexadecimal flag.
    ///
    /// Note that if `separator` is an empty string, the method would split into
    /// individual characters, which is almost certainly not intended.
    ///
    /// # Errors
    ///
    /// Returns [`BitFlagsParseError`] if an `input` is not recognized as a
    /// `separator`-separated list of named flags or hexadecimal flags.
    #[inline]
    pub fn try_from_str(
        input: &str,
        separator: &str,
        name_matcher: NameMatcher,
    ) -> Result<Self, BitFlagsParseError>
    where
        T: BitOrAssign<T> + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        if input.trim().is_empty() {
            return Ok(Self::empty());
        }

        let mut this = Self::empty();
        let input = input.split(separator);
        for input in input {
            let input = input.trim();

            let flag = BitFlag::from_name_opt(input, name_matcher);
            if let Some(flag) = flag {
                this |= flag;
                continue;
            }

            if let Some(maybe_hex_flag) = input.strip_prefix("0x") {
                // `from_str_radix` allows optional `+` prefix.
                // Parsing here should prohibit it.
                if !maybe_hex_flag.starts_with('+') {
                    let bits = T::from_str_radix(maybe_hex_flag, 16);
                    if let Ok(bits) = bits {
                        this |= Self::from_bits_retain(bits);
                        continue;
                    }
                }
            }

            #[cfg(feature = "alloc")]
            let given = alloc::string::ToString::to_string(input);
            #[cfg(not(feature = "alloc"))]
            let given = ();
            let expected = U::FLAG_NAMES;
            return Err(BitFlagsParseError { given, expected });
        }

        Ok(this)
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
            /// Creates a flag set from a single [`BitFlag`].
            ///
            /// This retains the bit index from `bit_flag` without validating it
            /// against the width of `T` or [`BitFlagsSpec::VALID_BITS_MASK`].
            #[inline]
            #[must_use]
            pub const fn from_flag(flag: BitFlag<$ty, U>) -> Self {
                Self::from_bit_retain(flag.bit())
            }

            /// Creates a flag set with only the bit at index `bit` set.
            ///
            /// The bit index is not validated against
            /// [`BitFlagsSpec::VALID_BITS_MASK`].
            /// The resulting value may contain an invalid bit.
            /// Use [`BitFlags::try_from_bit`] or
            /// [`BitFlags::from_bits_truncate`] when validation or truncation
            /// is required.
            #[inline]
            #[must_use]
            pub const fn from_bit_retain(bit: u8) -> Self {
                #[expect(clippy::as_conversions, reason = "supported in const functions")]
                let bit = bit as u32;
                if bit < <$ty>::BITS {
                    Self::from_bits_retain(1 << bit)
                } else {
                    Self::empty()
                }
            }

            /// Creates a flag set from bit index `bit` if that bit is valid.
            ///
            /// Returns `None` if `bit` is outside the width of `T` or if the
            /// resulting bit is not present in
            /// [`BitFlagsSpec::VALID_BITS_MASK`].
            #[inline]
            #[must_use]
            pub const fn try_from_bit(bit: u8) -> Option<Self>
            where
                U: BitFlagsSpec<$ty>,
            {
                #[expect(clippy::as_conversions, reason = "supported in const functions")]
                let bit = bit as u32;
                if bit < <$ty>::BITS {
                    Self::try_from_bits(1 << bit)
                } else {
                    None
                }
            }

            /// Creates a flag set from raw bits if all set bits are valid.
            ///
            /// Returns `None` if `bits` contains any bit not present in
            /// [`BitFlagsSpec::VALID_BITS_MASK`].
            ///
            /// Use [`BitFlags::from_bits_retain`] to keep invalid bits, or
            /// [`BitFlags::from_bits_truncate`] to clear them.
            #[inline]
            #[must_use]
            pub const fn try_from_bits(bits: $ty) -> Option<Self>
            where
                U: BitFlagsSpec<$ty>,
            {
                if bits & !U::VALID_BITS_MASK == 0 {
                    Some(Self(bits, PhantomData))
                } else {
                    None
                }
            }

            /// Creates a flag set from raw bits, clearing any bits not present in
            /// [`BitFlagsSpec::VALID_BITS_MASK`].
            ///
            /// This always succeeds and never stores invalid bits.
            #[inline]
            #[must_use]
            pub const fn from_bits_truncate(bits: $ty) -> Self
            where
                U: BitFlagsSpec<$ty>,
            {
                Self(bits & U::VALID_BITS_MASK, PhantomData)
            }

            /// Returns `true` if no bits are set.
            ///
            /// This checks the raw stored bits.
            /// A value containing only invalid bits is not empty.
            #[inline]
            #[must_use]
            pub const fn is_empty(&self) -> bool {
                self.0 == 0
            }

            /// Returns `true` if all set bits in `self` are valid for this tag.
            ///
            /// A `BitFlags` value can contain bits not present in
            /// [`BitFlagsSpec::VALID_BITS_MASK`] after construction with
            /// [`BitFlags::from_bits_retain`] or after raw-bit operations such as
            /// [`BitFlags::insert`], [`BitFlags::toggle`], and [`BitFlags::union`].
            /// This method returns `false` for such values.
            ///
            /// Use [`BitFlags::truncate`] or [`BitFlags::from_bits_truncate`] to
            /// restrict a value to its valid bits.
            #[inline]
            #[must_use]
            pub const fn is_valid(&self) -> bool
            where
                U: BitFlagsSpec<$ty>,
            {
                self.bits() & !U::VALID_BITS_MASK == 0
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

            /// Clears all bits not present in
            /// [`BitFlagsSpec::VALID_BITS_MASK`].
            ///
            /// This leaves `self` unchanged if it already contains only valid
            /// bits.
            #[inline]
            pub const fn truncate(&mut self)
            where
                U: BitFlagsSpec<$ty>,
            {
                self.0 &= U::VALID_BITS_MASK
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
                Self(!self.0 & U::VALID_BITS_MASK, PhantomData)
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

impl_bit_flags!(i8);
impl_bit_flags!(i16);
impl_bit_flags!(i32);
impl_bit_flags!(i64);
impl_bit_flags!(i128);
impl_bit_flags!(isize);

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

impl<T, U> Eq for BitFlags<T, U>
where
    T: Eq,
    U: ?Sized,
{
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

impl<T, U> PartialEq<BitFlag<T, U>> for BitFlags<T, U>
where
    T: Copy + PartialEq + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn eq(&self, other: &BitFlag<T, U>) -> bool {
        self.eq(&Self::from(*other))
    }
}

impl<T, U> PartialEq<BitFlags<T, U>> for BitFlag<T, U>
where
    T: Copy + PartialEq + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn eq(&self, other: &BitFlags<T, U>) -> bool {
        BitFlags::from(*self).eq(other)
    }
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

impl<T, U> PartialOrd<BitFlag<T, U>> for BitFlags<T, U>
where
    T: Copy + PartialOrd + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn partial_cmp(&self, other: &BitFlag<T, U>) -> Option<Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

impl<T, U> PartialOrd<BitFlags<T, U>> for BitFlag<T, U>
where
    T: Copy + PartialOrd + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn partial_cmp(&self, other: &BitFlags<T, U>) -> Option<Ordering> {
        BitFlags::from(*self).partial_cmp(other)
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

impl<T, U> From<BitFlag<T, U>> for BitFlags<T, U>
where
    T: util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn from(flag: BitFlag<T, U>) -> Self {
        let bit = flag.bit();
        T::ONE
            .checked_shl(u32::from(bit))
            .map_or_else(Self::empty, Self::from_bits_retain)
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

impl<T, U> BitAnd<Self> for BitFlags<T, U>
where
    T: BitAnd<T, Output = T>,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, PhantomData)
    }
}

impl<T, U> BitAnd<BitFlag<T, U>> for BitFlags<T, U>
where
    T: BitAnd<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: BitFlag<T, U>) -> Self::Output {
        self.bitand(Self::from(rhs))
    }
}

impl<T, U> BitAnd<BitFlags<T, U>> for BitFlag<T, U>
where
    T: BitAnd<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = BitFlags<T, U>;

    #[inline]
    fn bitand(self, rhs: BitFlags<T, U>) -> Self::Output {
        BitFlags::from(self).bitand(rhs)
    }
}

impl<T, U> BitAndAssign<Self> for BitFlags<T, U>
where
    T: BitAndAssign<T>,
    U: ?Sized,
{
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl<T, U> BitAndAssign<BitFlag<T, U>> for BitFlags<T, U>
where
    T: BitAndAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn bitand_assign(&mut self, rhs: BitFlag<T, U>) {
        self.bitand_assign(Self::from(rhs));
    }
}

impl<T, U> BitOr<Self> for BitFlags<T, U>
where
    T: BitOr<T, Output = T>,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, PhantomData)
    }
}

impl<T, U> BitOr<BitFlag<T, U>> for BitFlags<T, U>
where
    T: BitOr<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: BitFlag<T, U>) -> Self::Output {
        self.bitor(Self::from(rhs))
    }
}

impl<T, U> BitOr<BitFlags<T, U>> for BitFlag<T, U>
where
    T: BitOr<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = BitFlags<T, U>;

    #[inline]
    fn bitor(self, rhs: BitFlags<T, U>) -> Self::Output {
        BitFlags::from(self).bitor(rhs)
    }
}

impl<T, U> BitOrAssign<Self> for BitFlags<T, U>
where
    T: BitOrAssign<T>,
    U: ?Sized,
{
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl<T, U> BitOrAssign<BitFlag<T, U>> for BitFlags<T, U>
where
    T: BitOrAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn bitor_assign(&mut self, rhs: BitFlag<T, U>) {
        self.bitor_assign(Self::from(rhs));
    }
}

impl<T, U> BitXor<Self> for BitFlags<T, U>
where
    T: BitXor<T, Output = T>,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0, PhantomData)
    }
}

impl<T, U> BitXor<BitFlag<T, U>> for BitFlags<T, U>
where
    T: BitXor<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: BitFlag<T, U>) -> Self::Output {
        self.bitxor(Self::from(rhs))
    }
}

impl<T, U> BitXor<BitFlags<T, U>> for BitFlag<T, U>
where
    T: BitXor<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = BitFlags<T, U>;

    #[inline]
    fn bitxor(self, rhs: BitFlags<T, U>) -> Self::Output {
        BitFlags::from(self).bitxor(rhs)
    }
}

impl<T, U> BitXorAssign<Self> for BitFlags<T, U>
where
    T: BitXorAssign<T>,
    U: ?Sized,
{
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl<T, U> BitXorAssign<BitFlag<T, U>> for BitFlags<T, U>
where
    T: BitXorAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn bitxor_assign(&mut self, rhs: BitFlag<T, U>) {
        self.bitxor_assign(Self::from(rhs));
    }
}

impl<T, U> Sub<Self> for BitFlags<T, U>
where
    T: Not<Output = T> + BitAnd<T, Output = T>,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 & !rhs.0, PhantomData)
    }
}

impl<T, U> Sub<BitFlag<T, U>> for BitFlags<T, U>
where
    T: Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: BitFlag<T, U>) -> Self::Output {
        self.sub(Self::from(rhs))
    }
}

impl<T, U> SubAssign<Self> for BitFlags<T, U>
where
    T: Not<Output = T> + BitAndAssign<T>,
    U: ?Sized,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.0;
    }
}

impl<T, U> SubAssign<BitFlag<T, U>> for BitFlags<T, U>
where
    T: Not<Output = T> + BitAndAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn sub_assign(&mut self, rhs: BitFlag<T, U>) {
        self.sub_assign(Self::from(rhs));
    }
}

impl<T, U> Not for BitFlags<T, U>
where
    T: Not<Output = T> + BitAnd<T, Output = T>,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0 & U::VALID_BITS_MASK, PhantomData)
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
        + util::IntOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut separator = "";
        let mut this = self.0;
        for (i, names) in U::FLAG_NAMES.iter().enumerate() {
            let Some(i) = u32::try_from(i).ok() else {
                break; // Extremely exotic type.
            };
            let Some(mask) = T::ONE.checked_shl(i) else {
                break;
            };
            if let Some(name) = names.first() {
                let new_this = this & !mask;
                if this != new_this {
                    write!(f, "{separator}{name}")?;
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
    T: BitOrAssign<T> + util::IntOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Err = BitFlagsParseError;

    #[inline]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(input, "|", U::DEFAULT_NAME_MATCHER)
    }
}

impl<T, U> IntoIterator for BitFlags<T, U>
where
    T: Copy + Not<Output = T> + BitAndAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    type IntoIter = BitFlagsIter<T, U>;
    type Item = BitFlag<T, U>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitFlagsIter::new(self)
    }
}

impl<T, U> Extend<Self> for BitFlags<T, U>
where
    T: BitOrAssign<T>,
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

impl<T, U> Extend<BitFlag<T, U>> for BitFlags<T, U>
where
    T: BitOrAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = BitFlag<T, U>>,
    {
        for other in iter {
            *self |= Self::from(other);
        }
    }
}

impl<T, U> FromIterator<Self> for BitFlags<T, U>
where
    T: Default + BitOrAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        let mut this = Self(T::ZERO, PhantomData);
        for other in iter {
            this.0 |= other.0;
        }
        this
    }
}

impl<T, U> FromIterator<BitFlag<T, U>> for BitFlags<T, U>
where
    T: Default + BitOrAssign<T> + util::IntOpsExt,
    U: ?Sized,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = BitFlag<T, U>>,
    {
        let mut this = Self(T::ZERO, PhantomData);
        for other in iter {
            this |= Self::from(other);
        }
        this
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
        + util::IntOpsExt,
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
    T: BitOrAssign<T> + util::IntOpsExt,
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
    T: BitOrAssign<T> + util::IntOpsExt,
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

/// An error encountered while parsing flags from text.
#[cfg_attr(
    not(feature = "alloc"),
    expect(
        missing_copy_implementations,
        reason = "the payload is feature-dependent"
    )
)]
#[derive(Clone, Debug)]
pub struct BitFlagsParseError {
    #[cfg(not(feature = "alloc"))]
    given: (),
    #[cfg(feature = "alloc")]
    given: alloc::string::String,

    expected: &'static [&'static [&'static str]],
}

impl fmt::Display for BitFlagsParseError {
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

        #[cfg(not(feature = "alloc"))]
        let () = self.given;

        Ok(())
    }
}

impl core::error::Error for BitFlagsParseError {}

/// Writes the expected flag format for [`BitFlagsParseError`] and serde
/// visitors.
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
        for names in expected {
            if let Some(name) = names.first() {
                write!(f, "{separator}\"{name}\"")?;
                separator = ", ";
            }
        }
        write!(f, ", or a valid hex flag with \"0x\" prefix")?;
    }
    Ok(())
}
