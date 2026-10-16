//! Tests for `flags.rs` and `flag.rs` files.

#![expect(clippy::unwrap_used, reason = "okay in tests")]
#![allow(unused_crate_dependencies, reason = "okay in tests")]

use core::cmp::Ordering;
use core::fmt::{self, Write};
use core::hash::{Hash, Hasher};
use core::str::FromStr;
use std::collections::HashSet;
use std::hash::DefaultHasher;

use bitflags_lite::{BitFlag, BitFlags, BitFlagsIter, BitFlagsSpec, NameMatcher};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct PermsTag;
type Perm = BitFlag<u8, PermsTag>;
type Perms = BitFlags<u8, PermsTag>;

impl BitFlagsSpec<u8> for PermsTag {
    const VALID_BITS_MASK: u8 = 0b111;
    const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
    const COMBINED_NAMES: &[(u8, &str)] = &[(0, "none"), (0b111, "rwx")];
    const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
}

const NONE: Perms = Perms::empty();
const READ: Perm = Perm::try_from_bit(0).unwrap();
const WRITE: Perm = Perm::try_from_bit(1).unwrap();
const EXEC: Perm = Perm::try_from_bit(2).unwrap();

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct UnnamedPermsTag;
type UnnamedPerm = BitFlag<u8, UnnamedPermsTag>;
type UnnamedPerms = BitFlags<u8, UnnamedPermsTag>;

impl BitFlagsSpec<u8> for UnnamedPermsTag {
    const VALID_BITS_MASK: u8 = 0b111;
    const FLAG_NAMES: &[&[&str]] = &[];
    const COMBINED_NAMES: &[(u8, &str)] = &[];
    const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
}

const U_READ: UnnamedPerm = UnnamedPerm::try_from_bit(0).unwrap();
const U_WRITE: UnnamedPerm = UnnamedPerm::try_from_bit(1).unwrap();
const U_EXEC: UnnamedPerm = UnnamedPerm::try_from_bit(2).unwrap();

#[test]
fn test_flags_empty() {
    let flags = Perms::empty();
    assert!(flags.is_empty());
    assert_eq!(flags.bits(), 0);
    assert_eq!(flags.into_iter().len(), 0);
}

#[test]
fn test_flags_all_valid() {
    let flags = Perms::all_valid();
    assert!(!flags.is_empty());
    assert_eq!(flags.bits(), 0b111);
    assert_eq!(flags.into_iter().len(), 3);
}

#[test]
fn test_flags_into_bits() {
    for bits in 0..=255 {
        assert_eq!(Perms::from_bits_retain(bits).into_bits(), bits);
    }
}

#[test]
fn test_flags_from() {
    let rwx = READ | WRITE | EXEC;
    assert_eq!(u8::from(rwx), 0b111);

    let rx = READ | EXEC;
    assert_eq!(u8::from(rx), 0b101);

    let flags = Perms::from_bits_retain(0b1111_1111);
    assert_eq!(u8::from(flags), 0b1111_1111);

    for bit in 0..=255 {
        let flag = Perm::from_bit_retain(bit);
        assert_eq!(Perms::from_flag(flag), Perms::from(flag));
    }
}

#[test]
fn test_flags_try_from() {
    assert_eq!(Perms::try_from_bits(0b1), Some(READ.into()));
    assert_eq!(Perms::try_from_bits(0b10), Some(WRITE.into()));
    assert_eq!(Perms::try_from_bits(0b11), Some(READ | WRITE));
    assert_eq!(Perms::try_from_bits(0b100), Some(EXEC.into()));
    assert_eq!(Perms::try_from_bits(0b110), Some(WRITE | EXEC));
    assert_eq!(Perms::try_from_bits(0b111), Some(READ | WRITE | EXEC));
    assert_eq!(Perms::try_from_bits(0b1000), None);
    assert_eq!(Perms::try_from_bits(0b1100), None);
    assert_eq!(Perms::try_from_bits(0b1110), None);
    assert_eq!(Perms::try_from_bits(0b1111), None);

    assert_eq!(Perms::try_from_bit(0), Some(READ.into()));
    assert_eq!(Perms::try_from_bit(1), Some(WRITE.into()));
    assert_eq!(Perms::try_from_bit(2), Some(EXEC.into()));
    assert_eq!(Perms::try_from_bit(3), None);
    assert_eq!(Perms::try_from_bit(4), None);
    assert_eq!(Perms::try_from_bit(5), None);
    assert_eq!(Perms::try_from_bit(6), None);
    assert_eq!(Perms::try_from_bit(7), None);
    assert_eq!(Perms::try_from_bit(8), None);
}

