use core::any::type_name;
use core::borrow::Borrow;
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
#[repr(transparent)]
pub struct BitFlag<T, U>(u8, PhantomData<T>, PhantomData<fn() -> *const U>)
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
    /// Creates a bit flag for bit index `i` if `i` is less than the number of
    /// bits in `T`.
    ///
    /// The index is not checked against [`BitFlagsSpec::VALID_BITS_MASK`] or
    /// against [`BitFlagsSpec::FLAG_NAMES`].
    /// Use [`BitFlag::try_from_bit`] when validation is required.
    #[inline]
    #[must_use]
    pub const fn try_from_bit_retain(i: u8) -> Option<Self> {
        let max_bits = const {
            size_of::<T>()
                .checked_mul(8)
                .expect("unexpectedly large `T`")
        };

        #[expect(clippy::as_conversions, reason = "supported in const functions")]
        if (i as usize) < max_bits {
            Some(Self(i, PhantomData, PhantomData))
        } else {
            None
        }
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

    /// Parses a single flag from `input` using `parser_matcher`.
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
    pub fn try_from_str(input: &str, parser_matcher: NameMatcher) -> Result<Self, BitFlagParseError>
    where
        T: util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        if let Some(flag) = Self::from_str_opt(input, parser_matcher) {
            return Ok(flag);
        }

        #[cfg(feature = "alloc")]
        let given = alloc::string::ToString::to_string(input);
        #[cfg(not(feature = "alloc"))]
        let given = ();
        let expected = U::FLAG_NAMES;

        Err(BitFlagParseError { given, expected })
    }

    /// Parses a single flag from `input` using `parser_matcher`.
    ///
    /// The input is first matched against the names and aliases in
    /// [`BitFlagsSpec::FLAG_NAMES`]. If no name matches, it is parsed as a
    /// hexadecimal flag with a `0x` prefix that must have exactly one bit set.
    ///
    /// Returns `None` if `input` is not recognized.
    #[inline]
    #[must_use]
    pub fn from_str_opt(input: &str, parser_matcher: NameMatcher) -> Option<Self>
    where
        T: util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        if let Some(flag) = Self::from_name_opt(input, parser_matcher) {
            return Some(flag);
        }
        if let Some(flag) = Self::from_hex_opt(input) {
            return Some(flag);
        }

        None
    }

    /// Parses a named flag from `input` using `parser_matcher`.
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
        parser_matcher: NameMatcher,
    ) -> Result<Self, BitFlagParseNameError>
    where
        T: util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        if let Some(flag) = Self::from_name_opt(input, parser_matcher) {
            return Ok(flag);
        }

        #[cfg(feature = "alloc")]
        let given = alloc::string::ToString::to_string(input);
        #[cfg(not(feature = "alloc"))]
        let given = ();
        let expected = U::FLAG_NAMES;

        Err(BitFlagParseNameError { given, expected })
    }

    /// Parses a named flag from `input` using `parser_matcher`.
    ///
    /// The input is matched against all names and aliases in
    /// [`BitFlagsSpec::FLAG_NAMES`].
    /// Returns `None` if no name matches.
    #[inline]
    #[must_use]
    pub fn from_name_opt(input: &str, parser_matcher: NameMatcher) -> Option<Self>
    where
        T: util::IntOpsExt,
        U: BitFlagsSpec<T>,
    {
        for (i, variants) in U::FLAG_NAMES.iter().enumerate() {
            let Ok(i) = u8::try_from(i) else {
                break;
            };
            let Some(bit_flag) = Self::try_from_bit_retain(i) else {
                break;
            };
            for &expected in *variants {
                let is_matched = parser_matcher.matches(input, expected);
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
        Self::try_from_bit_retain(bit)
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
                    Self::try_from_bit_retain(bit as u8)
                } else {
                    None
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

impl<T, U> Copy for BitFlag<T, U>
where
    T: Copy,
    U: ?Sized,
{
}

impl<T, U> Clone for BitFlag<T, U>
where
    T: Clone,
    U: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0, PhantomData, PhantomData)
    }
}

impl<T, U> Eq for BitFlag<T, U>
where
    T: Eq,
    U: ?Sized,
{
}

impl<T, U> PartialEq for BitFlag<T, U>
where
    T: PartialEq,
    U: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, U> Ord for BitFlag<T, U>
where
    T: Ord,
    U: ?Sized,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T, U> PartialOrd for BitFlag<T, U>
where
    T: PartialOrd,
    U: ?Sized,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T, U> Hash for BitFlag<T, U>
where
    T: Hash,
    U: ?Sized,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T, U> AsRef<u8> for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

impl<T, U> Borrow<u8> for BitFlag<T, U>
where
    U: ?Sized,
{
    #[inline]
    fn borrow(&self) -> &u8 {
        &self.0
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
    T: Not<Output = T> + BitAnd<T, Output = T> + util::IntOpsExt,
    U: ?Sized + BitFlagsSpec<T>,
{
    type Output = BitFlags<T, U>;

    #[inline]
    fn not(self) -> Self::Output {
        !BitFlags::from(self)
    }
}

impl<T, U> fmt::Display for BitFlag<T, U>
where
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
            write!(f, "{:#x}", 1 << bit)
        }
    }
}

impl<T, U> fmt::Debug for BitFlag<T, U>
where
    T: fmt::LowerHex,
    U: ?Sized,
{
    #[expect(clippy::missing_inline_in_public_items, reason = "bloats binary size")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitFlag({:#x}, PhantomData<{}>, PhantomData<{}>)",
            self.0,
            type_name::<T>(),
            type_name::<U>(),
        )
    }
}

impl<T, U> fmt::Binary for BitFlag<T, U>
where
    T: fmt::Binary,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl<T, U> fmt::Octal for BitFlag<T, U>
where
    T: fmt::Octal,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl<T, U> fmt::LowerHex for BitFlag<T, U>
where
    T: fmt::LowerHex,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl<T, U> fmt::UpperHex for BitFlag<T, U>
where
    T: fmt::UpperHex,
    U: ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl<T, U> FromStr for BitFlag<T, U>
where
    T: util::IntOpsExt,
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
unsafe impl<T, U> bytemuck::Zeroable for BitFlag<T, U>
where
    T: bytemuck::Zeroable,
    U: ?Sized,
{
}

#[cfg(feature = "bytemuck")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytemuck")))]
// SAFETY: The type is Copy, Zeroable, has no padding bytes, and any
// bit pattern of the correct size is a valid instance of this type.
unsafe impl<T, U> bytemuck::Pod for BitFlag<T, U>
where
    T: bytemuck::Pod,
    U: 'static + ?Sized,
{
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T, U> serde::Deserialize<'de> for BitFlag<T, U>
where
    T: util::IntOpsExt,
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
    T: util::IntOpsExt,
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
            write!(f, "unrecognized named flag, ")?;
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

#[cfg(feature = "alloc")]
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
        write!(f, "a valid hex flag with \"0x\" prefix")?;
    } else {
        fmt_expected_name(f, expected)?;
        write!(f, ", or a valid hex flag with \"0x\" prefix")?;
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

#[cfg(feature = "alloc")]
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
        write!(f, ", expected a valid hex flag with \"0x\" prefix")?;

        #[cfg(not(feature = "alloc"))]
        let () = self.given;

        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl core::error::Error for BitFlagParseHexError {}
