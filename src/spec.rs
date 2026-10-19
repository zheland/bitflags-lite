/// Associates a bit-storage type `T` with the set of bits that are valid for a
/// particular [`BitFlags`] tag.
///
/// Implement this trait for the tag type `U` in [`BitFlags<T, U>`].
/// [`VALID_BITS_MASK`](Self::VALID_BITS_MASK) defines which bits are valid for
/// the flag set.
///
/// Bits not set in `VALID_BITS_MASK` are invalid:
/// - [`BitFlags::try_from_bits`] rejects them.
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
/// use bitflags_lite::{BitFlags, BitFlagsSpec, NameMatcher};
///
/// struct PermissionsTag;
///
/// impl BitFlagsSpec<u32> for PermissionsTag {
///     const VALID_BITS_MASK: u32 = 0b111;
///     const DEFAULT_BITS: u32 = 0;
///     const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
///     const COMBINED_NAMES: &[(u32, &str)] = &[(0, "none"), (0b111, "rwx")];
///     const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
/// }
///
/// type Permissions = BitFlags<u32, PermissionsTag>;
///
/// const READ: Permissions = Permissions::try_from_bit(0).unwrap();
/// const WRITE: Permissions = Permissions::try_from_bit(1).unwrap();
/// const EXEC: Permissions = Permissions::try_from_bit(2).unwrap();
///
/// assert_eq!(Permissions::try_from_bits(0b1), Some(READ));
/// assert_eq!(Permissions::try_from_bits(0b11), Some(READ | WRITE));
/// assert_eq!(Permissions::try_from_bits(0b111), Some(READ | WRITE | EXEC));
/// assert_eq!(Permissions::try_from_bits(0b1111), None);
/// ```
///
/// [`BitFlags`]: struct.BitFlags.html
/// [`BitFlags<T, U>`]: struct.BitFlags.html
/// [`BitFlags::try_from_bits`]: struct.BitFlags.html#method.try_from_bits
/// [`BitFlags::from_bits_truncate`]: struct.BitFlags.html#method.from_bits_truncate
/// [`BitFlags::truncate`]: struct.BitFlags.html#method.truncate
/// [`BitFlags::complement`]: struct.BitFlags.html#method.complement
/// [`BitFlags::from_bits_retain`]: struct.BitFlags.html#method.from_bits_retain
pub trait BitFlagsSpec<T>
where
    T: 'static,
{
    /// A bit pattern whose set bits are the valid bits for this flag set.
    ///
    /// Bits not set in this value are invalid for the associated [`BitFlags`]
    /// type.
    /// They are rejected by [`BitFlags::try_from_bits`] and cleared by
    /// [`BitFlags::from_bits_truncate`], [`BitFlags::truncate`],
    /// [`BitFlags::complement`], and [`Not`](core::ops::Not).
    ///
    /// [`BitFlags`]: struct.BitFlags.html
    /// [`BitFlags::try_from_bits`]: struct.BitFlags.html#method.try_from_bits
    /// [`BitFlags::from_bits_truncate`]: struct.BitFlags.html#method.from_bits_truncate
    /// [`BitFlags::truncate`]: struct.BitFlags.html#method.truncate
    /// [`BitFlags::complement`]: struct.BitFlags.html#method.complement
    const VALID_BITS_MASK: T;

    /// The bit pattern returned by [`BitFlags::default`].
    ///
    /// For flag sets where zero is the natural default, set this to `0`.
    /// For flag sets with an "on by default" semantics, set it to the
    /// corresponding mask.
    ///
    /// This value is not validated against [`VALID_BITS_MASK`]; the caller may
    /// choose to allow invalid bits in the default, matching the behavior of
    /// [`BitFlags::from_bits_retain`].
    ///
    /// [`BitFlags::default`]: struct.BitFlags.html#method.default
    /// [`BitFlags::from_bits_retain`]: struct.BitFlags.html#method.from_bits_retain
    /// [`VALID_BITS_MASK`]: trait.BitFlagsSpec.html#associatedconstant.VALID_BITS_MASK
    const DEFAULT_BITS: T;

    /// A slice indexed by bit position, where each element contains the names
    /// associated with that bit.
    ///
    /// The first name in each inner slice is used by [`BitFlag`]'s and
    /// [`BitFlags`]'s [`Display`](core::fmt::Display) implementations.
    /// All names are accepted when parsing.
    /// An empty inner slice means the bit has no name.
    ///
    /// The index `i` corresponds to the bit mask `1 << i`.
    ///
    /// [`BitFlag`]: struct.BitFlag.html
    /// [`BitFlags`]: struct.BitFlags.html
    const FLAG_NAMES: &[&[&str]];

    /// Names for combinations of flags, consulted when parsing.
    ///
    /// Each entry pairs a bit pattern with the single name that should parse
    /// to it. When parsing, the input string is matched against all entries;
    /// the first match wins.
    ///
    /// The bit pattern may include bits that are not named in
    /// [`FLAG_NAMES`], or that are not declared as individual flags at all.
    /// This const does not validate against
    /// [`VALID_BITS_MASK`](Self::VALID_BITS_MASK).
    /// Parsing preserves whatever bits the entry contains.
    ///
    /// This const is **not** consulted by [`Display`](core::fmt::Display).
    /// Formatting a value with multiple bits set always writes the individual
    /// flag names from [`FLAG_NAMES`], never a combined name.
    ///
    /// [`FLAG_NAMES`]: trait.BitFlagsSpec.html#associatedconstant.FLAG_NAMES
    const COMBINED_NAMES: &[(T, &str)];

    /// Controls whether the default [`FromStr`](core::str::FromStr) and
    /// `Deserialize` implementations for [`BitFlags`] parse named flags
    /// case-sensitively.
    ///
    /// If [`NameMatcher::Exact`], named flags are matched exactly.
    /// If [`NameMatcher::AsciiCaseInsensitive`], named flags are matched
    /// using ASCII case-insensitive comparison.
    ///
    /// This value does not affect methods that accept an explicit
    /// [`NameMatcher`], such as [`BitFlags::try_from_str`].
    ///
    /// [`BitFlags`]: struct.BitFlags.html
    /// [`BitFlags::try_from_str`]: struct.BitFlags.html#method.try_from_str
    const DEFAULT_NAME_MATCHER: NameMatcher;
}

/// Controls how flag names are matched when parsing.
///
/// # Why `#[non_exhaustive]`
///
/// `NameMatcher` is only used as an input parameter to parsing APIs, never
/// returned as a result.
/// `#[non_exhaustive]` is appropriate here because new matcher variants can
/// then be added in minor releases without breaking callers.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum NameMatcher {
    /// Matches flag names exactly.
    Exact,

    /// Matches flag names using ASCII case-insensitive comparison.
    AsciiCaseInsensitive,
}

impl NameMatcher {
    /// Returns `true` if `given` matches `expected` according to this matcher.
    #[inline]
    #[must_use]
    pub fn matches(self, given: &str, expected: &str) -> bool {
        match self {
            Self::Exact => given == expected,
            Self::AsciiCaseInsensitive => given.eq_ignore_ascii_case(expected),
        }
    }
}
