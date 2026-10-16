use core::any::type_name;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, BitXor, Not};
use core::str::FromStr;

use crate::{BitFlags, BitFlagsSpec, NameMatcher, util};

/// A single named or indexed bit in a [`BitFlags`] set.
///
/// A `BitFlag` stores a bit index and is tied to the same storage type `T` and
/// tag type `U` as the corresponding [`BitFlags<T, U>`].
/// It is primarily returned by [`BitFlagsIter`] and can be combined with other
/// flags or converted into a [`BitFlags`] set.
///
/// # Formatting
///
/// [`Display`](fmt::Display) writes the flag's name if one is defined in
/// [`BitFlagsSpec::FLAG_NAMES`], and its mask (`1 << bit`) in lowercase
/// hexadecimal with a `0x` prefix otherwise.
/// The mask is computed from the bit index regardless of whether the index is
/// in range for `T`.
///
/// [`Debug`](fmt::Debug) writes the bit index and the type parameters.
///
/// [`Binary`](fmt::Binary), [`Octal`](fmt::Octal),
/// [`LowerHex`](fmt::LowerHex), and [`UpperHex`](fmt::UpperHex) write the
/// mask in the corresponding numeric base.
///
/// # Method summary
///
/// - [`from_bit_retain()`](Self::from_bit_retain): Return a flag for a bit
///   index, without validation.
/// - [`try_from_bit()`](Self::try_from_bit): Return a flag for a bit index,
///   rejecting it if invalid.
/// - [`try_from_bits()`](Self::try_from_bits): Return a flag from a single-bit
///   mask, rejecting it if invalid.
/// - [`try_from_bits_retain()`](Self::try_from_bits_retain): Return a flag from
///   a single-bit mask, without validation.
/// - [`try_from_flags()`](Self::try_from_flags): Return a flag from a
///   [`BitFlags`] with exactly one valid bit.
/// - [`try_from_str()`](Self::try_from_str): Parse from a name or a hex flag.
/// - [`from_str_opt()`](Self::from_str_opt): Parse from a name or a hex flag,
///   returning `Option`.
/// - [`try_from_name()`](Self::try_from_name): Parse from a name only.
/// - [`from_name_opt()`](Self::from_name_opt): Parse from a name only,
///   returning `Option`.
/// - [`try_from_hex()`](Self::try_from_hex): Parse from a hex flag only.
/// - [`from_hex_opt()`](Self::from_hex_opt): Parse from a hex flag only,
///   returning `Option`.
/// - [`bit()`](Self::bit): Return the zero-based bit index.
/// - [`mask()`](Self::mask): Return the single-bit mask, or `None` if out of
///   range.
/// - [`mask_or_default()`](Self::mask_or_default): Return the single-bit mask,
///   or `T::default` if out of range.
/// - [`name()`](Self::name): Return the primary name, if any.
/// - [`name_and_aliases()`](Self::name_and_aliases): Return all names and
///   aliases.
/// - [`is_valid()`](Self::is_valid): Return `true` if the bit is valid for the
///   tag.
/// - [`intersection()`](Self::intersection): Return the bits set in both flags.
/// - [`union()`](Self::union): Return the bits set in either flag.
/// - [`difference()`](Self::difference): Return the bits set in `self` but not
///   in `other`.
/// - [`symmetric_difference()`](Self::symmetric_difference): Return the bits
///   set in exactly one flag.
/// - [`complement()`](Self::complement): Return the valid bits not set in
///   `self`.
///
/// [`BitFlagsIter`]: struct.BitFlagsIter.html
#[repr(transparent)]
pub struct BitFlag<T, U>(u8, PhantomData<fn() -> T>, PhantomData<fn() -> *const U>)
where
    U: ?Sized;

/// Serde visitor for deserializing [`BitFlag`] from a string.
#[cfg(feature = "serde")]
struct BitFlagVisitor<T, U>(PhantomData<T>, PhantomData<fn() -> *const U>)
where
    U: ?Sized;

