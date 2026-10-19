//! Tests for `flag.rs` file.

#![expect(clippy::unwrap_used, reason = "okay in tests")]
#![allow(unused_crate_dependencies, reason = "okay in tests")]

use core::cmp::Ordering;
use core::fmt::{self, Write};
use core::hash::Hash;
use core::str::FromStr;

use bitflags_lite::{BitFlag, BitFlagFromFlagsError, BitFlags, BitFlagsSpec, NameMatcher};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct PermsTag;
type Perm = BitFlag<u8, PermsTag>;
type Perms = BitFlags<u8, PermsTag>;

impl BitFlagsSpec<u8> for PermsTag {
    const VALID_BITS_MASK: u8 = 0b111;
    const DEFAULT_BITS: u8 = 0b000;
    const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
    const COMBINED_NAMES: &[(u8, &str)] = &[];
    const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
}

const NONE: Perms = Perms::empty();
const READ: Perm = Perm::try_from_bit(0).unwrap();
const WRITE: Perm = Perm::try_from_bit(1).unwrap();
const EXEC: Perm = Perm::try_from_bit(2).unwrap();

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct UnnamedPermsTag;
type UnnamedPerm = BitFlag<u8, UnnamedPermsTag>;

impl BitFlagsSpec<u8> for UnnamedPermsTag {
    const VALID_BITS_MASK: u8 = 0b111;
    const DEFAULT_BITS: u8 = 0b000;
    const FLAG_NAMES: &[&[&str]] = &[];
    const COMBINED_NAMES: &[(u8, &str)] = &[];
    const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
}

const U_READ: UnnamedPerm = UnnamedPerm::try_from_bit(0).unwrap();
const U_WRITE: UnnamedPerm = UnnamedPerm::try_from_bit(1).unwrap();
const U_EXEC: UnnamedPerm = UnnamedPerm::try_from_bit(2).unwrap();

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
    assert_eq!(Perm::try_from_bits(0), None);
    assert_eq!(Perm::try_from_bits(1), Some(READ));
    assert_eq!(Perm::try_from_bits(2), Some(WRITE));
    assert_eq!(Perm::try_from_bits(3), None);
    assert_eq!(Perm::try_from_bits(4), Some(EXEC));
    assert_eq!(Perm::try_from_bits(5), None);
    assert_eq!(Perm::try_from_bits(6), None);
    assert_eq!(Perm::try_from_bits(7), None);
    assert_eq!(Perm::try_from_bits(8), None);

    assert_eq!(Perm::try_from_bits_retain(0), None);
    assert_eq!(Perm::try_from_bits_retain(1), Some(READ));
    assert_eq!(Perm::try_from_bits_retain(2), Some(WRITE));
    assert_eq!(Perm::try_from_bits_retain(3), None);
    assert_eq!(Perm::try_from_bits_retain(4), Some(EXEC));
    assert_eq!(Perm::try_from_bits_retain(5), None);
    assert_eq!(Perm::try_from_bits_retain(6), None);
    assert_eq!(Perm::try_from_bits_retain(7), None);
    assert_eq!(Perm::try_from_bits_retain(8).unwrap().bit(), 3);
    assert_eq!(Perm::try_from_bits_retain(16).unwrap().bit(), 4);
    assert_eq!(Perm::try_from_bits_retain(32).unwrap().bit(), 5);

    assert_eq!(Perm::try_from_flags((READ | WRITE) - (READ | WRITE)), None);
    assert_eq!(Perm::try_from_flags(READ.into()), Some(READ));
    assert_eq!(Perm::try_from_flags(WRITE.into()), Some(WRITE));
    assert_eq!(Perm::try_from_flags(READ | WRITE), None);
    assert_eq!(Perm::try_from_flags(EXEC.into()), Some(EXEC));
    assert_eq!(Perm::try_from_flags(READ | EXEC), None);
    assert_eq!(Perm::try_from_flags(WRITE | EXEC), None);
    assert_eq!(Perm::try_from_flags(READ | WRITE | EXEC), None);
    assert_eq!(Perm::try_from_flags(Perms::from_bits_retain(0x8)), None);

    assert_eq!(
        Perm::try_from((READ | WRITE) - (READ | WRITE)),
        Err(BitFlagFromFlagsError)
    );
    assert_eq!(Perm::try_from(Perms::from(READ)), Ok(READ));
    assert_eq!(Perm::try_from(Perms::from(WRITE)), Ok(WRITE));
    assert_eq!(Perm::try_from(READ | WRITE), Err(BitFlagFromFlagsError));
    assert_eq!(Perm::try_from(Perms::from(EXEC)), Ok(EXEC));
    assert_eq!(Perm::try_from(READ | EXEC), Err(BitFlagFromFlagsError));
    assert_eq!(Perm::try_from(WRITE | EXEC), Err(BitFlagFromFlagsError));
    assert_eq!(
        Perm::try_from(READ | WRITE | EXEC),
        Err(BitFlagFromFlagsError)
    );
    assert_eq!(
        Perm::try_from(Perms::from_bits_retain(0x8)),
        Err(BitFlagFromFlagsError)
    );

    assert_eq!(
        Perm::try_from((READ | WRITE) - (READ | WRITE))
            .err()
            .unwrap()
            .to_string(),
        "flags do not contain exactly one valid bit"
    );
}

