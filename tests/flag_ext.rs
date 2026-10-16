//! Tests for `flag.rs` file.

#![expect(clippy::unwrap_used, reason = "okay in tests")]
#![allow(unused_crate_dependencies, reason = "okay in tests")]

use core::cmp::Ordering;
use core::fmt::{self, LowerHex};
use core::hash::Hash;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use bitflags_lite::{
    BitFlag, BitFlagExt, BitFlagFromFlagsError, BitFlags, BitFlagsSpec, NameMatcher,
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct MyU8(pub u8);

impl Not for MyU8 {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAnd<Self> for MyU8 {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign<Self> for MyU8 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr<Self> for MyU8 {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign<Self> for MyU8 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor<Self> for MyU8 {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign<Self> for MyU8 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl LowerHex for MyU8 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <u8 as LowerHex>::fmt(&self.0, f)
    }
}

impl bitflags_lite::util::IntOpsExt for MyU8 {
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const BITS: u8 = 8;

    #[inline]
    fn checked_shl(&self, rhs: u32) -> Option<Self> {
        Some(Self(u8::checked_shl(self.0, rhs)?))
    }

    #[inline]
    fn count_ones(&self) -> u32 {
        u8::count_ones(self.0)
    }

    #[inline]
    fn lowest_one(&self) -> Option<u32> {
        u8::lowest_one(self.0)
    }

    #[inline]
    fn highest_one(&self) -> Option<u32> {
        u8::highest_one(self.0)
    }

    #[inline]
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, core::num::ParseIntError> {
        Ok(Self(u8::from_str_radix(src, radix)?))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct PermsTag;
type Perm = BitFlag<MyU8, PermsTag>;
type Perms = BitFlags<MyU8, PermsTag>;

impl BitFlagsSpec<MyU8> for PermsTag {
    const VALID_BITS_MASK: MyU8 = MyU8(0b111);
    const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
    const COMBINED_NAMES: &[(MyU8, &str)] = &[];
    const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
}

const NONE: Perms = Perms::empty();

#[test]
fn test_flag_from() {
    for bit in 0..=255 {
        assert_eq!(Perm::from_bit_retain(bit).bit(), bit);
    }

    for bit in 0..=2 {
        assert_eq!(Perm::try_from_bit(bit).unwrap().bit(), bit);
    }
    for bit in 3..=255 {
        assert!(Perm::try_from_bit(bit).is_none());
    }
}

#[test]
fn test_flag_try_from() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(Perm::try_from_bits(MyU8(0)), None);
    assert_eq!(Perm::try_from_bits(MyU8(1)), Some(read));
    assert_eq!(Perm::try_from_bits(MyU8(2)), Some(write));
    assert_eq!(Perm::try_from_bits(MyU8(3)), None);
    assert_eq!(Perm::try_from_bits(MyU8(4)), Some(exec));
    assert_eq!(Perm::try_from_bits(MyU8(5)), None);
    assert_eq!(Perm::try_from_bits(MyU8(6)), None);
    assert_eq!(Perm::try_from_bits(MyU8(7)), None);
    assert_eq!(Perm::try_from_bits(MyU8(8)), None);

    assert_eq!(Perm::try_from_bits_retain(MyU8(0)), None);
    assert_eq!(Perm::try_from_bits_retain(MyU8(1)), Some(read));
    assert_eq!(Perm::try_from_bits_retain(MyU8(2)), Some(write));
    assert_eq!(Perm::try_from_bits_retain(MyU8(3)), None);
    assert_eq!(Perm::try_from_bits_retain(MyU8(4)), Some(exec));
    assert_eq!(Perm::try_from_bits_retain(MyU8(5)), None);
    assert_eq!(Perm::try_from_bits_retain(MyU8(6)), None);
    assert_eq!(Perm::try_from_bits_retain(MyU8(7)), None);
    assert_eq!(Perm::try_from_bits_retain(MyU8(8)).unwrap().bit(), 3);
    assert_eq!(Perm::try_from_bits_retain(MyU8(16)).unwrap().bit(), 4);
    assert_eq!(Perm::try_from_bits_retain(MyU8(32)).unwrap().bit(), 5);

    assert_eq!(Perm::try_from_flags((read | write) - (read | write)), None);
    assert_eq!(Perm::try_from_flags(read.into()), Some(read));
    assert_eq!(Perm::try_from_flags(write.into()), Some(write));
    assert_eq!(Perm::try_from_flags(read | write), None);
    assert_eq!(Perm::try_from_flags(exec.into()), Some(exec));
    assert_eq!(Perm::try_from_flags(read | exec), None);
    assert_eq!(Perm::try_from_flags(write | exec), None);
    assert_eq!(Perm::try_from_flags(read | write | exec), None);
    assert_eq!(
        Perm::try_from_flags(Perms::from_bits_retain(MyU8(0x8))),
        None
    );

    assert_eq!(
        Perm::try_from((read | write) - (read | write)),
        Err(BitFlagFromFlagsError)
    );
    assert_eq!(Perm::try_from(Perms::from(read)), Ok(read));
    assert_eq!(Perm::try_from(Perms::from(write)), Ok(write));
    assert_eq!(Perm::try_from(read | write), Err(BitFlagFromFlagsError));
    assert_eq!(Perm::try_from(Perms::from(exec)), Ok(exec));
    assert_eq!(Perm::try_from(read | exec), Err(BitFlagFromFlagsError));
    assert_eq!(Perm::try_from(write | exec), Err(BitFlagFromFlagsError));
    assert_eq!(
        Perm::try_from(read | write | exec),
        Err(BitFlagFromFlagsError)
    );
    assert_eq!(
        Perm::try_from(Perms::from_bits_retain(MyU8(0x8))),
        Err(BitFlagFromFlagsError)
    );

    assert_eq!(
        Perm::try_from((read | write) - (read | write))
            .err()
            .unwrap()
            .to_string(),
        "flags do not contain exactly one valid bit"
    );
}

#[test]
fn test_flag_name() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(read.name(), Some("read"));
    assert_eq!(write.name(), Some("write"));
    assert_eq!(exec.name(), Some("exec"));
    assert_eq!(Perm::from_bit_retain(3).name(), None);

    assert_eq!(read.name_and_aliases(), &["read"]);
    assert_eq!(write.name_and_aliases(), &["write"]);
    assert_eq!(exec.name_and_aliases(), &["exec", "execute"]);
    assert_eq!(Perm::from_bit_retain(3).name_and_aliases(), &[""; 0]);
}

#[test]
fn test_flag_predicates() {
    for bits in 0..=2 {
        assert!(Perm::from_bit_retain(bits).is_valid());
    }
    for bits in 3..=0xFF {
        assert!(!Perm::from_bit_retain(bits).is_valid());
    }
}

#[test]
fn test_flag_mask() {
    assert_eq!(Perm::from_bit_retain(0).mask(), Some(MyU8(0x01)));
    assert_eq!(Perm::from_bit_retain(1).mask(), Some(MyU8(0x02)));
    assert_eq!(Perm::from_bit_retain(2).mask(), Some(MyU8(0x04)));
    assert_eq!(Perm::from_bit_retain(3).mask(), Some(MyU8(0x08)));
    assert_eq!(Perm::from_bit_retain(4).mask(), Some(MyU8(0x10)));
    assert_eq!(Perm::from_bit_retain(5).mask(), Some(MyU8(0x20)));
    assert_eq!(Perm::from_bit_retain(6).mask(), Some(MyU8(0x40)));
    assert_eq!(Perm::from_bit_retain(7).mask(), Some(MyU8(0x80)));
    assert_eq!(Perm::from_bit_retain(8).mask(), None);
    assert_eq!(Perm::from_bit_retain(9).mask(), None);
}

#[test]
fn test_flag_math_ops() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(read.intersection(read), Perms::from(read));
    assert_eq!(read.intersection(write), NONE);

    assert_eq!(read.union(read), Perms::from(read));
    assert_eq!(read.union(write), Perms::from(read | write));

    assert_eq!(read.difference(read), NONE);
    assert_eq!(read.difference(write), Perms::from(read));

    assert_eq!(read.symmetric_difference(read), NONE);
    assert_eq!(read.symmetric_difference(write), Perms::from(read | write));

    assert_eq!(read.complement(), Perms::from(write | exec));
    assert_eq!(write.complement(), Perms::from(read | exec));
}

#[test]
fn test_flag_basic_traits() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(read.clone(), read);
    #[expect(clippy::manual_assert_eq, reason = "explicit equality test")]
    (assert!(read == read));
    assert_eq!(read.cmp(&read), Ordering::Equal);
    assert_eq!(read.partial_cmp(&read), Some(Ordering::Equal));

    assert!(read < write);
    assert_eq!(read.cmp(&write), Ordering::Less);
    assert_eq!(read.partial_cmp(&write), Some(Ordering::Less));

    let mut set = [
        Perm::from_bit_retain(4),
        read,
        write,
        exec,
        read,
        write,
        exec,
        write,
        exec,
        exec,
    ];
    set.sort_unstable();
    assert_eq!(
        set,
        [
            read,
            read,
            write,
            write,
            write,
            exec,
            exec,
            exec,
            exec,
            Perm::from_bit_retain(4),
        ]
    );
}