impl<T, U> BitFlag<T, U>
where
    U: ?Sized,
{
    /// Creates a bit flag for bit index `i`.
    ///
    /// The index is not checked against the number of bits in `T`, against
    /// [`BitFlagsSpec::VALID_BITS_MASK`], or against
    /// [`BitFlagsSpec::FLAG_NAMES`].
    /// A `BitFlag` created with this method may refer to a bit that is out of
    /// range for `T` or invalid for the tag.
    /// Use [`BitFlag::try_from_bit`] or [`BitFlag::is_valid`] when validation
    /// is required.
    #[inline]
    #[must_use]
    pub const fn from_bit_retain(i: u8) -> Self {
        Self(i, PhantomData, PhantomData)
    }

    /// Returns the zero-based bit index stored in this flag.
    #[inline]
    #[must_use]
    pub const fn bit(&self) -> u8 {
        self.0
    }

    /// Returns the primary name of this bit, if one is defined in
    /// [`BitFlagsSpec::FLAG_NAMES`].
    ///
    /// The primary name is the first entry in the inner slice for this bit.
    /// Returns `None` if the bit has no name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<&'static str>
    where
        T: 'static,
        U: BitFlagsSpec<T>,
    {
        let names_and_aliases = self.name_and_aliases();
        if names_and_aliases.is_empty() {
            None
        } else {
            Some(names_and_aliases[0])
        }
    }

    /// Returns all names and aliases for this bit.
    ///
    /// The returned slice is the entry at this bit's index in
    /// [`BitFlagsSpec::FLAG_NAMES`], or an empty slice if the index is out of
    /// range or the bit has no names.
    #[inline]
    #[must_use]
    pub const fn name_and_aliases(&self) -> &'static [&'static str]
    where
        T: 'static,
        U: BitFlagsSpec<T>,
    {
        #[expect(clippy::as_conversions, reason = "supported in const functions")]
        let bit = self.0 as usize;
        if bit < U::FLAG_NAMES.len() {
            U::FLAG_NAMES[bit]
        } else {
            &[]
        }
    }

    /// Parses a single flag from `input` using `name_matcher`.
    ///
    /// This is equivalent to [`BitFlag::try_from_str`], but returns a
    /// [`BitFlagParseError`] describing the input and expected names on
    /// failure.
    ///
    /// # Errors
    ///
    /// Returns [`BitFlagParseError`] if `input` is neither a recognized named
    /// flag nor a hexadecimal flag of the form `0x...` with exactly one bit
    /// set.
    #[inline]
    pub fn try_from_str(input: &str, name_matcher: NameMatcher) -> Result<Self, BitFlagParseError>
    where
        T: 'static + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        if let Some(flag) = Self::from_str_opt(input, name_matcher) {
            return Ok(flag);
        }

        #[cfg(feature = "alloc")]
        let given = alloc::string::ToString::to_string(input);
        #[cfg(not(feature = "alloc"))]
        let given = ();
        let expected = U::FLAG_NAMES;

        Err(BitFlagParseError { given, expected })
    }

    /// Parses a single flag from `input` using `name_matcher`.
    ///
    /// The input is first matched against the names and aliases in
    /// [`BitFlagsSpec::FLAG_NAMES`]. If no name matches, it is parsed as a
    /// hexadecimal flag with a `0x` prefix that must have exactly one bit set.
    ///
    /// Returns `None` if `input` is not recognized.
    #[inline]
    #[must_use]
    pub fn from_str_opt(input: &str, name_matcher: NameMatcher) -> Option<Self>
    where
        T: 'static + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        if let Some(flag) = Self::from_name_opt(input, name_matcher) {
            return Some(flag);
        }
        if let Some(flag) = Self::from_hex_opt(input) {
            return Some(flag);
        }

        None
    }

    /// Parses a named flag from `input` using `name_matcher`.
    ///
    /// This is equivalent to [`BitFlag::try_from_name`], but returns a
    /// [`BitFlagParseNameError`] describing the input and expected names on
    /// failure.
    ///
    /// # Errors
    ///
    /// Returns [`BitFlagParseNameError`] if `input` does not match any name or
    /// alias in [`BitFlagsSpec::FLAG_NAMES`].
    #[inline]
    pub fn try_from_name(
        input: &str,
        name_matcher: NameMatcher,
    ) -> Result<Self, BitFlagParseNameError>
    where
        T: 'static + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        if let Some(flag) = Self::from_name_opt(input, name_matcher) {
            return Ok(flag);
        }

        #[cfg(feature = "alloc")]
        let given = alloc::string::ToString::to_string(input);
        #[cfg(not(feature = "alloc"))]
        let given = ();
        let expected = U::FLAG_NAMES;

        Err(BitFlagParseNameError { given, expected })
    }

    /// Parses a named flag from `input` using `name_matcher`.
    ///
    /// The input is matched against all names and aliases in
    /// [`BitFlagsSpec::FLAG_NAMES`].
    /// Returns `None` if no name matches.
    #[inline]
    #[must_use]
    pub fn from_name_opt(input: &str, name_matcher: NameMatcher) -> Option<Self>
    where
        T: 'static + util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        for (i, variants) in U::FLAG_NAMES.iter().enumerate().take(usize::from(T::BITS)) {
            let Ok(bit) = u8::try_from(i) else {
                #[cfg(debug_assertions)]
                unreachable!("`i < T::BITS <= u8::MAX`, so this is unreachable");
                #[cfg(not(debug_assertions))]
                break;
            };
            let bit_flag = Self::from_bit_retain(bit);
            for &expected in *variants {
                let is_matched = name_matcher.matches(input, expected);
                if is_matched {
                    return Some(bit_flag);
                }
            }
        }

        None
    }

    /// Parses a hexadecimal flag from `input`.
    ///
    /// This is equivalent to [`BitFlag::try_from_hex`], but returns a
    /// [`BitFlagParseHexError`] describing the input on failure.
    ///
    /// # Errors
    ///
    /// Returns [`BitFlagParseHexError`] if `input` is not a valid `0x`-prefixed
    /// hexadecimal flag with exactly one bit set.
    #[inline]
    pub fn try_from_hex(input: &str) -> Result<Self, BitFlagParseHexError>
    where
        T: util::IntOpsExt,
    {
        if let Some(flag) = Self::from_hex_opt(input) {
            return Ok(flag);
        }

        #[cfg(feature = "alloc")]
        let given = alloc::string::ToString::to_string(input);
        #[cfg(not(feature = "alloc"))]
        let given = ();

        Err(BitFlagParseHexError { given })
    }

    /// Parses a hexadecimal flag from `input`.
    ///
    /// The input must have a `0x` prefix and represent a value with exactly one
    /// bit set.
    /// A leading `+` is not accepted.
    /// Returns `None` if `input` is not a valid hexadecimal flag.
    #[inline]
    #[must_use]
    pub fn from_hex_opt(input: &str) -> Option<Self>
    where
        T: util::IntOpsExt,
    {
        let input = input.strip_prefix("0x")?;
        if input.starts_with('+') {
            return None;
        }
        let bits = T::from_str_radix(input, 16).ok()?;

        if bits.count_ones() != 1 {
            return None;
        }
        let bit = bits.lowest_one()?;
        let bit = u8::try_from(bit).ok()?;
        (bit < T::BITS).then(|| Self::from_bit_retain(bit))
    }
}