#[test]
fn test_flag_name() {
    assert_eq!(READ.name(), Some("read"));
    assert_eq!(WRITE.name(), Some("write"));
    assert_eq!(EXEC.name(), Some("exec"));
    assert_eq!(Perm::from_bit_retain(3).name(), None);

    assert_eq!(READ.name_and_aliases(), &["read"]);
    assert_eq!(WRITE.name_and_aliases(), &["write"]);
    assert_eq!(EXEC.name_and_aliases(), &["exec", "execute"]);
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
    assert_eq!(Perm::from_bit_retain(0).mask(), Some(0x01));
    assert_eq!(Perm::from_bit_retain(1).mask(), Some(0x02));
    assert_eq!(Perm::from_bit_retain(2).mask(), Some(0x04));
    assert_eq!(Perm::from_bit_retain(3).mask(), Some(0x08));
    assert_eq!(Perm::from_bit_retain(4).mask(), Some(0x10));
    assert_eq!(Perm::from_bit_retain(5).mask(), Some(0x20));
    assert_eq!(Perm::from_bit_retain(6).mask(), Some(0x40));
    assert_eq!(Perm::from_bit_retain(7).mask(), Some(0x80));
    assert_eq!(Perm::from_bit_retain(8).mask(), None);
    assert_eq!(Perm::from_bit_retain(9).mask(), None);
}

#[test]
fn test_flag_mask_of_default() {
    assert_eq!(Perm::from_bit_retain(0).mask_or_empty(), 0x01);
    assert_eq!(Perm::from_bit_retain(1).mask_or_empty(), 0x02);
    assert_eq!(Perm::from_bit_retain(2).mask_or_empty(), 0x04);
    assert_eq!(Perm::from_bit_retain(3).mask_or_empty(), 0x08);
    assert_eq!(Perm::from_bit_retain(4).mask_or_empty(), 0x10);
    assert_eq!(Perm::from_bit_retain(5).mask_or_empty(), 0x20);
    assert_eq!(Perm::from_bit_retain(6).mask_or_empty(), 0x40);
    assert_eq!(Perm::from_bit_retain(7).mask_or_empty(), 0x80);
    assert_eq!(Perm::from_bit_retain(8).mask_or_empty(), 0x00);
    assert_eq!(Perm::from_bit_retain(9).mask_or_empty(), 0x00);
}

#[test]
fn test_flag_math_ops() {
    assert_eq!(READ.intersection(READ), Perms::from(READ));
    assert_eq!(READ.intersection(WRITE), NONE);

    assert_eq!(READ.union(READ), Perms::from(READ));
    assert_eq!(READ.union(WRITE), Perms::from(READ | WRITE));

    assert_eq!(READ.difference(READ), NONE);
    assert_eq!(READ.difference(WRITE), Perms::from(READ));

    assert_eq!(READ.symmetric_difference(READ), NONE);
    assert_eq!(READ.symmetric_difference(WRITE), Perms::from(READ | WRITE));

    assert_eq!(READ.complement(), Perms::from(WRITE | EXEC));
    assert_eq!(WRITE.complement(), Perms::from(READ | EXEC));
}

