//! Simple example-like tests.

#![allow(unused_crate_dependencies, reason = "okay in tests")]

use bitflags_lite::{BitFlags, BitFlagsSpec};

/// A tag for permissions bit flags allowing only valid bit combinations.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct PermsTag;

/// Permissions bit flags allowing only valid bit combinations.
pub type Perms = BitFlags<u32, PermsTag>;

const NONE: Perms = Perms::empty();
const READ: Perms = Perms::from_bits(1 << 0).unwrap();
const WRITE: Perms = Perms::from_bits(1 << 1).unwrap();
const EXEC: Perms = Perms::from_bits(1 << 2).unwrap();

impl BitFlagsSpec<u32> for PermsTag {
    const VALID_BITS: u32 = (1 << 3) - 1;
    type AnyBitsAllowed = bitflags_lite::False;
}

/// A tag for permissions bit flags allowing any bit combinations.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct PermsAnyTag;

/// Permissions bit flags allowing any bit combinations.
pub type PermsAny = BitFlags<u32, PermsAnyTag>;

const NONE_ANY: PermsAny = PermsAny::empty();
const READ_ANY: PermsAny = PermsAny::from_bits(1 << 0).unwrap();
const WRITE_ANY: PermsAny = PermsAny::from_bits(1 << 1).unwrap();
const EXEC_ANY: PermsAny = PermsAny::from_bits(1 << 2).unwrap();

impl BitFlagsSpec<u32> for PermsAnyTag {
    const VALID_BITS: u32 = (1 << 3) - 1;
    type AnyBitsAllowed = bitflags_lite::True;
}

#[test]
fn test_empty() {
    let flags = Perms::empty();
    assert!(flags.is_empty());
    assert_eq!(flags.bits(), 0);
    assert_eq!(flags.into_iter().len(), 0);
}

#[test]
fn test_all() {
    let flags = Perms::all();
    assert!(!flags.is_empty());
    assert_eq!(flags.bits(), 0b111);
    assert_eq!(flags.into_iter().len(), 3);
}

#[test]
fn test_all_valid() {
    let flags = Perms::all_valid();
    assert!(!flags.is_empty());
    assert_eq!(flags.bits(), 0b111);
    assert_eq!(flags.into_iter().len(), 3);

    let flags = PermsAny::all_valid();
    assert!(!flags.is_empty());
    assert_eq!(flags.bits(), 0b111);
    assert_eq!(flags.into_iter().len(), 3);
}

#[test]
fn test_from_bits() {
    assert_eq!(Perms::from_bits(0b1), Some(READ));
    assert_eq!(Perms::from_bits(0b10), Some(WRITE));
    assert_eq!(Perms::from_bits(0b11), Some(READ | WRITE));
    assert_eq!(Perms::from_bits(0b100), Some(EXEC));
    assert_eq!(Perms::from_bits(0b110), Some(WRITE | EXEC));
    assert_eq!(Perms::from_bits(0b111), Some(READ | WRITE | EXEC));
    assert_eq!(Perms::from_bits(0b1000), None);
    assert_eq!(Perms::from_bits(0b1100), None);
    assert_eq!(Perms::from_bits(0b1110), None);
    assert_eq!(Perms::from_bits(0b1111), None);
}

#[test]
fn test_from_truncate() {
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
}

#[test]
fn test_from_retain() {
    assert_eq!(PermsAny::from_bits(1 << 3), None);
    let bit4 = PermsAny::from_bits_retain(1 << 3);
    assert_eq!(bit4.bits(), 0b1000);

    assert_eq!(PermsAny::from_bits_retain(0b1), READ_ANY);
    assert_eq!(PermsAny::from_bits_retain(0b10), WRITE_ANY);
    assert_eq!(PermsAny::from_bits_retain(0b11), READ_ANY | WRITE_ANY);
    assert_eq!(PermsAny::from_bits_retain(0b100), EXEC_ANY);
    assert_eq!(PermsAny::from_bits_retain(0b110), WRITE_ANY | EXEC_ANY);
    assert_eq!(
        PermsAny::from_bits_retain(0b111),
        READ_ANY | WRITE_ANY | EXEC_ANY
    );
    assert_eq!(PermsAny::from_bits_retain(0b1000), NONE_ANY | bit4);
    assert_eq!(PermsAny::from_bits_retain(0b1100), EXEC_ANY | bit4);
    assert_eq!(
        PermsAny::from_bits_retain(0b1110),
        WRITE_ANY | EXEC_ANY | bit4
    );
    assert_eq!(
        PermsAny::from_bits_retain(0b1111),
        READ_ANY | WRITE_ANY | EXEC_ANY | bit4
    );
}

#[test]
fn test_truncate() {
    let flags = PermsAny::from_bits_retain(0b1111_1111);
    assert_eq!(
        flags,
        PermsAny::from_bits_retain(0b1111_1000) | READ_ANY | WRITE_ANY | EXEC_ANY
    );
    assert_eq!(flags.truncate(), READ_ANY | WRITE_ANY | EXEC_ANY);

    let flags = PermsAny::from_bits_retain(0b1111_1101);
    assert_eq!(
        flags,
        PermsAny::from_bits_retain(0b1111_1000) | READ_ANY | EXEC_ANY
    );
    assert_eq!(flags.truncate(), READ_ANY | EXEC_ANY);
}