#[test]
fn test_flags_from_truncate() {
    assert_eq!(Perms::from_bits_truncate(0b1), READ);
    assert_eq!(Perms::from_bits_truncate(0b10), WRITE);
    assert_eq!(Perms::from_bits_truncate(0b11), READ | WRITE);
    assert_eq!(Perms::from_bits_truncate(0b100), EXEC);
    assert_eq!(Perms::from_bits_truncate(0b110), WRITE | EXEC);
    assert_eq!(Perms::from_bits_truncate(0b111), READ | WRITE | EXEC);
    assert_eq!(Perms::from_bits_truncate(0b1000), NONE);
    assert_eq!(Perms::from_bits_truncate(0b1100), EXEC);
    assert_eq!(Perms::from_bits_truncate(0b1110), WRITE | EXEC);
    assert_eq!(Perms::from_bits_truncate(0b1111), READ | WRITE | EXEC);

    assert_eq!(Perms::from_bit_truncate(0), READ);
    assert_eq!(Perms::from_bit_truncate(1), WRITE);
    assert_eq!(Perms::from_bit_truncate(2), EXEC);
    assert_eq!(Perms::from_bit_truncate(3), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(4), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(5), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(6), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(7), Perms::empty());
    assert_eq!(Perms::from_bit_truncate(8), Perms::empty());
}

#[test]
fn test_flags_from_retain() {
    assert_eq!(Perms::try_from_bits(0b1000), None);
    let bit4 = Perms::from_bits_retain(0b1000);
    assert_eq!(bit4.bits(), 0b1000);

    assert_eq!(Perms::from_bits_retain(0b1), READ);
    assert_eq!(Perms::from_bits_retain(0b10), WRITE);
    assert_eq!(Perms::from_bits_retain(0b11), READ | WRITE);
    assert_eq!(Perms::from_bits_retain(0b100), EXEC);
    assert_eq!(Perms::from_bits_retain(0b110), WRITE | EXEC);
    assert_eq!(Perms::from_bits_retain(0b111), READ | WRITE | EXEC);
    assert_eq!(Perms::from_bits_retain(0b1000), NONE | bit4);
    assert_eq!(Perms::from_bits_retain(0b1100), EXEC | bit4);
    assert_eq!(Perms::from_bits_retain(0b1110), WRITE | EXEC | bit4);
    assert_eq!(Perms::from_bits_retain(0b1111), READ | WRITE | EXEC | bit4);

    assert_eq!(Perms::from_bit_retain(0), READ);
    assert_eq!(Perms::from_bit_retain(1), WRITE);
    assert_eq!(Perms::from_bit_retain(2), EXEC);
    assert_eq!(Perms::from_bit_retain(3), Perms::from_bits_retain(0x08));
    assert_eq!(Perms::from_bit_retain(4), Perms::from_bits_retain(0x10));
    assert_eq!(Perms::from_bit_retain(5), Perms::from_bits_retain(0x20));
    assert_eq!(Perms::from_bit_retain(6), Perms::from_bits_retain(0x40));
    assert_eq!(Perms::from_bit_retain(7), Perms::from_bits_retain(0x80));
    assert_eq!(Perms::from_bit_retain(8), Perms::empty());
}

#[test]
fn test_flags_predicates() {
    assert!(Perms::from_bits_retain(0).is_empty());
    for bits in 1..=0xFF {
        assert!(!Perms::from_bits_retain(bits).is_empty());
    }

    for bits in 0..=0b111 {
        assert!(Perms::from_bits_retain(bits).is_valid());
    }
    for bits in 0b1000..=0xFF {
        assert!(!Perms::from_bits_retain(bits).is_valid());
    }
}

#[test]
fn test_flags_intersects() {
    assert!(Perms::from(READ).intersects(READ.into()));
    assert!(!Perms::from(READ).intersects(WRITE.into()));
    assert!((READ | WRITE).intersects(READ | WRITE));
    assert!((READ | WRITE).intersects(WRITE | EXEC));
    assert!(!Perms::from(READ).intersects(WRITE | EXEC));
}

#[test]
fn test_flags_contains() {
    assert!(Perms::from(READ).contains(READ.into()));
    assert!(!Perms::from(READ).contains(WRITE.into()));
    assert!((READ | WRITE).contains(READ | WRITE));
    assert!(!(READ | WRITE).contains(WRITE | EXEC));
    assert!(!Perms::from(READ).contains(WRITE | EXEC));
    assert!((READ | WRITE | EXEC).contains(WRITE | EXEC));
}

#[test]
fn test_flags_truncate() {
    let mut flags = Perms::from_bits_retain(0b1111_1111);
    assert_eq!(
        flags,
        Perms::from_bits_retain(0b1111_1000) | READ | WRITE | EXEC
    );
    flags.truncate();
    assert_eq!(flags, READ | WRITE | EXEC);

    let mut flags = Perms::from_bits_retain(0b1111_1101);
    assert_eq!(flags, Perms::from_bits_retain(0b1111_1000) | READ | EXEC);
    flags.truncate();
    assert_eq!(flags, READ | EXEC);
}

