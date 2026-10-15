//! Tests for `flags_ext.rs` file.

#![expect(clippy::unwrap_used, reason = "okay in tests")]
#![allow(unused_crate_dependencies, reason = "okay in tests")]

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt::{self, LowerHex};
use core::hash::{Hash, Hasher};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use std::collections::HashSet;
use std::hash::DefaultHasher;

use bitflags_lite::{
    BitFlag, BitFlagExt, BitFlags, BitFlagsExt, BitFlagsIter, BitFlagsSpec, NameMatcher,
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
    const VALID_BITS_MASK: MyU8 = MyU8((1 << 3) - 1);
    const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
    const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
}

const NONE: Perms = Perms::empty();

#[test]
fn test_flags_ext_empty() {
    let flags = Perms::empty();
    assert!(flags.is_empty());
    assert_eq!(flags.bits(), MyU8(0));
    assert_eq!(flags.into_iter().len(), 0);
}

#[test]
fn test_flags_ext_all_valid() {
    let flags = Perms::all_valid();
    assert!(!flags.is_empty());
    assert_eq!(flags.bits(), MyU8(0b111));
    assert_eq!(flags.into_iter().len(), 3);
}

#[test]
fn test_flags_ext_into_bits() {
    for bits in 0..=255 {
        assert_eq!(Perms::from_bits_retain(MyU8(bits)).into_bits(), MyU8(bits));
    }
}

#[test]
fn test_flags_ext_from() {
    for bit in 0..=255 {
        let flag = Perm::from_bit_retain(bit);
        assert_eq!(Perms::from_flag(flag), Perms::from(flag));
    }
}

#[test]
fn test_flags_ext_try_from() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(Perms::try_from_bits(MyU8(0b1)), Some(read.into()));
    assert_eq!(Perms::try_from_bits(MyU8(0b10)), Some(write.into()));
    assert_eq!(Perms::try_from_bits(MyU8(0b11)), Some(read | write));
    assert_eq!(Perms::try_from_bits(MyU8(0b100)), Some(exec.into()));
    assert_eq!(Perms::try_from_bits(MyU8(0b110)), Some(write | exec));
    assert_eq!(Perms::try_from_bits(MyU8(0b111)), Some(read | write | exec));
    assert_eq!(Perms::try_from_bits(MyU8(0b1000)), None);
    assert_eq!(Perms::try_from_bits(MyU8(0b1100)), None);
    assert_eq!(Perms::try_from_bits(MyU8(0b1110)), None);
    assert_eq!(Perms::try_from_bits(MyU8(0b1111)), None);

    assert_eq!(Perms::try_from_bit(0), Some(read.into()));
    assert_eq!(Perms::try_from_bit(1), Some(write.into()));
    assert_eq!(Perms::try_from_bit(2), Some(exec.into()));
    assert_eq!(Perms::try_from_bit(3), None);
    assert_eq!(Perms::try_from_bit(4), None);
    assert_eq!(Perms::try_from_bit(5), None);
    assert_eq!(Perms::try_from_bit(6), None);
    assert_eq!(Perms::try_from_bit(7), None);
    assert_eq!(Perms::try_from_bit(8), None);
}

#[test]
fn test_flags_ext_from_truncate() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(Perms::from_bits_truncate(MyU8(0b1)), read);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b10)), write);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b11)), read | write);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b100)), exec);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b110)), write | exec);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b111)), read | write | exec);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b1000)), NONE);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b1100)), exec);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b1110)), write | exec);
    assert_eq!(Perms::from_bits_truncate(MyU8(0b1111)), read | write | exec);

    assert_eq!(Perms::from_bit_truncate(0), read);
    assert_eq!(Perms::from_bit_truncate(1), write);
    assert_eq!(Perms::from_bit_truncate(2), exec);
    assert_eq!(Perms::from_bit_truncate(3), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(4), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(5), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(6), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(7), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(8), Perms::empty());
}