#[test]
fn test_flag_basic_traits() {
    assert_eq!(READ.clone(), READ);
    #[expect(clippy::manual_assert_eq, reason = "explicit equality test")]
    (assert!(READ == READ));
    assert_eq!(READ.cmp(&READ), Ordering::Equal);
    assert_eq!(READ.partial_cmp(&READ), Some(Ordering::Equal));

    assert!(READ < WRITE);
    assert_eq!(READ.cmp(&WRITE), Ordering::Less);
    assert_eq!(READ.partial_cmp(&WRITE), Some(Ordering::Less));

    let mut set = [
        Perm::from_bit_retain(4),
        READ,
        WRITE,
        EXEC,
        READ,
        WRITE,
        EXEC,
        WRITE,
        EXEC,
        EXEC,
    ];
    set.sort_unstable();
    assert_eq!(
        set,
        [
            READ,
            READ,
            WRITE,
            WRITE,
            WRITE,
            EXEC,
            EXEC,
            EXEC,
            EXEC,
            Perm::from_bit_retain(4),
        ]
    );
}

#[test]
fn test_flag_fmt_unnamed() {
    let bit5 = UnnamedPerm::from_bit_retain(4);

    assert_eq!(format!("{U_READ}"), "0x1");
    assert_eq!(format!("{U_WRITE}"), "0x2");
    assert_eq!(format!("{U_EXEC}"), "0x4");
    assert_eq!(format!("{bit5}"), "0x10");

    assert_eq!(format!("{bit5:b}"), "10000");
    assert_eq!(format!("{bit5:#b}"), "0b10000");
    assert_eq!(format!("{bit5:o}"), "20");
    assert_eq!(format!("{bit5:#o}"), "0o20");
    assert_eq!(format!("{bit5:x}"), "10");
    assert_eq!(format!("{bit5:#x}"), "0x10");
    assert_eq!(format!("{bit5:X}"), "10");
    assert_eq!(format!("{bit5:#X}"), "0x10");

    assert_eq!(
        format!("{bit5:?}"),
        "BitFlag(1 << 4, PhantomData<(u8, flag::UnnamedPermsTag)>)"
    );
    assert_eq!(
        format!("{bit5:#?}"),
        "BitFlag(1 << 4, PhantomData<(u8, flag::UnnamedPermsTag)>)"
    );
}

#[test]
fn test_flag_fmt_named() {
    let bit5 = Perm::from_bit_retain(4);

    assert_eq!(format!("{READ}"), "read");
    assert_eq!(format!("{WRITE}"), "write");
    assert_eq!(format!("{EXEC}"), "exec");
    assert_eq!(format!("{bit5}"), "0x10");

    assert_eq!(format!("{bit5:b}"), "10000");
    assert_eq!(format!("{bit5:#b}"), "0b10000");
    assert_eq!(format!("{bit5:o}"), "20");
    assert_eq!(format!("{bit5:#o}"), "0o20");
    assert_eq!(format!("{bit5:x}"), "10");
    assert_eq!(format!("{bit5:#x}"), "0x10");
    assert_eq!(format!("{bit5:X}"), "10");
    assert_eq!(format!("{bit5:#X}"), "0x10");

    assert_eq!(
        format!("{bit5:?}"),
        "BitFlag(1 << 4, PhantomData<(u8, flag::PermsTag)>)"
    );
    assert_eq!(
        format!("{bit5:#?}"),
        "BitFlag(1 << 4, PhantomData<(u8, flag::PermsTag)>)"
    );
}