// Const traits are not stable yet, so the const methods are implemented for
// each integer type separately via a macro.
// See: https://github.com/rust-lang/rust/issues/143874
macro_rules! impl_bit_flag {
    ($ty:ty) => {
        impl<U> BitFlag<$ty, U>
        where
            U: ?Sized,
        {
            /// Creates a bit flag from a [`BitFlags`] set.
            ///
            /// Returns `Some` if `flags` contains exactly one set bit and that
            /// bit is valid according to [`BitFlagsSpec::VALID_BITS_MASK`].
            ///
            /// Returns `None` if `flags` is empty, has more than one bit set,
            /// or the single set bit is not valid.
            ///
            /// This is equivalent to [`BitFlag::try_from_bits`] on
            /// `flags.bits()`.
            #[inline]
            #[must_use]
            pub const fn try_from_flags(flags: BitFlags<$ty, U>) -> Option<Self>
            where
                U: BitFlagsSpec<$ty>,
            {
                Self::try_from_bits(flags.bits())
            }

            /// Creates a validated bit flag for bit index `i`.
            ///
            /// Returns `None` if `i` is not set in
            /// [`BitFlagsSpec::VALID_BITS_MASK`].
            ///
            /// The index `i` must be less than the number of bits in the
            /// storage type.
            #[inline]
            #[must_use]
            pub const fn try_from_bit(i: u8) -> Option<Self>
            where
                U: BitFlagsSpec<$ty>,
            {
                const MAX_BITS: usize = size_of::<$ty>() * 8;

                #[expect(clippy::as_conversions, reason = "supported in const functions")]
                if (i as usize) < MAX_BITS && ((1 << (i as u32)) & !U::VALID_BITS_MASK) == 0 {
                    Some(Self(i, PhantomData, PhantomData))
                } else {
                    None
                }
            }

            /// Creates a validated bit flag from `bits` if exactly one bit is
            /// set and that bit is valid according to
            /// [`BitFlagsSpec::VALID_BITS_MASK`].
            ///
            /// Returns `None` if `bits` is zero, has more than one bit set, or
            /// the single set bit is not valid.
            #[inline]
            #[must_use]
            pub const fn try_from_bits(bits: $ty) -> Option<Self>
            where
                U: BitFlagsSpec<$ty>,
            {
                if bits.count_ones() == 1
                    && let Some(bit) = bits.lowest_one()
                {
                    #[expect(
                        clippy::as_conversions,
                        clippy::cast_possible_truncation,
                        reason = "supported in const functions"
                    )]
                    Self::try_from_bit(bit as u8)
                } else {
                    None
                }
            }

            /// Creates a single-bit flag from `bits` if exactly one bit is set.
            ///
            /// The single set bit is not checked against
            /// [`BitFlagsSpec::VALID_BITS_MASK`].
            ///
            /// Returns `None` if `bits` is `0` or has more than one bit set.
            #[inline]
            #[must_use]
            pub const fn try_from_bits_retain(bits: $ty) -> Option<Self> {
                if bits.count_ones() == 1
                    && let Some(bit) = bits.lowest_one()
                {
                    #[expect(
                        clippy::as_conversions,
                        clippy::cast_possible_truncation,
                        reason = "supported in const functions"
                    )]
                    Some(Self::from_bit_retain(bit as u8))
                } else {
                    None
                }
            }

            /// Returns `true` if this flag refers to a valid bit.
            ///
            /// A flag is valid when its bit index is within the width of `T` and the
            /// corresponding bit is present in [`BitFlagsSpec::VALID_BITS_MASK`].
            ///
            /// [`BitFlag::from_bit_retain`] does not validate the bit index, so a
            /// `BitFlag` constructed through it may be invalid.
            #[inline]
            #[must_use]
            pub const fn is_valid(&self) -> bool
            where
                U: BitFlagsSpec<$ty>,
            {
                #[expect(clippy::as_conversions, reason = "supported in const functions")]
                ((self.0 as u32) < <$ty>::BITS && BitFlags::<$ty, U>::from_flag(*self).is_valid())
            }

            /// Returns the single-bit mask for this flag, or `None` if the bit index is
            /// outside the width of `T`.
            ///
            /// The mask is `1 << bit`. It is not checked against
            /// [`BitFlagsSpec::VALID_BITS_MASK`]; use [`BitFlag::is_valid`] when tag
            /// validity is required.
            #[inline]
            #[must_use]
            pub const fn mask(&self) -> Option<$ty> {
                #[expect(clippy::as_conversions, reason = "supported in const functions")]
                let bit = self.0 as u32;
                if bit < <$ty>::BITS {
                    Some(1 << bit)
                } else {
                    None
                }
            }

            /// Returns the single-bit mask for this flag, or `0` if the bit index is
            /// outside the width of `$ty`.
            ///
            /// This is the const-friendly equivalent of `self.mask().unwrap_or(0)`.
            #[inline]
            #[must_use]
            pub const fn mask_or_default(&self) -> $ty {
                if let Some(mask) = self.mask() {
                    mask
                } else {
                    0
                }
            }

            /// Returns the intersection of `self` and `other`.
            ///
            /// This is a raw-bit operation.
            /// Invalid bits in either operand are preserved when they are set
            /// in both operands.
            #[inline]
            #[must_use]
            pub const fn intersection(self, other: Self) -> BitFlags<$ty, U> {
                BitFlags::<$ty, U>::from_flag(self)
                    .intersection(BitFlags::<$ty, U>::from_flag(other))
            }

            /// Returns the union of `self` and `other`.
            ///
            /// This is a raw-bit operation.
            /// Invalid bits in either operand are preserved.
            #[inline]
            #[must_use]
            pub const fn union(self, other: Self) -> BitFlags<$ty, U> {
                BitFlags::<$ty, U>::from_flag(self).union(BitFlags::<$ty, U>::from_flag(other))
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
            pub const fn difference(self, other: Self) -> BitFlags<$ty, U> {
                BitFlags::<$ty, U>::from_flag(self).difference(BitFlags::<$ty, U>::from_flag(other))
            }

            /// Returns the symmetric difference of `self` and `other`.
            ///
            /// The result contains bits that are set in exactly one of `self`
            /// or `other`.
            /// This is a raw-bit operation.
            /// Invalid bits participate and may be preserved.
            #[inline]
            #[must_use]
            pub const fn symmetric_difference(self, other: Self) -> BitFlags<$ty, U> {
                BitFlags::<$ty, U>::from_flag(self)
                    .symmetric_difference(BitFlags::<$ty, U>::from_flag(other))
            }

            /// Returns the complement of `self`.
            ///
            /// All valid bits not set in `self` become set, and all invalid
            /// bits become unset.
            /// The result always contains only valid bits.
            #[inline]
            #[must_use]
            pub const fn complement(self) -> BitFlags<$ty, U>
            where
                U: BitFlagsSpec<$ty>,
            {
                BitFlags::<$ty, U>::from_flag(self).complement()
            }
        }
    };
}

