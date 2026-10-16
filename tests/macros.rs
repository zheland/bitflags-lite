//! Tests for `macros.rs` file.

#![allow(unused_crate_dependencies, reason = "okay in tests")]

use bitflags_lite::{BitFlagsSpec, NameMatcher};

#[test]
fn test_macro_generated_simple() {
    bitflags_lite::bitflags! {
        pub type Permissions = BitFlags<u32, pub struct PermissionsTag>;

        pub bit READ  = 0, &["read"];
        pub bit WRITE = 1, &["write"];
        pub bit EXEC  = 2, &["exec", "execute"];

        pub mask RWX  = mask READ | WRITE | EXEC, "all";
        pub mask NONE = 0, "none";
    }

    assert_eq!(
        <PermissionsTag as BitFlagsSpec<u32>>::VALID_BITS_MASK,
        0b111
    );
    assert_eq!(
        <PermissionsTag as BitFlagsSpec<u32>>::FLAG_NAMES,
        &[&["read"][..], &["write"], &["exec", "execute"]]
    );
    assert_eq!(
        <PermissionsTag as BitFlagsSpec<u32>>::COMBINED_NAMES,
        &[(0b111, "all"), (0, "none")]
    );
    assert_eq!(
        <PermissionsTag as BitFlagsSpec<u32>>::DEFAULT_NAME_MATCHER,
        NameMatcher::Exact
    );

    assert_eq!(READ.bit(), 0);
    assert_eq!(WRITE.bit(), 1);
    assert_eq!(EXEC.bit(), 2);
    assert_eq!(RWX.bits(), 0b111);
    assert_eq!(NONE.bits(), 0);
}

#[test]
fn test_macro_generated_with_skips() {
    bitflags_lite::bitflags! {
        #[name_matcher(NameMatcher::AsciiCaseInsensitive)]
        pub type Flags = BitFlags<u32, pub struct FlagsTag>;

        pub bit FIRST = 0, &["first", "1st"];
        pub bit THIRD = 2, &["third", "3rd"];
        pub bit       = 3;
        pub bit TENTH = 9, &["tenth", "10th"];

        pub mask ALL  = mask FIRST | THIRD | TENTH, "all";
        pub mask NONE = 0, "none";
    }

    assert_eq!(
        <FlagsTag as BitFlagsSpec<u32>>::VALID_BITS_MASK,
        0b10_0000_1101
    );
    assert_eq!(
        <FlagsTag as BitFlagsSpec<u32>>::FLAG_NAMES,
        &[
            &["first", "1st"][..],
            &[],
            &["third", "3rd"],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &["tenth", "10th"]
        ]
    );
    assert_eq!(
        <FlagsTag as BitFlagsSpec<u32>>::COMBINED_NAMES,
        &[(0b10_0000_0101, "all"), (0, "none")]
    );
    assert_eq!(
        <FlagsTag as BitFlagsSpec<u32>>::DEFAULT_NAME_MATCHER,
        NameMatcher::AsciiCaseInsensitive
    );

    assert_eq!(FIRST.bit(), 0);
    assert_eq!(THIRD.bit(), 2);
    assert_eq!(TENTH.bit(), 9);
    assert_eq!(ALL.bits(), 0b10_0000_0101);
    assert_eq!(NONE.bits(), 0);
}