#[test]
fn test_flag_from_str() {
    let bit5 = Perms::from_bits_retain(0x10);

    assert_eq!(Perm::from_str("read").unwrap(), READ);
    assert_eq!(Perm::from_str("write").unwrap(), WRITE);
    assert_eq!(Perm::from_str("exec").unwrap(), EXEC);
    assert_eq!(Perm::from_str("execute").unwrap(), EXEC);
    assert_eq!(Perm::from_str("0x10").unwrap(), bit5);

    assert!(Perm::from_str("READ").is_err());
    assert!(Perm::from_str("WRITE").is_err());
    assert!(Perm::from_str("EXEC").is_err());
    assert!(Perm::from_str("EXECUTE").is_err());
    assert!(Perm::from_str("0x100").is_err());
    assert!(Perm::from_str("0x+10").is_err());

    #[cfg(feature = "alloc")]
    assert_eq!(
        Perm::from_str("READ").err().unwrap().to_string(),
        "unrecognized named flag or hex flag, given: \"READ\", expected one of \"read\", \
         \"write\", \"exec\", or a valid one-bit hex flag with \"0x\" prefix"
    );
    #[cfg(not(feature = "alloc"))]
    assert_eq!(
        Perm::from_str("READ").err().unwrap().to_string(),
        "unrecognized named flag or hex flag, expected one of \"read\", \"write\", \"exec\", or a \
         valid one-bit hex flag with \"0x\" prefix"
    );
}

#[test]
fn test_flag_try_from_str() {
    use bitflags_lite::NameMatcher::{AsciiCaseInsensitive as Aci, Exact};

    let bit5 = Perms::from_bits_retain(0x10);

    assert_eq!(Perm::try_from_str("read", Exact).unwrap(), READ);
    assert_eq!(Perm::try_from_str("write", Exact).unwrap(), WRITE);
    assert_eq!(Perm::try_from_str("exec", Exact).unwrap(), EXEC);
    assert_eq!(Perm::try_from_str("execute", Exact).unwrap(), EXEC);
    assert_eq!(Perm::try_from_str("0x10", Exact).unwrap(), bit5);

    assert!(Perm::try_from_str("READ", Exact).is_err());
    assert!(Perm::try_from_str("WRITE", Exact).is_err());
    assert!(Perm::try_from_str("EXEC", Exact).is_err());
    assert!(Perm::try_from_str("EXECUTE", Exact).is_err());
    assert!(Perm::try_from_str("0x100", Exact).is_err());
    assert!(Perm::try_from_str("0x+10", Exact).is_err());

    assert_eq!(Perm::try_from_str("read", Aci).unwrap(), READ);
    assert_eq!(Perm::try_from_str("write", Aci).unwrap(), WRITE);
    assert_eq!(Perm::try_from_str("exec", Aci).unwrap(), EXEC);
    assert_eq!(Perm::try_from_str("execute", Aci).unwrap(), EXEC);

    assert_eq!(Perm::try_from_str("READ", Aci).unwrap(), READ);
    assert_eq!(Perm::try_from_str("WRITE", Aci).unwrap(), WRITE);
    assert_eq!(Perm::try_from_str("EXEC", Aci).unwrap(), EXEC);
    assert_eq!(Perm::try_from_str("EXECUTE", Aci).unwrap(), EXEC);

    assert_eq!(Perm::try_from_str("ReaD", Aci).unwrap(), READ);
    assert_eq!(Perm::try_from_str("WritE", Aci).unwrap(), WRITE);
    assert_eq!(Perm::try_from_str("ExeC", Aci).unwrap(), EXEC);

    assert!(Perm::try_from_str("re", Aci).is_err());
    assert!(Perm::try_from_str("ad", Aci).is_err());
    assert!(Perm::try_from_str("readable", Aci).is_err());

    #[cfg(feature = "alloc")]
    assert_eq!(
        Perm::try_from_str("READ", Exact).err().unwrap().to_string(),
        "unrecognized named flag or hex flag, given: \"READ\", expected one of \"read\", \
         \"write\", \"exec\", or a valid one-bit hex flag with \"0x\" prefix"
    );
    #[cfg(not(feature = "alloc"))]
    assert_eq!(
        Perm::try_from_str("READ", Exact).err().unwrap().to_string(),
        "unrecognized named flag or hex flag, expected one of \"read\", \"write\", \"exec\", or a \
         valid one-bit hex flag with \"0x\" prefix"
    );
}