#[test]
fn test_flags_ext_from_retain() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(Perms::try_from_bits(MyU8(0b1000)), None);
    let bit4 = Perms::from_bits_retain(MyU8(0b1000));
    assert_eq!(bit4.bits(), MyU8(0b1000));

    assert_eq!(Perms::from_bits_retain(MyU8(0b1)), read);
    assert_eq!(Perms::from_bits_retain(MyU8(0b10)), write);
    assert_eq!(Perms::from_bits_retain(MyU8(0b11)), read | write);
    assert_eq!(Perms::from_bits_retain(MyU8(0b100)), exec);
    assert_eq!(Perms::from_bits_retain(MyU8(0b110)), write | exec);
    assert_eq!(Perms::from_bits_retain(MyU8(0b111)), read | write | exec);
    assert_eq!(Perms::from_bits_retain(MyU8(0b1000)), NONE | bit4);
    assert_eq!(Perms::from_bits_retain(MyU8(0b1100)), exec | bit4);
    assert_eq!(Perms::from_bits_retain(MyU8(0b1110)), write | exec | bit4);
    assert_eq!(
        Perms::from_bits_retain(MyU8(0b1111)),
        read | write | exec | bit4
    );

    assert_eq!(Perms::from_bit_retain(0), read);
    assert_eq!(Perms::from_bit_retain(1), write);
    assert_eq!(Perms::from_bit_retain(2), exec);
    assert_eq!(
        Perms::from_bit_retain(3),
        Perms::from_bits_retain(MyU8(0x08))
    );
    assert_eq!(
        Perms::from_bit_retain(4),
        Perms::from_bits_retain(MyU8(0x10))
    );
    assert_eq!(
        Perms::from_bit_retain(5),
        Perms::from_bits_retain(MyU8(0x20))
    );
    assert_eq!(
        Perms::from_bit_retain(6),
        Perms::from_bits_retain(MyU8(0x40))
    );
    assert_eq!(
        Perms::from_bit_retain(7),
        Perms::from_bits_retain(MyU8(0x80))
    );
    assert_eq!(Perms::from_bit_retain(8), Perms::empty());
}

#[test]
fn test_flags_ext_predicates() {
    assert!(Perms::from_bits_retain(MyU8(0)).is_empty());
    for bits in 1..=0xFF {
        assert!(!Perms::from_bits_retain(MyU8(bits)).is_empty());
    }

    for bits in 0..=0b111 {
        assert!(Perms::from_bits_retain(MyU8(bits)).is_valid());
    }
    for bits in 0b1000..=0xFF {
        assert!(!Perms::from_bits_retain(MyU8(bits)).is_valid());
    }
}

#[test]
fn test_flags_ext_intersects() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert!(Perms::from(read).intersects(read.into()));
    assert!(!Perms::from(read).intersects(write.into()));
    assert!((read | write).intersects(read | write));
    assert!((read | write).intersects(write | exec));
    assert!(!Perms::from(read).intersects(write | exec));
}

#[test]
fn test_flags_ext_contains() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert!(Perms::from(read).contains(read.into()));
    assert!(!Perms::from(read).contains(write.into()));
    assert!((read | write).contains(read | write));
    assert!(!(read | write).contains(write | exec));
    assert!(!Perms::from(read).contains(write | exec));
    assert!((read | write | exec).contains(write | exec));
}

#[test]
fn test_flags_ext_truncate() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let mut flags = Perms::from_bits_retain(MyU8(0b1111_1111));
    assert_eq!(
        flags,
        Perms::from_bits_retain(MyU8(0b1111_1000)) | read | write | exec
    );
    flags.truncate();
    assert_eq!(flags, read | write | exec);

    let mut flags = Perms::from_bits_retain(MyU8(0b1111_1101));
    assert_eq!(
        flags,
        Perms::from_bits_retain(MyU8(0b1111_1000)) | read | exec
    );
    flags.truncate();
    assert_eq!(flags, read | exec);
}

