//! Simple example-like tests.

#![allow(unused_crate_dependencies, reason = "okay in tests")]

use bitflags_lite::{BitFlags, BitFlagsIter, BitFlagsSpec};

/// A tag for permissions bit flags allowing only valid bit combinations.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct PermsTag;

/// Permissions bit flags allowing only valid bit combinations.
pub type Perms = BitFlags<u8, PermsTag>;

const NONE: Perms = Perms::empty();
const READ: Perms = Perms::from_bits(1 << 0).unwrap();
const WRITE: Perms = Perms::from_bits(1 << 1).unwrap();
const EXEC: Perms = Perms::from_bits(1 << 2).unwrap();

impl BitFlagsSpec<u8> for PermsTag {
    const VALID_BITS: u8 = (1 << 3) - 1;
    const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
    const ASCII_CASE_INSENSITIVE_PARSING: bool = false;
}

#[test]
fn test_empty() {
    let flags = Perms::empty();
    assert!(flags.is_empty());
    assert_eq!(flags.bits(), 0);
    assert_eq!(flags.into_iter().len(), 0);
}

#[test]
fn test_all_valid() {
    let flags = Perms::all_valid();
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
    assert_eq!(Perms::from_bits(1 << 3), None);
    let bit4 = Perms::from_bits_retain(1 << 3);
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
}