#[test]
fn test_flag_try_from_name() {
    use bitflags_lite::NameMatcher::{AsciiCaseInsensitive as Aci, Exact};

    assert_eq!(Perm::try_from_name("read", Exact).unwrap(), READ);
    assert_eq!(Perm::try_from_name("write", Exact).unwrap(), WRITE);
    assert_eq!(Perm::try_from_name("exec", Exact).unwrap(), EXEC);
    assert_eq!(Perm::try_from_name("execute", Exact).unwrap(), EXEC);

    assert!(Perm::try_from_name("READ", Exact).is_err());
    assert!(Perm::try_from_name("WRITE", Exact).is_err());
    assert!(Perm::try_from_name("EXEC", Exact).is_err());
    assert!(Perm::try_from_name("EXECUTE", Exact).is_err());
    assert!(Perm::try_from_name("0x10", Exact).is_err());
    assert!(Perm::try_from_name("0x100", Exact).is_err());
    assert!(Perm::try_from_name("0x+10", Exact).is_err());

    assert_eq!(Perm::try_from_name("read", Aci).unwrap(), READ);
    assert_eq!(Perm::try_from_name("write", Aci).unwrap(), WRITE);
    assert_eq!(Perm::try_from_name("exec", Aci).unwrap(), EXEC);
    assert_eq!(Perm::try_from_name("execute", Aci).unwrap(), EXEC);

    assert_eq!(Perm::try_from_name("READ", Aci).unwrap(), READ);
    assert_eq!(Perm::try_from_name("WRITE", Aci).unwrap(), WRITE);
    assert_eq!(Perm::try_from_name("EXEC", Aci).unwrap(), EXEC);
    assert_eq!(Perm::try_from_name("EXECUTE", Aci).unwrap(), EXEC);

    assert_eq!(Perm::try_from_name("ReaD", Aci).unwrap(), READ);
    assert_eq!(Perm::try_from_name("WritE", Aci).unwrap(), WRITE);
    assert_eq!(Perm::try_from_name("ExeC", Aci).unwrap(), EXEC);

    assert!(Perm::try_from_name("re", Aci).is_err());
    assert!(Perm::try_from_name("ad", Aci).is_err());
    assert!(Perm::try_from_name("readable", Aci).is_err());

    #[cfg(feature = "alloc")]
    assert_eq!(
        Perm::try_from_name("READ", Exact)
            .err()
            .unwrap()
            .to_string(),
        "unrecognized named flag, given: \"READ\", expected one of \"read\", \"write\", \"exec\", \
         or a valid one-bit hex flag with \"0x\" prefix"
    );
    #[cfg(not(feature = "alloc"))]
    assert_eq!(
        Perm::try_from_name("READ", Exact)
            .err()
            .unwrap()
            .to_string(),
        "unrecognized named flag, expected one of \"read\", \"write\", \"exec\", or a valid \
         one-bit hex flag with \"0x\" prefix"
    );

    assert_eq!(
        UnnamedPerm::try_from_name("READ", Exact)
            .err()
            .unwrap()
            .to_string(),
        "does not support named flags"
    );
}

#[test]
fn test_flag_try_from_name_with_no_names() {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
    struct FlagsTag;
    type Flag = BitFlag<u8, FlagsTag>;

    impl BitFlagsSpec<u8> for FlagsTag {
        const VALID_BITS_MASK: u8 = 0b111;
        const DEFAULT_BITS: u8 = 0b000;
        const FLAG_NAMES: &[&[&str]] = &[&[], &[], &[]];
        const COMBINED_NAMES: &[(u8, &str)] = &[];
        const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
    }

    assert_eq!(
        Flag::try_from_name("READ", NameMatcher::Exact)
            .err()
            .unwrap()
            .to_string(),
        "does not support named flags"
    );
}