impl_bit_flag!(u8);
impl_bit_flag!(u16);
impl_bit_flag!(u32);
impl_bit_flag!(u64);
impl_bit_flag!(u128);
impl_bit_flag!(usize);

impl_bit_flag!(i8);
impl_bit_flag!(i16);
impl_bit_flag!(i32);
impl_bit_flag!(i64);
impl_bit_flag!(i128);
impl_bit_flag!(isize);

impl<T, U> Copy for BitFlag<T, U> where U: ?Sized {}

impl<T, U> Clone for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, U> Eq for BitFlag<T, U> where U: ?Sized {}

impl<T, U> PartialEq for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, U> Ord for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T, U> PartialOrd for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, U> Hash for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Converts a flag set into a single-bit flag.
///
/// This fails unless `flags` contains exactly one set bit and that bit is
/// valid for the tag. See [`BitFlag::try_from_flags`].
impl<T, U> TryFrom<BitFlags<T, U>> for BitFlag<T, U>
where
    T: 'static + Copy + PartialEq + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
    U: BitFlagsSpec<T>,
{
    type Error = BitFlagFromFlagsError;

    #[inline]
    fn try_from(flags: BitFlags<T, U>) -> Result<Self, Self::Error> {
        let bits = flags.bits();
        if bits.count_ones() == 1
            && let Some(bit) = bits.lowest_one()
            && let Ok(bit) = u8::try_from(bit)
            && let Some(mask) = T::ONE.checked_shl(u32::from(bit))
            && (mask & !U::VALID_BITS_MASK) == T::ZERO
        {
            Ok(Self::from_bit_retain(bit))
        } else {
            Err(BitFlagFromFlagsError)
        }
    }
}