#[test]
fn test_flags_ext_insert_remove_toggle_set() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let mut flags = NONE;

    flags.insert(read.into());
    assert_eq!(flags, read);
    flags.insert(read.into());
    assert_eq!(flags, read);
    flags.remove(read.into());
    assert_eq!(flags, NONE);
    flags.remove(read.into());
    assert_eq!(flags, NONE);
    flags.toggle(read.into());
    assert_eq!(flags, read);

    flags.insert(write.into());
    assert_eq!(flags, read | write);
    flags.remove(write.into());
    assert_eq!(flags, read);
    flags.toggle(write.into());
    assert_eq!(flags, read | write);
    flags.toggle(read.into());
    assert_eq!(flags, write);

    flags.set(exec.into(), false);
    assert_eq!(flags, write);
    flags.set(exec.into(), true);
    assert_eq!(flags, write | exec);

    flags.toggle(read | write | exec);
    assert_eq!(flags, read);
    flags.clear();
    assert_eq!(flags, NONE);
}

#[test]
fn test_flags_ext_intersection() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(NONE.intersection(read.into()), NONE);
    assert_eq!(Perms::from(read).intersection(read.into()), read);
    assert_eq!(Perms::from(read).intersection(write.into()), NONE);
    assert_eq!((read | write).intersection(read | write), read | write);
    assert_eq!((read | write).intersection(write | exec), write);
    assert_eq!(Perms::from(read).intersection(write | exec), NONE);
}

#[test]
fn test_flags_ext_union() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(NONE.union(read.into()), read);
    assert_eq!(Perms::from(read).union(read.into()), read);
    assert_eq!(Perms::from(read).union(write.into()), read | write);
    assert_eq!((read | write).union(read | write), read | write);
    assert_eq!((read | write).union(write | exec), read | write | exec);
    assert_eq!(Perms::from(read).union(write | exec), read | write | exec);
}

#[test]
fn test_flags_ext_difference() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(NONE.difference(read.into()), NONE);
    assert_eq!(Perms::from(read).difference(read.into()), NONE);
    assert_eq!(Perms::from(read).difference(write.into()), read);
    assert_eq!((read | write).difference(read | write), NONE);
    assert_eq!((read | write).difference(write | exec), read);
    assert_eq!(Perms::from(read).difference(write | exec), read);
}

#[test]
fn test_flags_ext_symmetric_difference() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(NONE.symmetric_difference(read.into()), read);
    assert_eq!(Perms::from(read).symmetric_difference(read.into()), NONE);
    assert_eq!(
        Perms::from(read).symmetric_difference(write.into()),
        read | write
    );
    assert_eq!((read | write).symmetric_difference(read | write), NONE);
    assert_eq!(
        (read | write).symmetric_difference(write | exec),
        read | exec
    );
    assert_eq!(
        Perms::from(read).symmetric_difference(write | exec),
        read | write | exec
    );
}

#[test]
fn test_flags_ext_complement() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(NONE.complement(), read | write | exec);
    assert_eq!(Perms::from(read).complement(), write | exec);
    assert_eq!(Perms::from(write).complement(), read | exec);
    assert_eq!(Perms::from(exec).complement(), read | write);
    assert_eq!((read | write).complement(), exec);
    assert_eq!((write | exec).complement(), read);
    assert_eq!((read | exec).complement(), write);
    assert_eq!((read | write | exec).complement(), NONE);
}

#[test]
fn test_flags_ext_iterator() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let mut rwx = read | write | exec;
    assert_eq!(rwx.into_iter().len(), 3);
    assert_eq!(rwx.into_iter().size_hint(), (3, Some(3)));
    assert_eq!(rwx.into_iter().collect::<Vec<_>>(), vec![read, write, exec]);
    assert_eq!(
        rwx.into_iter().rev().collect::<Vec<_>>(),
        vec![exec, write, read]
    );

    assert_eq!(
        (&rwx).into_iter().rev().collect::<Vec<_>>(),
        vec![exec, write, read]
    );
    assert_eq!(
        (&mut rwx).into_iter().rev().collect::<Vec<_>>(),
        vec![exec, write, read]
    );

    let rw = read | write;
    assert_eq!(rw.iter().len(), 2);
    assert_eq!(rw.iter().size_hint(), (2, Some(2)));
    assert_eq!(rw.iter().collect::<Vec<_>>(), vec![read, write]);
    assert_eq!(rw.iter().rev().collect::<Vec<_>>(), vec![write, read]);

    let none = NONE;
    assert_eq!(none.iter().len(), 0);
    assert_eq!(none.iter().size_hint(), (0, Some(0)));
    assert_eq!(none.iter().collect::<Vec<_>>(), vec![NONE; 0]);
    assert_eq!(none.iter().rev().collect::<Vec<_>>(), vec![NONE; 0]);
}