#[test]
fn test_flags_insert_remove_toggle_set() {
    let mut flags = NONE;

    flags.insert(READ.into());
    assert_eq!(flags, READ);
    flags.insert(READ.into());
    assert_eq!(flags, READ);
    flags.remove(READ.into());
    assert_eq!(flags, NONE);
    flags.remove(READ.into());
    assert_eq!(flags, NONE);
    flags.toggle(READ.into());
    assert_eq!(flags, READ);

    flags.insert(WRITE.into());
    assert_eq!(flags, READ | WRITE);
    flags.remove(WRITE.into());
    assert_eq!(flags, READ);
    flags.toggle(WRITE.into());
    assert_eq!(flags, READ | WRITE);
    flags.toggle(READ.into());
    assert_eq!(flags, WRITE);

    flags.set(EXEC.into(), false);
    assert_eq!(flags, WRITE);
    flags.set(EXEC.into(), true);
    assert_eq!(flags, WRITE | EXEC);

    flags.toggle(READ | WRITE | EXEC);
    assert_eq!(flags, READ);
    flags.clear();
    assert_eq!(flags, NONE);
}

#[test]
fn test_flags_intersection() {
    assert_eq!(NONE.intersection(READ.into()), NONE);
    assert_eq!(Perms::from(READ).intersection(READ.into()), READ);
    assert_eq!(Perms::from(READ).intersection(WRITE.into()), NONE);
    assert_eq!((READ | WRITE).intersection(READ | WRITE), READ | WRITE);
    assert_eq!((READ | WRITE).intersection(WRITE | EXEC), WRITE);
    assert_eq!(Perms::from(READ).intersection(WRITE | EXEC), NONE);
}

#[test]
fn test_flags_union() {
    assert_eq!(NONE.union(READ.into()), READ);
    assert_eq!(Perms::from(READ).union(READ.into()), READ);
    assert_eq!(Perms::from(READ).union(WRITE.into()), READ | WRITE);
    assert_eq!((READ | WRITE).union(READ | WRITE), READ | WRITE);
    assert_eq!((READ | WRITE).union(WRITE | EXEC), READ | WRITE | EXEC);
    assert_eq!(Perms::from(READ).union(WRITE | EXEC), READ | WRITE | EXEC);
}

#[test]
fn test_flags_difference() {
    assert_eq!(NONE.difference(READ.into()), NONE);
    assert_eq!(Perms::from(READ).difference(READ.into()), NONE);
    assert_eq!(Perms::from(READ).difference(WRITE.into()), READ);
    assert_eq!((READ | WRITE).difference(READ | WRITE), NONE);
    assert_eq!((READ | WRITE).difference(WRITE | EXEC), READ);
    assert_eq!(Perms::from(READ).difference(WRITE | EXEC), READ);
}

#[test]
fn test_flags_symmetric_difference() {
    assert_eq!(NONE.symmetric_difference(READ.into()), READ);
    assert_eq!(Perms::from(READ).symmetric_difference(READ.into()), NONE);
    assert_eq!(
        Perms::from(READ).symmetric_difference(WRITE.into()),
        READ | WRITE
    );
    assert_eq!((READ | WRITE).symmetric_difference(READ | WRITE), NONE);
    assert_eq!(
        (READ | WRITE).symmetric_difference(WRITE | EXEC),
        READ | EXEC
    );
    assert_eq!(
        Perms::from(READ).symmetric_difference(WRITE | EXEC),
        READ | WRITE | EXEC
    );
}

#[test]
fn test_flags_complement() {
    assert_eq!(NONE.complement(), READ | WRITE | EXEC);
    assert_eq!(Perms::from(READ).complement(), WRITE | EXEC);
    assert_eq!(Perms::from(WRITE).complement(), READ | EXEC);
    assert_eq!(Perms::from(EXEC).complement(), READ | WRITE);
    assert_eq!((READ | WRITE).complement(), EXEC);
    assert_eq!((WRITE | EXEC).complement(), READ);
    assert_eq!((READ | EXEC).complement(), WRITE);
    assert_eq!((READ | WRITE | EXEC).complement(), NONE);
}

#[test]
fn test_flags_clone() {
    let rwx = READ | WRITE | EXEC;
    #[expect(clippy::clone_on_copy, reason = "explicit clone test")]
    let rwx = rwx.clone();
    assert_eq!(rwx.bits(), 0b111);

    let mut iter1 = rwx.into_iter();
    assert_eq!(iter1.next(), Some(READ));
    let iter2 = iter1.clone();

    assert_eq!(iter1.collect::<Vec<_>>(), [WRITE, EXEC]);
    assert_eq!(iter2.collect::<Vec<_>>(), [WRITE, EXEC]);
}