#[test]
fn test_flag_try_from_name_with_skipped() {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
    struct FlagsTag;
    type Flag = BitFlag<u8, FlagsTag>;

    impl BitFlagsSpec<u8> for FlagsTag {
        const VALID_BITS_MASK: u8 = 0b111;
        const DEFAULT_BITS: u8 = 0b000;
        const FLAG_NAMES: &[&[&str]] = &[&["first"], &[], &["third"]];
        const COMBINED_NAMES: &[(u8, &str)] = &[];
        const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
    }

    #[cfg(feature = "alloc")]
    assert_eq!(
        Flag::try_from_name("READ", NameMatcher::Exact)
            .err()
            .unwrap()
            .to_string(),
        "unrecognized named flag, given: \"READ\", expected one of \"first\", \"third\", or a \
         valid one-bit hex flag with \"0x\" prefix"
    );
    #[cfg(not(feature = "alloc"))]
    assert_eq!(
        Flag::try_from_name("READ", NameMatcher::Exact)
            .err()
            .unwrap()
            .to_string(),
        "unrecognized named flag, expected one of \"first\", \"third\", or a valid one-bit hex \
         flag with \"0x\" prefix"
    );
}

#[test]
fn test_flag_try_from_hex() {
    assert!(Perm::try_from_hex("read").is_err());
    assert!(Perm::try_from_hex("write").is_err());
    assert!(Perm::try_from_hex("exec").is_err());
    assert!(Perm::try_from_hex("execute").is_err());

    assert!(Perm::try_from_hex("READ").is_err());
    assert!(Perm::try_from_hex("WRITE").is_err());
    assert!(Perm::try_from_hex("EXEC").is_err());
    assert!(Perm::try_from_hex("EXECUTE").is_err());

    assert_eq!(Perm::try_from_hex("0x1").unwrap(), READ);
    assert_eq!(Perm::try_from_hex("0x2").unwrap(), WRITE);
    assert_eq!(Perm::try_from_hex("0x4").unwrap(), EXEC);
    assert_eq!(
        Perm::try_from_hex("0x10").unwrap(),
        Perms::from_bits_retain(0x10)
    );
    assert_eq!(
        Perm::try_from_hex("0x80").unwrap(),
        Perms::from_bits_retain(0x80)
    );

    assert!(Perm::try_from_hex("").is_err());
    assert!(Perm::try_from_hex("0").is_err());
    assert!(Perm::try_from_hex("1").is_err());
    assert!(Perm::try_from_hex("2").is_err());
    assert!(Perm::try_from_hex("0x").is_err());
    assert!(Perm::try_from_hex("0x0").is_err());
    assert!(Perm::try_from_hex("0x3").is_err());
    assert!(Perm::try_from_hex("0x5").is_err());
    assert!(Perm::try_from_hex("0x6").is_err());
    assert!(Perm::try_from_hex("0x7").is_err());
    assert!(Perm::try_from_hex("0xG").is_err());
    assert!(Perm::try_from_hex("0x100").is_err());
    assert!(Perm::try_from_hex("0x+10").is_err());

    #[cfg(feature = "alloc")]
    assert_eq!(
        Perm::try_from_hex("READ").err().unwrap().to_string(),
        "unrecognized hex flag, given: \"READ\", expected a valid one-bit hex flag with \"0x\" \
         prefix"
    );
    #[cfg(not(feature = "alloc"))]
    assert_eq!(
        Perm::try_from_hex("READ").err().unwrap().to_string(),
        "unrecognized hex flag, expected a valid one-bit hex flag with \"0x\" prefix"
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

    const FMT_LEN: usize = 139;

    #[cfg(feature = "alloc")]
    const NAMED_ERR_FMT_LEN: usize = 141;
    #[cfg(not(feature = "alloc"))]
    const NAMED_ERR_FMT_LEN: usize = 122;

    #[cfg(feature = "alloc")]
    const UNNAMED_ERR_FMT_LEN: usize = 92;
    #[cfg(not(feature = "alloc"))]
    const UNNAMED_ERR_FMT_LEN: usize = 73;

    #[cfg(feature = "alloc")]
    const NAMED_FROM_NAME_ERR_FMT_LEN: usize = 129;
    #[cfg(not(feature = "alloc"))]
    const NAMED_FROM_NAME_ERR_FMT_LEN: usize = 110;

    const UNNAMED_FROM_NAME_ERR_FMT_LEN: usize = 28;

    #[cfg(feature = "alloc")]
    const FROM_HEX_ERR_FMT_LEN: usize = 92;
    #[cfg(not(feature = "alloc"))]
    const FROM_HEX_ERR_FMT_LEN: usize = 73;

    let flag = WRITE;
    for i in 0..FMT_LEN {
        assert!(
            write!(
                &mut SmallWriter(i),
                "{flag} {flag:#} {flag:b} {flag:#b} {flag:o} {flag:#o} {flag:x} {flag:#x} \
                 {flag:X} {flag:#X} {flag:?} {flag:#?}"
            )
            .is_err(),
            "Unexpectedly succeed at {i}"
        );
    }
    assert!(
        write!(
            &mut SmallWriter(FMT_LEN),
            "{flag} {flag:#} {flag:b} {flag:#b} {flag:o} {flag:#o} {flag:x} {flag:#x} {flag:X} \
             {flag:#X} {flag:?} {flag:#?}"
        )
        .is_ok()
    );

    let error = Perm::from_str("readable").err().unwrap();
    for i in 0..NAMED_ERR_FMT_LEN {
        assert!(write!(&mut SmallWriter(i), "{error}").is_err(), "{i}");
    }
    assert!(write!(&mut SmallWriter(NAMED_ERR_FMT_LEN), "{error}").is_ok());

    let error = UnnamedPerm::from_str("readable").err().unwrap();
    for i in 0..UNNAMED_ERR_FMT_LEN {
        assert!(write!(&mut SmallWriter(i), "{error}").is_err(), "{i}");
    }
    assert!(write!(&mut SmallWriter(UNNAMED_ERR_FMT_LEN), "{error}").is_ok());

    let error = Perm::try_from_name("readable", NameMatcher::Exact)
        .err()
        .unwrap();
    for i in 0..NAMED_FROM_NAME_ERR_FMT_LEN {
        assert!(write!(&mut SmallWriter(i), "{error}").is_err(), "{i}");
    }
    assert!(write!(&mut SmallWriter(NAMED_FROM_NAME_ERR_FMT_LEN), "{error}").is_ok());

    let error = UnnamedPerm::try_from_name("readable", NameMatcher::Exact)
        .err()
        .unwrap();
    for i in 0..UNNAMED_FROM_NAME_ERR_FMT_LEN {
        assert!(write!(&mut SmallWriter(i), "{error}").is_err(), "{i}");
    }
    assert!(write!(&mut SmallWriter(UNNAMED_FROM_NAME_ERR_FMT_LEN), "{error}").is_ok());

    let error = UnnamedPerm::try_from_hex("readable").err().unwrap();
    for i in 0..FROM_HEX_ERR_FMT_LEN {
        assert!(write!(&mut SmallWriter(i), "{error}").is_err(), "{i}");
    }
    assert!(write!(&mut SmallWriter(FROM_HEX_ERR_FMT_LEN), "{error}").is_ok());
}

#[cfg(feature = "arbitrary")]
#[test]
fn test_flag_arbitrary() {
    use arbitrary::{Arbitrary, Unstructured};

    let data = &[0, 1, 2, 3];
    let mut u = Unstructured::new(data);
    assert_eq!(<[Perm; 4]>::size_hint(1), (4, Some(4)));
    assert_eq!(
        <[Perm; 4]>::arbitrary(&mut u).unwrap(),
        [READ, WRITE, EXEC, Perm::from_bit_retain(3)]
    );
}

#[cfg(feature = "bytemuck")]
#[test]
fn test_flag_bytemuck() {
    use bytemuck::{Pod, Zeroable};

    fn assert_pod<T: Pod>() {}
    fn assert_zeroable<T: Zeroable>() {}

    assert_pod::<Perm>();
    assert_zeroable::<Perm>();

    let f: Perm = Zeroable::zeroed();
    assert_eq!(f.bit(), 0);

    let bits = 2;
    let f: Perm = bytemuck::cast(bits);
    assert_eq!(f.bit(), 2);

    let raw: u8 = bytemuck::cast(f);
    assert_eq!(raw, bits);

    let flags = [READ, WRITE, EXEC, Perm::from_bit_retain(3)];
    let bytes: &[u8] = bytemuck::cast_slice(&flags);
    assert_eq!(bytes, &[0, 1, 2, 3]);
}

#[cfg(feature = "serde")]
#[test]
fn test_flag_serde_unnamed() {
    use serde_test::{Token, assert_de_tokens_error, assert_tokens};

    #[cfg(not(feature = "alloc"))]
    const ERROR_CORE: &str =
        "unrecognized hex flag, expected a valid one-bit hex flag with \"0x\" prefix";

    assert_tokens(&U_READ, &[Token::Str("0x1")]);
    assert_tokens(&U_WRITE, &[Token::Str("0x2")]);
    assert_tokens(&U_EXEC, &[Token::Str("0x4")]);
    assert_tokens(&Perm::from_bit_retain(3), &[Token::Str("0x8")]);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<UnnamedPerm>(
        &[Token::Str("5")],
        "unrecognized hex flag, given: \"5\", expected a valid one-bit hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<UnnamedPerm>(&[Token::Str("5")], ERROR_CORE);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<UnnamedPerm>(
        &[Token::Str("red")],
        "unrecognized hex flag, given: \"red\", expected a valid one-bit hex flag with \"0x\" \
         prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<UnnamedPerm>(&[Token::Str("red")], ERROR_CORE);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<UnnamedPerm>(
        &[Token::Str("0xCOFFEE")],
        "unrecognized hex flag, given: \"0xCOFFEE\", expected a valid one-bit hex flag with \
         \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<UnnamedPerm>(&[Token::Str("0xCOFFEE")], ERROR_CORE);

    assert_de_tokens_error::<UnnamedPerm>(
        &[Token::I32(123)],
        "invalid type: integer `123`, expected a valid one-bit hex flag with \"0x\" prefix",
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_flag_serde_named() {
    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error, assert_tokens};

    #[cfg(not(feature = "alloc"))]
    const ERROR_CORE: &str = "unrecognized named flag or hex flag, expected one of \"read\", \
                              \"write\", \"exec\", or a valid one-bit hex flag with \"0x\" prefix";

    assert_tokens(&READ, &[Token::Str("read")]);
    assert_tokens(&WRITE, &[Token::Str("write")]);
    assert_tokens(&EXEC, &[Token::Str("exec")]);
    assert_de_tokens(&EXEC, &[Token::Str("execute")]);
    assert_tokens(&Perm::from_bit_retain(3), &[Token::Str("0x8")]);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<Perm>(
        &[Token::Str("5")],
        "unrecognized named flag or hex flag, given: \"5\", expected one of \"read\", \"write\", \
         \"exec\", or a valid one-bit hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<Perm>(&[Token::Str("5")], ERROR_CORE);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<Perm>(
        &[Token::Str("red")],
        "unrecognized named flag or hex flag, given: \"red\", expected one of \"read\", \
         \"write\", \"exec\", or a valid one-bit hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<Perm>(&[Token::Str("red")], ERROR_CORE);

    #[cfg(feature = "alloc")]
    assert_de_tokens_error::<Perm>(
        &[Token::Str("0xCOFFEE")],
        "unrecognized named flag or hex flag, given: \"0xCOFFEE\", expected one of \"read\", \
         \"write\", \"exec\", or a valid one-bit hex flag with \"0x\" prefix",
    );
    #[cfg(not(feature = "alloc"))]
    assert_de_tokens_error::<Perm>(&[Token::Str("0xCOFFEE")], ERROR_CORE);

    assert_de_tokens_error::<Perm>(
        &[Token::I32(123)],
        "invalid type: integer `123`, expected one of \"read\", \"write\", \"exec\", or a valid \
         one-bit hex flag with \"0x\" prefix",
    );
}