#[test]
fn test_truncate() {
    let flags = Perms::from_bits_retain(0b1111_1111);
    assert_eq!(
        flags,
        Perms::from_bits_retain(0b1111_1000) | READ | WRITE | EXEC
    );
    assert_eq!(flags.truncate(), READ | WRITE | EXEC);

    let flags = Perms::from_bits_retain(0b1111_1101);
    assert_eq!(flags, Perms::from_bits_retain(0b1111_1000) | READ | EXEC);
    assert_eq!(flags.truncate(), READ | EXEC);
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
fn test_from() {
    let rwx = READ | WRITE | EXEC;
    assert_eq!(u8::from(rwx), 0b111);

    let rx = READ | EXEC;
    assert_eq!(u8::from(rx), 0b101);

    let flags = Perms::from_bits_retain(0b1111_1111);
    assert_eq!(u8::from(flags), 0b1111_1111);
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

#[test]
fn test_clone() {
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
fn test_cmp() {
    assert_eq!((READ | WRITE) - WRITE, READ);
    assert!(READ < WRITE);
    assert!(WRITE < EXEC);

    let mut set = [
        Perms::from_bits_retain(0b1000),
        Perms::from_bits_retain(0b1010),
        READ,
        WRITE,
        EXEC,
        READ | WRITE,
        WRITE | EXEC,
        EXEC | READ,
        READ,
        WRITE,
        EXEC,
        READ | WRITE | EXEC,
        NONE,
    ];
    set.sort_unstable();
    assert_eq!(
        set,
        [
            NONE,
            READ,
            READ,
            WRITE,
            WRITE,
            READ | WRITE,
            EXEC,
            EXEC,
            READ | EXEC,
            WRITE | EXEC,
            READ | WRITE | EXEC,
            Perms::from_bits_retain(0b1000),
            Perms::from_bits_retain(0b1010),
        ]
    );
}

#[test]
fn test_hash() {
    use core::hash::{Hash, Hasher};
    use std::collections::HashSet;
    use std::hash::DefaultHasher;

    fn hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    assert_eq!(hash(&READ), hash(&READ));
    assert_ne!(hash(&READ), hash(&WRITE));
    assert_ne!(hash(&READ), hash(&Perms::from_bits_retain(0b1000)));

    let input = [
        READ,
        WRITE | EXEC,
        READ,
        READ | WRITE | EXEC,
        Perms::from_bits_retain(0b1000),
        Perms::from_bits_retain(0b1010),
        READ | WRITE | EXEC,
    ];

    let set: HashSet<_> = input.into_iter().collect();

    assert_eq!(set.len(), 5);
    assert!(set.contains(&READ));
    assert!(set.contains(&(WRITE | EXEC)));
    assert!(set.contains(&(READ | WRITE | EXEC)));
    assert!(set.contains(&Perms::from_bits_retain(0b1000)));
    assert!(set.contains(&Perms::from_bits_retain(0b1010)));
    assert!(!set.contains(&NONE));
    assert!(!set.contains(&WRITE));
    assert!(!set.contains(&(READ | WRITE)));
    assert!(!set.contains(&(READ | EXEC)));
}

#[test]
fn test_default() {
    let flags = Perms::default();
    assert!(flags.is_empty());
    assert_eq!(flags.bits(), 0);

    let mut iter = BitFlagsIter::<u8, PermsTag>::default();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_as_ref() {
    use core::borrow::Borrow;

    let rwx = READ | WRITE | EXEC;
    assert_eq!(rwx.as_ref(), &0b111);
    assert_eq!(Borrow::<u8>::borrow(&rwx), &0b111);
}

#[test]
fn test_ops() {
    assert_eq!(!NONE, READ | WRITE | EXEC);
    assert_eq!(!!NONE, NONE);
    assert_eq!(!READ, WRITE | EXEC);
    assert_eq!(!READ & !WRITE, EXEC);
    assert_eq!(READ ^ WRITE, READ | WRITE);
    assert_eq!((READ | WRITE) & (WRITE | EXEC), WRITE);
    assert_eq!((READ | WRITE) ^ (WRITE | EXEC), READ | EXEC);
    assert_eq!((READ | WRITE) - (WRITE | EXEC), READ);
}

#[test]
fn test_assign_ops() {
    let mut rwx = READ | WRITE | EXEC;
    rwx |= READ;
    assert_eq!(rwx, READ | WRITE | EXEC);
    rwx &= !WRITE;
    assert_eq!(rwx, READ | EXEC);
    rwx ^= WRITE;
    assert_eq!(rwx, READ | WRITE | EXEC);
    rwx ^= READ;
    assert_eq!(rwx, WRITE | EXEC);
    rwx &= READ | EXEC;
    assert_eq!(rwx, EXEC);
    rwx |= WRITE;
    assert_eq!(rwx, WRITE | EXEC);
    rwx -= READ | WRITE;
    assert_eq!(rwx, EXEC);
}

#[test]
fn test_fmt() {
    let rwx = READ | WRITE | EXEC | Perms::from_bits_retain(0b1000);
    assert_eq!(format!("{rwx}"), "1111");
    assert_eq!(format!("{rwx:#}"), "0b1111");

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
        "BitFlags(0b1111, PhantomData<basic::PermsTag>)"
    );
    assert_eq!(
        format!("{rwx:#?}"),
        "BitFlags(0b1111, PhantomData<basic::PermsTag>)"
    );
    assert_eq!(
        format!("{:?}", rwx.into_iter()),
        "BitFlagsIter(BitFlags(0b1111, PhantomData<basic::PermsTag>))"
    );
}

#[test]
fn test_collect() {
    assert_eq!(
        [READ, READ, EXEC].into_iter().collect::<Perms>(),
        READ | EXEC
    );

    assert_eq!(
        [READ, NONE, WRITE, NONE, Perms::from_bits_retain(0b1000)]
            .into_iter()
            .collect::<Perms>(),
        READ | WRITE | Perms::from_bits_retain(0b1000)
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_serde() {
    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error, assert_tokens};

    assert_tokens(&(READ | EXEC), &[Token::Str("101")]);
    assert_tokens(
        &(READ | WRITE | Perms::from_bits_truncate(0b1000)),
        &[Token::Str("1011")],
    );
    assert_tokens(&NONE, &[Token::Str("0")]);

    assert_de_tokens(&(READ | EXEC), &[Token::Str("0b01")]);
    assert_de_tokens(&(READ | EXEC), &[Token::Str("0B01")]);
    assert_de_tokens_error::<Perms>(&[Token::Str("0b0B01")], "");
}

#[cfg(feature = "bytemuck")]
#[test]
fn test_bytemuck() {
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
        READ,
        WRITE,
        READ | EXEC,
        WRITE | Perms::from_bits_truncate(0b1000),
    ];
    let bytes: &[u8] = bytemuck::cast_slice(&flags);
    assert_eq!(bytes, &[0b001, 0b010, 0b101, 0b1010]);
}

#[cfg(feature = "arbitrary")]
#[test]
fn test_arbitrary() {
    use arbitrary::{Arbitrary, Unstructured};

    let data = &[0b001, 0b010, 0b101, 0b1010];
    let mut u = Unstructured::new(data);
    let perms = <[Perms; 4]>::arbitrary(&mut u).expect("should not fail");
    assert_eq!(
        perms,
        [
            READ,
            WRITE,
            READ | EXEC,
            WRITE | Perms::from_bits_truncate(0b1000),
        ]
    );
}