#[test]
fn test_flags_cmp() {
    assert_eq!((READ | WRITE) - WRITE, READ);
    assert_eq!(READ, (READ | WRITE) - WRITE);
    assert!(Perms::from(READ) < Perms::from(WRITE));
    assert!(Perms::from(WRITE) < Perms::from(EXEC));
    assert!(READ < Perms::from(WRITE));
    assert!(Perms::from(READ) < WRITE);

    assert_eq!(Perms::from(READ).cmp(&Perms::from(WRITE)), Ordering::Less);
    assert_eq!(Perms::from(READ).partial_cmp(&WRITE), Some(Ordering::Less));
    assert_eq!(READ.partial_cmp(&Perms::from(WRITE)), Some(Ordering::Less));

    let mut set = [
        Perms::from_bits_retain(0b1000),
        Perms::from_bits_retain(0b1010),
        READ.into(),
        WRITE.into(),
        EXEC.into(),
        READ | WRITE,
        WRITE | EXEC,
        EXEC | READ,
        READ.into(),
        WRITE.into(),
        EXEC.into(),
        READ | WRITE | EXEC,
        NONE,
    ];
    set.sort_unstable();
    assert_eq!(
        set,
        [
            NONE,
            READ.into(),
            READ.into(),
            WRITE.into(),
            WRITE.into(),
            READ | WRITE,
            EXEC.into(),
            EXEC.into(),
            READ | EXEC,
            WRITE | EXEC,
            READ | WRITE | EXEC,
            Perms::from_bits_retain(0b1000),
            Perms::from_bits_retain(0b1010),
        ]
    );
}

#[test]
fn test_flags_hash() {
    fn hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    assert_eq!(hash(&READ), hash(&READ));
    assert_ne!(hash(&READ), hash(&WRITE));
    assert_ne!(hash(&READ), hash(&Perms::from_bits_retain(0b1000)));

    let input = [
        READ.into(),
        WRITE | EXEC,
        READ.into(),
        READ | WRITE | EXEC,
        Perms::from_bits_retain(0b1000),
        Perms::from_bits_retain(0b1010),
        READ | WRITE | EXEC,
    ];

    let set: HashSet<_> = input.into_iter().collect();

    assert_eq!(set.len(), 5);
    assert!(set.contains(&Perms::from(READ)));
    assert!(set.contains(&(WRITE | EXEC)));
    assert!(set.contains(&(READ | WRITE | EXEC)));
    assert!(set.contains(&Perms::from_bits_retain(0b1000)));
    assert!(set.contains(&Perms::from_bits_retain(0b1010)));
    assert!(!set.contains(&NONE));
    assert!(!set.contains(&Perms::from(WRITE)));
    assert!(!set.contains(&(READ | WRITE)));
    assert!(!set.contains(&(READ | EXEC)));
}

