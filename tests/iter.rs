//! Tests for `iter.rs` file.

#![allow(unused_crate_dependencies, reason = "okay in tests")]

use core::ops::{BitAndAssign, Not};

use bitflags_lite::{BitFlags, BitFlagsSpec, NameMatcher};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct BadInt(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct PermsTag;

impl BitFlagsSpec<BadInt> for PermsTag {
    const VALID_BITS_MASK: BadInt = BadInt(0xFF);
    const DEFAULT_BITS: BadInt = BadInt(0);
    const FLAG_NAMES: &[&[&str]] = &[];
    const COMBINED_NAMES: &[(BadInt, &str)] = &[];
    const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
}

impl Not for BadInt {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAndAssign<Self> for BadInt {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl bitflags_lite::util::IntOpsExt for BadInt {
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const BITS: u8 = 32;

    #[inline]
    fn checked_shl(&self, rhs: u32) -> Option<Self> {
        // Deliberately broken to catch the panic on dev builds.
        Some(Self(u32::checked_shl(self.0, rhs.wrapping_mul(2))?))
    }

    #[inline]
    fn count_ones(&self) -> u32 {
        u32::count_ones(self.0)
    }

    #[inline]
    fn lowest_one(&self) -> Option<u32> {
        u32::lowest_one(self.0)
    }

    #[inline]
    fn highest_one(&self) -> Option<u32> {
        u32::highest_one(self.0)
    }

    #[inline]
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, core::num::ParseIntError> {
        Ok(Self(u32::from_str_radix(src, radix)?))
    }
}

#[test]
#[cfg_attr(debug_assertions, should_panic(expected = "invalid implementation"))]
fn test_iterator_bad_ty_next() {
    let perms = BitFlags::<BadInt, PermsTag>::from_bits_retain(BadInt(0x1_0000));
    #[cfg(debug_assertions)]
    let _next = perms.into_iter().next();
    #[cfg(not(debug_assertions))]
    assert_eq!(perms.into_iter().next(), None);
}

#[test]
#[cfg_attr(debug_assertions, should_panic(expected = "invalid implementation"))]
fn test_iterator_bad_ty_next_back() {
    let perms = BitFlags::<BadInt, PermsTag>::from_bits_retain(BadInt(0x1_0000));
    #[cfg(debug_assertions)]
    let _next = perms.into_iter().next_back();
    #[cfg(not(debug_assertions))]
    assert_eq!(perms.into_iter().next_back(), None);
}