#[test]
fn test_flags_ext_extend() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let mut rwx = NONE;
    rwx.extend([read, exec]);
    assert_eq!(rwx.bits(), MyU8(0b101));
    rwx.extend([
        BitFlags::<MyU8, _>::from_bits_retain(MyU8(0b1000)),
        BitFlags::<MyU8, _>::from_bits_retain(MyU8(0b1000_0000)),
    ]);
    assert_eq!(rwx.bits(), MyU8(0b1000_1101));
}

#[test]
fn test_flags_ext_clone() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let rwx = read | write | exec;
    #[expect(clippy::clone_on_copy, reason = "explicit clone test")]
    let rwx = rwx.clone();
    assert_eq!(rwx.bits(), MyU8(0b111));

    let mut iter1 = rwx.into_iter();
    assert_eq!(iter1.next(), Some(read));
    let iter2 = iter1.clone();

    assert_eq!(iter1.collect::<Vec<_>>(), [write, exec]);
    assert_eq!(iter2.collect::<Vec<_>>(), [write, exec]);
}

#[test]
fn test_flags_ext_cmp() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!((read | write) - write, read);
    assert_eq!(read, (read | write) - write);
    assert!(Perms::from(read) < Perms::from(write));
    assert!(Perms::from(write) < Perms::from(exec));
    assert!(read < Perms::from(write));
    assert!(Perms::from(read) < write);

    assert_eq!(Perms::from(read).cmp(&Perms::from(write)), Ordering::Less);
    assert_eq!(Perms::from(read).partial_cmp(&write), Some(Ordering::Less));
    assert_eq!(read.partial_cmp(&Perms::from(write)), Some(Ordering::Less));

    let mut set = [
        Perms::from_bits_retain(MyU8(0b1000)),
        Perms::from_bits_retain(MyU8(0b1010)),
        read.into(),
        write.into(),
        exec.into(),
        read | write,
        write | exec,
        exec | read,
        read.into(),
        write.into(),
        exec.into(),
        read | write | exec,
        NONE,
    ];
    set.sort_unstable();
    assert_eq!(
        set,
        [
            NONE,
            read.into(),
            read.into(),
            write.into(),
            write.into(),
            read | write,
            exec.into(),
            exec.into(),
            read | exec,
            write | exec,
            read | write | exec,
            Perms::from_bits_retain(MyU8(0b1000)),
            Perms::from_bits_retain(MyU8(0b1010)),
        ]
    );
}

#[test]
fn test_flags_ext_hash() {
    fn hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(hash(&read), hash(&read));
    assert_ne!(hash(&read), hash(&write));
    assert_ne!(hash(&read), hash(&Perms::from_bits_retain(MyU8(0b1000))));

    let input = [
        read.into(),
        write | exec,
        read.into(),
        read | write | exec,
        Perms::from_bits_retain(MyU8(0b1000)),
        Perms::from_bits_retain(MyU8(0b1010)),
        read | write | exec,
    ];

    let set: HashSet<_> = input.into_iter().collect();

    assert_eq!(set.len(), 5);
    assert!(set.contains(&Perms::from(read)));
    assert!(set.contains(&(write | exec)));
    assert!(set.contains(&(read | write | exec)));
    assert!(set.contains(&Perms::from_bits_retain(MyU8(0b1000))));
    assert!(set.contains(&Perms::from_bits_retain(MyU8(0b1010))));
    assert!(!set.contains(&NONE));
    assert!(!set.contains(&Perms::from(write)));
    assert!(!set.contains(&(read | write)));
    assert!(!set.contains(&(read | exec)));
}