#[test]
fn test_flags_default() {
    let flags = Perms::default();
    assert!(flags.is_empty());
    assert_eq!(flags.bits(), 0);

    let mut iter = BitFlagsIter::<u8, PermsTag>::default();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_flags_as_ref() {
    use core::borrow::Borrow;

    let rwx = READ | WRITE | EXEC;
    assert_eq!(rwx.as_ref(), &0b111);
    assert_eq!(Borrow::<u8>::borrow(&rwx), &0b111);
}

#[test]
fn test_flags_bit_ops() {
    let rwx = READ | WRITE | EXEC;

    assert_eq!(!rwx, NONE);
    assert_eq!(!!rwx, READ | WRITE | EXEC);

    assert_eq!(rwx & rwx, READ | WRITE | EXEC);
    assert_eq!(rwx & READ, READ);
    assert_eq!(READ & rwx, READ);
    assert_eq!(READ & WRITE, NONE);

    assert_eq!(rwx | rwx, READ | WRITE | EXEC);
    assert_eq!(rwx | READ, READ | WRITE | EXEC);
    assert_eq!(READ | rwx, READ | WRITE | EXEC);
    assert_eq!(READ | WRITE, READ | WRITE);

    assert_eq!(rwx ^ rwx, NONE);
    assert_eq!(rwx ^ READ, WRITE | EXEC);
    assert_eq!(READ ^ rwx, WRITE | EXEC);
    assert_eq!(READ ^ WRITE, READ | WRITE);

    assert_eq!(rwx - rwx, NONE);
    assert_eq!(rwx - READ, WRITE | EXEC);
    assert_eq!(READ - rwx, NONE);
}

#[test]
fn test_flags_bit_assign_ops() {
    let mut flags = READ | WRITE | EXEC;
    flags &= READ | WRITE;
    assert_eq!(flags, READ | WRITE);
    flags &= WRITE | EXEC;
    assert_eq!(flags, WRITE);

    let mut flags = READ | WRITE;
    flags |= READ | WRITE;
    assert_eq!(flags, READ | WRITE);
    flags |= WRITE | EXEC;
    assert_eq!(flags, READ | WRITE | EXEC);

    let mut flags = READ | WRITE | EXEC;
    flags ^= READ | WRITE;
    assert_eq!(flags, EXEC);
    flags ^= WRITE | EXEC;
    assert_eq!(flags, WRITE);

    let mut flags = READ | WRITE | EXEC;
    flags -= READ | WRITE;
    assert_eq!(flags, EXEC);
    flags -= WRITE | EXEC;
    assert_eq!(flags, NONE);
}

#[test]
fn test_flags_assign_flag_ops() {
    let mut flags = READ | WRITE | EXEC;
    flags &= READ;
    assert_eq!(flags, READ);
    flags &= WRITE;
    assert_eq!(flags, NONE);

    let mut flags = READ | WRITE;
    flags |= READ;
    assert_eq!(flags, READ | WRITE);
    flags |= EXEC;
    assert_eq!(flags, READ | WRITE | EXEC);

    let mut flags = READ | WRITE | EXEC;
    flags ^= READ;
    assert_eq!(flags, WRITE | EXEC);
    flags ^= READ;
    assert_eq!(flags, READ | WRITE | EXEC);

    let mut flags = READ | WRITE | EXEC;
    flags -= READ;
    assert_eq!(flags, WRITE | EXEC);
    flags -= READ;
    assert_eq!(flags, WRITE | EXEC);
}

#[test]
fn test_flags_iterator() {
    let mut rwx = READ | WRITE | EXEC;
    assert_eq!(rwx.into_iter().len(), 3);
    assert_eq!(rwx.into_iter().size_hint(), (3, Some(3)));
    assert_eq!(rwx.into_iter().collect::<Vec<_>>(), vec![READ, WRITE, EXEC]);
    assert_eq!(
        rwx.into_iter().rev().collect::<Vec<_>>(),
        vec![EXEC, WRITE, READ]
    );

    assert_eq!(
        (&rwx).into_iter().rev().collect::<Vec<_>>(),
        vec![EXEC, WRITE, READ]
    );
    assert_eq!(
        (&mut rwx).into_iter().rev().collect::<Vec<_>>(),
        vec![EXEC, WRITE, READ]
    );

    let rw = READ | WRITE;
    assert_eq!(rw.iter().len(), 2);
    assert_eq!(rw.iter().size_hint(), (2, Some(2)));
    assert_eq!(rw.iter().collect::<Vec<_>>(), vec![READ, WRITE]);
    assert_eq!(rw.iter().rev().collect::<Vec<_>>(), vec![WRITE, READ]);

    let none = NONE;
    assert_eq!(none.iter().len(), 0);
    assert_eq!(none.iter().size_hint(), (0, Some(0)));
    assert_eq!(none.iter().collect::<Vec<_>>(), vec![NONE; 0]);
    assert_eq!(none.iter().rev().collect::<Vec<_>>(), vec![NONE; 0]);
}

#[test]
fn test_flags_extend() {
    let mut rwx = NONE;
    rwx.extend([READ, EXEC]);
    assert_eq!(rwx.bits(), 0b101);
    rwx.extend([
        BitFlags::<u8, _>::from_bits_retain(0b1000),
        BitFlags::<u8, _>::from_bits_retain(0b1000_0000),
    ]);
    assert_eq!(rwx.bits(), 0b1000_1101);
}

#[test]
fn test_flags_fmt_unnamed() {
    let rwx = U_READ | U_WRITE | U_EXEC | UnnamedPerms::from_bits_retain(0b1000);
    assert_eq!(format!("{rwx}"), "0xf");
    assert_eq!(format!("{rwx:#}"), "0xf");

    assert_eq!(format!("{rwx:b}"), "1111");
    assert_eq!(format!("{rwx:#b}"), "0b1111");
    assert_eq!(format!("{rwx:o}"), "17");
    assert_eq!(format!("{rwx:#o}"), "0o17");
    assert_eq!(format!("{rwx:x}"), "f");
    assert_eq!(format!("{rwx:#x}"), "0xf");
    assert_eq!(format!("{rwx:X}"), "F");
    assert_eq!(format!("{rwx:#X}"), "0xF");

    assert_eq!(
        format!("{rwx:?}"),
        "BitFlags(0xf, PhantomData<flags::UnnamedPermsTag>)"
    );
    assert_eq!(
        format!("{rwx:#?}"),
        "BitFlags(0xf, PhantomData<flags::UnnamedPermsTag>)"
    );
    assert_eq!(
        format!("{:?}", rwx.into_iter()),
        "BitFlagsIter(BitFlags(0xf, PhantomData<flags::UnnamedPermsTag>))"
    );
}

#[test]
fn test_flags_fmt_named() {
    let rwx = READ | WRITE | EXEC | Perms::from_bits_retain(0b1000);
    assert_eq!(format!("{rwx}"), "read | write | exec | 0x8");
    assert_eq!(format!("{rwx:#}"), "read | write | exec | 0x8");

    assert_eq!(format!("{rwx:b}"), "1111");
    assert_eq!(format!("{rwx:#b}"), "0b1111");
    assert_eq!(format!("{rwx:o}"), "17");
    assert_eq!(format!("{rwx:#o}"), "0o17");
    assert_eq!(format!("{rwx:x}"), "f");
    assert_eq!(format!("{rwx:#x}"), "0xf");
    assert_eq!(format!("{rwx:X}"), "F");
    assert_eq!(format!("{rwx:#X}"), "0xF");

    assert_eq!(
        format!("{rwx:?}"),
        "BitFlags(0xf, PhantomData<flags::PermsTag>)"
    );
    assert_eq!(
        format!("{rwx:#?}"),
        "BitFlags(0xf, PhantomData<flags::PermsTag>)"
    );
    assert_eq!(
        format!("{:?}", rwx.into_iter()),
        "BitFlagsIter(BitFlags(0xf, PhantomData<flags::PermsTag>))"
    );
}

#[test]
fn test_flags_from_str() {
    assert_eq!(Perms::from_str("").unwrap(), NONE);
    assert!(Perms::from_str(" | ").is_err());

    assert_eq!(Perms::from_str("read").unwrap(), READ);
    assert!(Perms::from_str("red").is_err());
    assert_eq!(
        Perms::from_str("read | write | exec | 0x8").unwrap(),
        READ | WRITE | EXEC | Perms::from_bits_retain(0b1000)
    );
    assert_eq!(Perms::from_str("none").unwrap(), NONE);
    assert_eq!(Perms::from_str("rwx").unwrap(), READ | WRITE | EXEC);
    assert_eq!(
        Perms::from_str("none | rwx | 0x8").unwrap(),
        READ | WRITE | EXEC | Perms::from_bits_retain(0b1000)
    );
    assert_eq!(
        Perms::from_str("0xD").unwrap(),
        READ | EXEC | Perms::from_bits_retain(0b1000)
    );

    assert!(Perms::from_str("0x100").is_err());
    assert!(Perms::from_str("0xG").is_err());
    assert!(Perms::from_str("0x+D").is_err());
}

#[test]
fn test_flags_from_case_sensitive_str() {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
    struct CsPermsTag;

    type CsPerms = BitFlags<u8, CsPermsTag>;

    impl BitFlagsSpec<u8> for CsPermsTag {
        const VALID_BITS_MASK: u8 = 0b111;
        const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
        const COMBINED_NAMES: &[(u8, &str)] = &[];
        const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
    }

    for valid in ["read", "  read  "] {
        assert_eq!(
            CsPerms::try_from_str(valid, "|", NameMatcher::Exact)
                .unwrap()
                .bits(),
            0b1
        );
    }

    for invalid in ["READ", "Read", "  ReAd  ", "readed", "readable", "r", "d"] {
        assert!(CsPerms::try_from_str(invalid, "|", NameMatcher::Exact).is_err());
    }
}

#[test]
fn test_flags_from_case_insensitive_str() {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
    struct CiPermsTag;

    type CiPerms = BitFlags<u8, CiPermsTag>;

    impl BitFlagsSpec<u8> for CiPermsTag {
        const VALID_BITS_MASK: u8 = 0b111;
        const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
        const COMBINED_NAMES: &[(u8, &str)] = &[];
        const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::AsciiCaseInsensitive;
    }

    for valid in ["read", "  read  ", "READ", "Read", "  ReAd  "] {
        assert_eq!(
            CiPerms::try_from_str(valid, "|", NameMatcher::AsciiCaseInsensitive)
                .unwrap()
                .bits(),
            0b1
        );
    }

    for invalid in ["red", "readed", "readable", "r", "d"] {
        assert!(CiPerms::try_from_str(invalid, "|", NameMatcher::AsciiCaseInsensitive).is_err());
    }
}

#[test]
fn test_flags_collect() {
    assert_eq!(
        [Perms::from(READ), READ.into(), EXEC.into()]
            .into_iter()
            .collect::<Perms>(),
        READ | EXEC
    );

    assert_eq!(
        [READ, READ, EXEC].into_iter().collect::<Perms>(),
        READ | EXEC
    );

    assert_eq!(
        [
            Perms::from(READ),
            NONE,
            WRITE.into(),
            NONE,
            Perms::from_bits_retain(0b1000)
        ]
        .into_iter()
        .collect::<Perms>(),
        READ | WRITE | Perms::from_bits_retain(0b1000)
    );
}

#[test]
fn test_flags_skipped_flag_names() {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
    struct FlagsTag;

    type Flags = BitFlags<u8, FlagsTag>;

    impl BitFlagsSpec<u8> for FlagsTag {
        const VALID_BITS_MASK: u8 = 0b1111_1111;
        const FLAG_NAMES: &[&[&str]] = &[
            &[],
            &["second"],
            &[],
            &["fourth"],
            &["fifth"],
            &[],
            &["seventh", "SEVENTH"],
            &[],
        ];
        const COMBINED_NAMES: &[(u8, &str)] = &[];
        const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
    }

    assert_eq!(
        Flags::from_bits_retain(0b1111_1111).to_string(),
        "second | fourth | fifth | seventh | 0xa5"
    );

    assert_eq!(Flags::from_str("0x0").unwrap().bits(), 0x0);
    assert_eq!(Flags::from_str("0x1").unwrap().bits(), 0x1);
    assert_eq!(Flags::from_str("0x2").unwrap().bits(), 0x2);
    assert_eq!(Flags::from_str("0x3").unwrap().bits(), 0x3);

    assert_eq!(
        Flags::from_str("second | fourth | fifth | seventh | 0xa5")
            .unwrap()
            .bits(),
        0b1111_1111
    );

    #[cfg(feature = "alloc")]
    assert_eq!(
        Flags::from_str("first").err().unwrap().to_string(),
        "unrecognized named flag or hex flag, given: \"first\", expected \"\", or a \
         \"|\"-separated set of \"second\", \"fourth\", \"fifth\", \"seventh\", or a valid hex \
         flag with \"0x\" prefix"
    );

    #[cfg(not(feature = "alloc"))]
    assert_eq!(
        Flags::from_str("first").err().unwrap().to_string(),
        "unrecognized named flag or hex flag, expected \"\", or a \"|\"-separated set of \
         \"second\", \"fourth\", \"fifth\", \"seventh\", or a valid hex flag with \"0x\" prefix"
    );
}

#[test]
fn test_flags_flag_names_beyond_bits() {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
    struct FlagsTag;

    type Flags = BitFlags<u8, FlagsTag>;

    impl BitFlagsSpec<u8> for FlagsTag {
        const VALID_BITS_MASK: u8 = 0b1111_1111;
        const FLAG_NAMES: &[&[&str]] = &[
            &["first"],
            &["second"],
            &["third"],
            &["fourth"],
            &["fifth"],
            &["sixth"],
            &["seventh"],
            &["eighth"],
            &["ninth"],
            &["tenth"],
        ];
        const COMBINED_NAMES: &[(u8, &str)] = &[];
        const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
    }

    assert_eq!(Flags::from_str("eighth").unwrap().bits(), 0b1000_0000);
    assert!(Flags::from_str("ninth").is_err());
    assert!(Flags::from_str("tenth").is_err());

    assert_eq!(
        Flags::from_bits_retain(0b1111_1111).to_string(),
        "first | second | third | fourth | fifth | sixth | seventh | eighth"
    );

    assert!(
        Flags::from_str(
            "first | second | third | fourth | fifth | sixth | seventh | eighth | ninth"
        )
        .is_err()
    );

    assert_eq!(
        Flags::from_str("first | second | third | fourth | fifth | sixth | seventh | eighth")
            .unwrap()
            .bits(),
        0b1111_1111
    );
    assert!(
        Flags::from_str(
            "first | second | third | fourth | fifth | sixth | seventh | eighth | ninth"
        )
        .is_err()
    );
}

#[test]
fn test_flags_failing_partial_fmt() {
    struct SmallWriter(usize);

    impl Write for SmallWriter {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let result = if self.0 < value.len() {
                Err(fmt::Error)
            } else {
                Ok(())
            };
            self.0 = self.0.saturating_sub(value.len());
            result
        }
    }

    const FMT_LEN: usize = 171;

    #[cfg(feature = "alloc")]
    const NAMED_ERR_FMT_LEN: usize = 156;
    #[cfg(not(feature = "alloc"))]
    const NAMED_ERR_FMT_LEN: usize = 137;

    #[cfg(feature = "alloc")]
    const UNNAMED_ERR_FMT_LEN: usize = 84;
    #[cfg(not(feature = "alloc"))]
    const UNNAMED_ERR_FMT_LEN: usize = 65;

    let flags = READ | WRITE | EXEC | Perms::from_bits_retain(0b1000);
    for i in 0..FMT_LEN {
        assert!(
            write!(
                &mut SmallWriter(i),
                "{flags} {flags:#} {flags:b} {flags:#b} {flags:o} {flags:#o} {flags:x} {flags:#x} \
                 {flags:X} {flags:#X} {flags:?} {flags:#?}"
            )
            .is_err(),
            "{i}"
        );
    }
    assert!(
        write!(
            &mut SmallWriter(FMT_LEN),
            "{flags} {flags:#} {flags:b} {flags:#b} {flags:o} {flags:#o} {flags:x} {flags:#x} \
             {flags:X} {flags:#X} {flags:?} {flags:#?}"
        )
        .is_ok()
    );

    let error = Perms::from_str("readable").err().unwrap();
    for i in 0..NAMED_ERR_FMT_LEN {
        assert!(write!(&mut SmallWriter(i), "{error}").is_err(), "{i}");
    }
    assert!(write!(&mut SmallWriter(NAMED_ERR_FMT_LEN), "{error}").is_ok());

    let error = UnnamedPerms::from_str("readable").err().unwrap();
    for i in 0..UNNAMED_ERR_FMT_LEN {
        assert!(write!(&mut SmallWriter(i), "{error}").is_err(), "{i}");
    }
    assert!(write!(&mut SmallWriter(UNNAMED_ERR_FMT_LEN), "{error}").is_ok());
}