impl<T, U> BitAnd<Self> for BitFlag<T, U>
where
    T: BitAnd<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = BitFlags<T, U>;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        BitFlags::from(self) & BitFlags::from(rhs)
    }
}

impl<T, U> BitOr<Self> for BitFlag<T, U>
where
    T: BitOr<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = BitFlags<T, U>;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        BitFlags::from(self) | BitFlags::from(rhs)
    }
}

impl<T, U> BitXor<Self> for BitFlag<T, U>
where
    T: BitXor<T, Output = T> + util::IntOpsExt,
    U: ?Sized,
{
    type Output = BitFlags<T, U>;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        BitFlags::from(self) ^ BitFlags::from(rhs)
    }
}

impl<T, U> Not for BitFlag<T, U>
where
    T: 'static + Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = BitFlags<T, U>;

    #[inline]
    fn not(self) -> Self::Output {
        !BitFlags::from(self)
    }
}

/// Formats this flag as its name if one is defined for its bit, or as its
/// mask (`1 << bit`) in lowercase hexadecimal with a `0x` prefix otherwise.
///
/// The mask is computed from the bit index regardless of whether the index
/// is in range for `T`. See [`BitFlag::is_valid`].
impl<T, U> fmt::Display for BitFlag<T, U>
where
    T: 'static,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bit = usize::from(self.0);
        if let Some(names) = U::FLAG_NAMES.get(bit)
            && let Some(name) = names.first()
        {
            write!(f, "{name}")
        } else {
            let leading = 1 << (bit % 4);
            let num_zeroes = bit / 4;
            write!(f, "0x{leading}{:0<num_zeroes$}", "")
        }
    }
}