#[test]
fn test_flags_ext_default() {
    let flags = Perms::default();
    assert!(flags.is_empty());
    assert_eq!(flags.bits(), MyU8(0));

    let mut iter = BitFlagsIter::<MyU8, PermsTag>::default();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_flags_ext_as_ref() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let rwx = read | write | exec;
    assert_eq!(rwx.as_ref(), &MyU8(0b111));
    assert_eq!(Borrow::<MyU8>::borrow(&rwx), &MyU8(0b111));
}

#[test]
fn test_flags_ext_ops() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(!NONE, read | write | exec);
    assert_eq!(!!NONE, NONE);
    assert_eq!(!read, write | exec);
    assert_eq!(!read & !write, exec);
    assert_eq!(read ^ write, read | write);
    assert_eq!((read | write) & (write | exec), write);
    assert_eq!((read | write) ^ (write | exec), read | exec);
    assert_eq!((read | write) - (write | exec), read);
}

#[test]
fn test_flags_ext_assign() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let rwx = read | write | exec;

    assert_eq!(rwx & rwx, read | write | exec);
    assert_eq!(rwx & read, read);
    assert_eq!(read & rwx, read);

    assert_eq!(rwx | rwx, read | write | exec);
    assert_eq!(rwx | read, read | write | exec);
    assert_eq!(read | rwx, read | write | exec);

    assert_eq!(rwx ^ rwx, NONE);
    assert_eq!(rwx ^ read, write | exec);
    assert_eq!(read ^ rwx, write | exec);

    assert_eq!(rwx - rwx, NONE);
    assert_eq!(rwx - read, write | exec);
}

#[test]
fn test_flags_ext_assign_flags_ops() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let mut flags = read | write | exec;
    flags &= read | write;
    assert_eq!(flags, read | write);
    flags &= write | exec;
    assert_eq!(flags, write);

    let mut flags = read | write;
    flags |= read | write;
    assert_eq!(flags, read | write);
    flags |= write | exec;
    assert_eq!(flags, read | write | exec);

    let mut flags = read | write | exec;
    flags ^= read | write;
    assert_eq!(flags, exec);
    flags ^= write | exec;
    assert_eq!(flags, write);

    let mut flags = read | write | exec;
    flags -= read | write;
    assert_eq!(flags, exec);
    flags -= write | exec;
    assert_eq!(flags, NONE);
}

#[test]
fn test_flags_ext_assign_flag_ops() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    let mut flags = read | write | exec;
    flags &= read;
    assert_eq!(flags, read);
    flags &= write;
    assert_eq!(flags, NONE);

    let mut flags = read | write;
    flags |= read;
    assert_eq!(flags, read | write);
    flags |= exec;
    assert_eq!(flags, read | write | exec);

    let mut flags = read | write | exec;
    flags ^= read;
    assert_eq!(flags, write | exec);
    flags ^= read;
    assert_eq!(flags, read | write | exec);

    let mut flags = read | write | exec;
    flags -= read;
    assert_eq!(flags, write | exec);
    flags -= read;
    assert_eq!(flags, write | exec);
}

#[test]
fn test_flags_ext_collect() {
    let read: Perm = Perm::try_from_bit(0).unwrap();
    let write: Perm = Perm::try_from_bit(1).unwrap();
    let exec: Perm = Perm::try_from_bit(2).unwrap();

    assert_eq!(
        [Perms::from(read), read.into(), exec.into()]
            .into_iter()
            .collect::<Perms>(),
        read | exec
    );

    assert_eq!(
        [read, read, exec].into_iter().collect::<Perms>(),
        read | exec
    );

    assert_eq!(
        [
            Perms::from(read),
            NONE,
            write.into(),
            NONE,
            Perms::from_bits_retain(MyU8(0b1000))
        ]
        .into_iter()
        .collect::<Perms>(),
        read | write | Perms::from_bits_retain(MyU8(0b1000))
    );
}