#[cfg(feature = "arbitrary")]
#[test]
fn test_flags_arbitrary() {
    use arbitrary::{Arbitrary, Unstructured};

    let data = &[0b001, 0b010, 0b101, 0b1010];
    let mut u = Unstructured::new(data);
    assert_eq!(<[Perms; 4]>::size_hint(1), (4, Some(4)));
    assert_eq!(
        <[Perms; 4]>::arbitrary(&mut u).unwrap(),
        [
            READ.into(),
            WRITE.into(),
            READ | EXEC,
            WRITE | Perms::from_bits_retain(0b1000),
        ]
    );
}

#[cfg(feature = "bytemuck")]
#[test]
fn test_flags_bytemuck() {
    use bytemuck::{Pod, Zeroable};

    fn assert_pod<T: Pod>() {}
    fn assert_zeroable<T: Zeroable>() {}

    assert_pod::<Perms>();
    assert_zeroable::<Perms>();

    let f: Perms = Zeroable::zeroed();
    assert!(f.is_empty());
    assert_eq!(f, NONE);

    let bits = 0b0101u8;
    let f: Perms = bytemuck::cast(bits);
    assert_eq!(f.bits(), bits);

    let raw: u8 = bytemuck::cast(f);
    assert_eq!(raw, bits);

    let flags = [
        READ.into(),
        WRITE.into(),
        READ | EXEC,
        WRITE | Perms::from_bits_retain(0b1000),
    ];
    let bytes: &[u8] = bytemuck::cast_slice(&flags);
    assert_eq!(bytes, &[0b001, 0b010, 0b101, 0b1010]);
}