/// Formats this flag's bit index and type parameters.
///
/// Use [`Display`](fmt::Display) to format the flag's mask or name instead.
impl<T, U> fmt::Debug for BitFlag<T, U>
where
    U: ?Sized,
{
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitFlag(1 << {}, PhantomData<({}, {})>)",
            self.0,
            type_name::<T>(),
            type_name::<U>(),
        )
    }
}

/// Formats this flag's mask (`1 << bit`) in binary.
impl<T, U> fmt::Binary for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0b")?;
        }
        let bit = usize::from(self.0);
        write!(f, "1{:0<bit$}", "")
    }
}

/// Formats this flag's mask (`1 << bit`) in octal.
impl<T, U> fmt::Octal for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0o")?;
        }
        let bit = usize::from(self.0);
        let leading = 1 << (bit % 3);
        let num_zeroes = bit / 3;
        write!(f, "{leading}{:0<num_zeroes$}", "")
    }
}

/// Formats this flag's mask (`1 << bit`) in lowercase hexadecimal.
impl<T, U> fmt::LowerHex for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        let bit = usize::from(self.0);
        let leading = 1 << (bit % 4);
        let num_zeroes = bit / 4;
        write!(f, "{leading}{:0<num_zeroes$}", "")
    }
}

/// Formats this flag's mask (`1 << bit`) in uppercase hexadecimal.
impl<T, U> fmt::UpperHex for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        let bit = usize::from(self.0);
        let leading = 1 << (bit % 4);
        let num_zeroes = bit / 4;
        write!(f, "{leading}{:0<num_zeroes$}", "")
    }
}

impl<T, U> FromStr for BitFlag<T, U>
where
    T: 'static + util::IntOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Err = BitFlagParseError;

    #[inline]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(input, U::DEFAULT_NAME_MATCHER)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<T, U> serde::Serialize for BitFlag<T, U>
where
    T: 'static,
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

#[cfg(feature = "arbitrary")]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary")))]
impl<'a, T, U> arbitrary::Arbitrary<'a> for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u8::arbitrary(u).map(|u| Self(u, PhantomData, PhantomData))
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        u8::size_hint(depth)
    }
}

#[cfg(feature = "bytemuck")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytemuck")))]
// SAFETY: All fields are valid when their memory is entirely zeroed out.
unsafe impl<T, U> bytemuck::Zeroable for BitFlag<T, U> where U: ?Sized {}

