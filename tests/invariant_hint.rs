//! A test checking that `BitFlags` invariants compiler hints active.

#![cfg(not(miri))]
#![cfg(not(debug_assertions))]
#![cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "arm"
))]
#![allow(unused_crate_dependencies, reason = "okay in tests")]

use core::fmt::{self, Display, Formatter};

use bitflags_lite::{BitFlags, BitFlagsSpec};

/// A tag for permissions bit flags allowing only valid bit combinations.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct PermsTag;

/// Permissions bit flags allowing only valid bit combinations.
pub type Perms = BitFlags<u32, PermsTag>;

impl BitFlagsSpec<u32> for PermsTag {
    const VALID_BITS: u32 = (1 << 3) - 1;
    type AnyBitsAllowed = bitflags_lite::False;
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "Guarantees no drop")]
const fn perms_are_valid(perms: &Perms) -> bool {
    perms.bits() & !0b111 == 0
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "Guarantees no drop")]
const fn perms_are_rw(perms: &Perms) -> bool {
    perms.bits() & !0b11 == 0
}

// The tests are known to be fragile but there is no more reliable alternative.
#[test]
fn test_invariant_hint() {
    assert!(is_fn_only_returns_true(perms_are_valid).is_ok());
    assert!(is_fn_only_returns_true(perms_are_rw).is_err());
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
type Opcode = u8;
#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
type Opcode = u32;

// Test with `--target i686-unknown-linux-gnu`.
// Test with `--target x86_64-unknown-linux-gnu`.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const EXPECTED_RET_TRUE_FN_BYTECODE: [Opcode; 3] = [
    0xB0, 0x01, // mov al, 0x1
    0xC3, // ret
];

// Test with `--target aarch64-unknown-linux-gnu`.
#[cfg(target_arch = "aarch64")]
const EXPECTED_RET_TRUE_FN_BYTECODE: [Opcode; 2] = [
    0x52800020, // mov w0, #1
    0xD65F03C0, // ret
];

// Test with `--target arm-unknown-linux-gnueabi`.
#[cfg(target_arch = "arm")]
const EXPECTED_RET_TRUE_FN_BYTECODE: [Opcode; 2] = [
    0xE3A00001, // mov r0, #1
    0xE12FFF1E, // bx lr
];

#[inline]
fn is_fn_only_returns_true<T>(func: extern "Rust" fn(&T) -> bool) -> Result<(), OpcodeError> {
    let ptr = fn_raw_ptr(func);
    let ptr: *const Opcode = ptr.cast();
    for (offset, &expected) in EXPECTED_RET_TRUE_FN_BYTECODE.iter().enumerate() {
        // SAFETY: Generally safe while expected instructions are discovered.
        let given = unsafe { ptr.add(offset).read() };
        if given != expected {
            return Err(OpcodeError {
                given,
                expected,
                offset,
            });
        }
    }
    Ok(())
}

/// Implicitly coerce function to function pointer.
#[expect(clippy::as_conversions, reason = "the idiomatic way")]
#[inline]
const fn fn_raw_ptr<T, R>(func: extern "Rust" fn(T) -> R) -> *const u8 {
    (func as *const fn(T) -> R).cast::<u8>()
}

#[derive(Clone, Debug)]
struct OpcodeError {
    given: Opcode,
    expected: Opcode,
    offset: usize,
}

impl Display for OpcodeError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "expected {:#b} at {}, got {:#b}",
            self.expected, self.offset, self.given
        )
    }
}