#[cfg(feature = "serde")]
#[test]
fn test_flags_serde_unnamed() {
    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error, assert_tokens};

    #[cfg(not(feature = "alloc"))]
    const ERROR_CORE: &str = "unrecognized hex flag, expected a valid hex flag with \"0x\" prefix";

    assert_tokens(&(U_READ | U_EXEC), &[Token::Str("0x5")]);
    assert_de_tokens(&(U_READ | U_EXEC), &[Token::Str("  0x1  |  0x4  ")]);
    assert_tokens(
        &(U_READ | U_WRITE | UnnamedPerms::from_bits_truncate(0b1000)),
        &[Token::Str("0x3")],
    );
    assert_tokens(&UnnamedPerms::empty(), &[Token::Str("")]);

    assert_de_tokens(&(U_READ | U_EXEC), &[Token::Str("0x0005")]);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<UnnamedPerms>(
        &[Token::Str("5")],
        "unrecognized hex flag, given: \"5\", expected a valid hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<UnnamedPerms>(&[Token::Str("5")], ERROR_CORE);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<UnnamedPerms>(
        &[Token::Str("red")],
        "unrecognized hex flag, given: \"red\", expected a valid hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<UnnamedPerms>(&[Token::Str("red")], ERROR_CORE);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<UnnamedPerms>(
        &[Token::Str("0xCOFFEE")],
        "unrecognized hex flag, given: \"0xCOFFEE\", expected a valid hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<UnnamedPerms>(&[Token::Str("0xCOFFEE")], ERROR_CORE);

    assert_de_tokens_error::<UnnamedPerms>(
        &[Token::I32(123)],
        "invalid type: integer `123`, expected a valid hex flag with \"0x\" prefix",
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_flags_serde_named() {
    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error, assert_tokens};

    #[cfg(not(feature = "alloc"))]
    const ERROR_CORE: &str = "unrecognized named flag or hex flag, expected \"\", or a \
                              \"|\"-separated set of \"read\", \"write\", \"exec\", or a valid \
                              hex flag with \"0x\" prefix";

    assert_tokens(&(READ | EXEC), &[Token::Str("read | exec")]);
    assert_de_tokens(&(READ | EXEC), &[Token::Str("  read  |execute          ")]);
    assert_tokens(
        &(READ | WRITE | Perms::from_bits_truncate(0b1000)),
        &[Token::Str("read | write")],
    );
    assert_tokens(&NONE, &[Token::Str("")]);

    assert_de_tokens(&(READ | EXEC), &[Token::Str("0x05")]);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<Perms>(
        &[Token::Str("5")],
        "unrecognized named flag or hex flag, given: \"5\", expected \"\", or a \"|\"-separated \
         set of \"read\", \"write\", \"exec\", or a valid hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<Perms>(&[Token::Str("5")], ERROR_CORE);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<Perms>(
        &[Token::Str("red")],
        "unrecognized named flag or hex flag, given: \"red\", expected \"\", or a \"|\"-separated \
         set of \"read\", \"write\", \"exec\", or a valid hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<Perms>(&[Token::Str("red")], ERROR_CORE);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<Perms>(
        &[Token::Str("0xCOFFEE")],
        "unrecognized named flag or hex flag, given: \"0xCOFFEE\", expected \"\", or a \
         \"|\"-separated set of \"read\", \"write\", \"exec\", or a valid hex flag with \"0x\" \
         prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<Perms>(&[Token::Str("0xCOFFEE")], ERROR_CORE);

    assert_de_tokens_error::<Perms>(
        &[Token::I32(123)],
        "invalid type: integer `123`, expected \"\", or a \"|\"-separated set of \"read\", \
         \"write\", \"exec\", or a valid hex flag with \"0x\" prefix",
    );
}