#[cfg(feature = "bytemuck")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytemuck")))]
// SAFETY: The type is Copy, Zeroable, has no padding bytes, and any
// bit pattern of the correct size is a valid instance of this type.
unsafe impl<T, U> bytemuck::Pod for BitFlag<T, U>
where
    T: 'static,
    U: 'static + ?Sized,
{
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T, U> serde::Deserialize<'de> for BitFlag<T, U>
where
    T: 'static + util::IntOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(BitFlagVisitor::<T, U>(PhantomData, PhantomData))
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<T, U> serde::de::Visitor<'_> for BitFlagVisitor<T, U>
where
    T: 'static + util::IntOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Value = BitFlag<T, U>;

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
pub struct BitFlagParseError {
    #[cfg(not(feature = "alloc"))]
    given: (),
    #[cfg(feature = "alloc")]
    given: alloc::string::String,

    expected: &'static [&'static [&'static str]],
}

impl fmt::Display for BitFlagParseError {
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

impl core::error::Error for BitFlagParseError {}

/// Writes the expected flag format for [`BitFlagParseError`] and serde
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
        write!(f, "a valid one-bit hex flag with \"0x\" prefix")?;
    } else {
        fmt_expected_name(f, expected)?;
        write!(f, ", or a valid one-bit hex flag with \"0x\" prefix")?;
    }
    Ok(())
}

/// An error returned when parsing a named flag fails.
#[cfg_attr(
    not(feature = "alloc"),
    expect(
        missing_copy_implementations,
        reason = "the payload is feature-dependent"
    )
)]
#[derive(Clone, Debug)]
pub struct BitFlagParseNameError {
    #[cfg(not(feature = "alloc"))]
    given: (),
    #[cfg(feature = "alloc")]
    given: alloc::string::String,

    expected: &'static [&'static [&'static str]],
}

impl fmt::Display for BitFlagParseNameError {
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let named_flags = self
            .expected
            .iter()
            .filter(|variants| !variants.is_empty())
            .count();

        if named_flags == 0 {
            write!(f, "does not support named flags")?;
        } else {
            write!(f, "unrecognized named flag, ")?;
            #[cfg(feature = "alloc")]
            write!(f, "given: \"{}\", expected ", self.given)?;
            #[cfg(not(feature = "alloc"))]
            write!(f, "expected ")?;
            fmt_expected(f, self.expected)?;
        }

        #[cfg(not(feature = "alloc"))]
        let () = self.given;

        Ok(())
    }
}

impl core::error::Error for BitFlagParseNameError {}

/// Writes the expected named-flag format.
#[inline]
fn fmt_expected_name(
    f: &mut fmt::Formatter<'_>,
    expected: &'static [&'static [&'static str]],
) -> fmt::Result {
    let mut separator = " ";

    write!(f, "one of")?;
    for names in expected {
        if let Some(name) = names.first() {
            write!(f, "{separator}\"{name}\"")?;
            separator = ", ";
        }
    }
    Ok(())
}

/// An error returned when parsing a hexadecimal flag fails.
#[cfg_attr(
    not(feature = "alloc"),
    expect(
        missing_copy_implementations,
        reason = "the payload is feature-dependent"
    )
)]
#[derive(Clone, Debug)]
pub struct BitFlagParseHexError {
    #[cfg(not(feature = "alloc"))]
    given: (),
    #[cfg(feature = "alloc")]
    given: alloc::string::String,
}

impl fmt::Display for BitFlagParseHexError {
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognized hex flag")?;

        #[cfg(feature = "alloc")]
        write!(f, ", given: \"{}\"", self.given)?;
        write!(f, ", expected a valid one-bit hex flag with \"0x\" prefix")?;

        #[cfg(not(feature = "alloc"))]
        let () = self.given;

        Ok(())
    }
}

impl core::error::Error for BitFlagParseHexError {}

/// An error returned when converting [`BitFlags`] into a [`BitFlag`] fails.
///
/// The conversion fails if the flag set contains zero or more than one set
/// bit, or if the single set bit is not valid for the tag.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct BitFlagFromFlagsError;

impl fmt::Display for BitFlagFromFlagsError {
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "flags do not contain exactly one valid bit")
    }
}

impl core::error::Error for BitFlagFromFlagsError {}