#[test]
fn test_intersects() {
    assert!(READ.intersects(READ));
    assert!(!READ.intersects(WRITE));
    assert!((READ | WRITE).intersects(READ | WRITE));
    assert!((READ | WRITE).intersects(WRITE | EXEC));
    assert!(!READ.intersects(WRITE | EXEC));
}

#[test]
fn test_contains() {
    assert!(READ.contains(READ));
    assert!(!READ.contains(WRITE));
    assert!((READ | WRITE).contains(READ | WRITE));
    assert!(!(READ | WRITE).contains(WRITE | EXEC));
    assert!(!READ.contains(WRITE | EXEC));
    assert!((READ | WRITE | EXEC).contains(WRITE | EXEC));
}

#[test]
fn test_insert_remove_toggle_set() {
    let mut perms = NONE;

    perms.insert(READ);
    assert_eq!(perms, READ);
    perms.insert(READ);
    assert_eq!(perms, READ);
    perms.remove(READ);
    assert_eq!(perms, NONE);
    perms.remove(READ);
    assert_eq!(perms, NONE);
    perms.toggle(READ);
    assert_eq!(perms, READ);

    perms.insert(WRITE);
    assert_eq!(perms, READ | WRITE);
    perms.remove(WRITE);
    assert_eq!(perms, READ);
    perms.toggle(WRITE);
    assert_eq!(perms, READ | WRITE);
    perms.toggle(READ);
    assert_eq!(perms, WRITE);

    perms.set(EXEC, false);
    assert_eq!(perms, WRITE);
    perms.set(EXEC, true);
    assert_eq!(perms, WRITE | EXEC);

    perms.toggle(READ | WRITE | EXEC);
    assert_eq!(perms, READ);
}

#[test]
fn test_intersection() {
    assert_eq!(NONE.intersection(READ), NONE);
    assert_eq!(READ.intersection(READ), READ);
    assert_eq!(READ.intersection(WRITE), NONE);
    assert_eq!((READ | WRITE).intersection(READ | WRITE), READ | WRITE);
    assert_eq!((READ | WRITE).intersection(WRITE | EXEC), WRITE);
    assert_eq!(READ.intersection(WRITE | EXEC), NONE);
}

#[test]
fn test_union() {
    assert_eq!(NONE.union(READ), READ);
    assert_eq!(READ.union(READ), READ);
    assert_eq!(READ.union(WRITE), READ | WRITE);
    assert_eq!((READ | WRITE).union(READ | WRITE), READ | WRITE);
    assert_eq!((READ | WRITE).union(WRITE | EXEC), READ | WRITE | EXEC);
    assert_eq!(READ.union(WRITE | EXEC), READ | WRITE | EXEC);
}

#[test]
fn test_difference() {
    assert_eq!(NONE.difference(READ), NONE);
    assert_eq!(READ.difference(READ), NONE);
    assert_eq!(READ.difference(WRITE), READ);
    assert_eq!((READ | WRITE).difference(READ | WRITE), NONE);
    assert_eq!((READ | WRITE).difference(WRITE | EXEC), READ);
    assert_eq!(READ.difference(WRITE | EXEC), READ);
}

#[test]
fn test_symmetric_difference() {
    assert_eq!(NONE.symmetric_difference(READ), READ);
    assert_eq!(READ.symmetric_difference(READ), NONE);
    assert_eq!(READ.symmetric_difference(WRITE), READ | WRITE);
    assert_eq!((READ | WRITE).symmetric_difference(READ | WRITE), NONE);
    assert_eq!(
        (READ | WRITE).symmetric_difference(WRITE | EXEC),
        READ | EXEC
    );
    assert_eq!(READ.symmetric_difference(WRITE | EXEC), READ | WRITE | EXEC);
}

#[test]
fn test_complement() {
    assert_eq!(NONE.complement(), READ | WRITE | EXEC);
    assert_eq!(READ.complement(), WRITE | EXEC);
    assert_eq!(WRITE.complement(), READ | EXEC);
    assert_eq!(EXEC.complement(), READ | WRITE);
    assert_eq!((READ | WRITE).complement(), EXEC);
    assert_eq!((WRITE | EXEC).complement(), READ);
    assert_eq!((READ | EXEC).complement(), WRITE);
    assert_eq!((READ | WRITE | EXEC).complement(), NONE);
}

#[test]
fn test_iterator() {
    let rwx = READ | WRITE | EXEC;
    assert_eq!(rwx.into_iter().len(), 3);
    assert_eq!(rwx.into_iter().size_hint(), (3, Some(3)));
    assert_eq!(rwx.into_iter().collect::<Vec<_>>(), vec![READ, WRITE, EXEC]);
    assert_eq!(
        rwx.into_iter().rev().collect::<Vec<_>>(),
        vec![EXEC, WRITE, READ]
    );

    let rw = READ | WRITE;
    assert_eq!(rw.into_iter().len(), 2);
    assert_eq!(rw.into_iter().size_hint(), (2, Some(2)));
    assert_eq!(rw.into_iter().collect::<Vec<_>>(), vec![READ, WRITE]);
    assert_eq!(rw.into_iter().rev().collect::<Vec<_>>(), vec![WRITE, READ]);

    let none = NONE;
    assert_eq!(none.into_iter().len(), 0);
    assert_eq!(none.into_iter().size_hint(), (0, Some(0)));
    assert_eq!(none.into_iter().collect::<Vec<_>>(), vec![]);
    assert_eq!(none.into_iter().rev().collect::<Vec<_>>(), vec![]);
}
