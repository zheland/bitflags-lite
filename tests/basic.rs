//! Simple example-like tests.

#![allow(unused_crate_dependencies, reason = "okay in tests")]

use bitflags_lite::{BitFlags, BitFlagsAny, BitFlagsSet};

/// A tag for permissions bit flags allowing only valid bit combinations.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct PermsTag;

/// Permissions bit flags allowing only valid bit combinations.
pub type Perms = BitFlags<u32, PermsTag>;

const NONE: Perms = Perms::empty();
const READ: Perms = Perms::from_bits(1 << 0).unwrap();
const WRITE: Perms = Perms::from_bits(1 << 1).unwrap();
const EXEC: Perms = Perms::from_bits(1 << 2).unwrap();

impl BitFlagsSet<u32> for PermsTag {
    const EMPTY_BITS: u32 = 0;
    const ALL_BITS: u32 = (1 << 3) - 1;
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

impl BitFlagsSet<u32> for PermsAnyTag {
    const EMPTY_BITS: u32 = 0;
    const ALL_BITS: u32 = (1 << 3) - 1;
}

impl BitFlagsAny for PermsAnyTag {}

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
    assert_eq!(PermsAny::from_bits_retain(0b1), READ_ANY);
    assert_eq!(PermsAny::from_bits_retain(0b10), WRITE_ANY);
    assert_eq!(PermsAny::from_bits_retain(0b11), READ_ANY | WRITE_ANY);
    assert_eq!(PermsAny::from_bits_retain(0b100), EXEC_ANY);
    assert_eq!(PermsAny::from_bits_retain(0b110), WRITE_ANY | EXEC_ANY);
    assert_eq!(
        PermsAny::from_bits_retain(0b111),
        READ_ANY | WRITE_ANY | EXEC_ANY
    );

    assert_eq!(PermsAny::from_bits(1 << 3), None);
    let bit4 = PermsAny::from_bits_retain(1 << 3);
    assert_eq!(bit4.bits(), 0b1000);

    assert_eq!(PermsAny::from_bits_truncate(0b1000), NONE_ANY);
    assert_eq!(PermsAny::from_bits_truncate(0b1100), EXEC_ANY);
    assert_eq!(PermsAny::from_bits_truncate(0b1110), WRITE_ANY | EXEC_ANY);
    assert_eq!(
        PermsAny::from_bits_truncate(0b1111),
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
