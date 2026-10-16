//! The `bitflags-lite` crate provides a lightweight, macro-free, generic
//! bitflags implementation for Rust.
//!
//! [`BitFlags<T, U>`] is a `#[repr(transparent)]` wrapper around an integer `T`
//! that adds a compile-time tag type `U`.
//! The tag distinguishes flag sets that share the same storage type and
//! describes which bits are valid through the [`BitFlagsSpec`] trait.
//!
//! It is `no_std` and `no_alloc` compatible, and const-friendly.
//!
//!
//! # Why a generic `BitFlags<T, U>`?
//!
//! Macro-based bitflags crates generate a distinct struct for every flag set.
//! Each invocation re-expands the full method set, and the resulting types
//! share no common trait or type with one another.
//! By contrast, `bitflags-lite` defines [`BitFlags<T, U>`] once and asks you to
//! specialize it through a plain trait implementation.
//!
//! That design has a few concrete consequences:
//!
//! - **Generic code over flag sets.** Because every flag set is the same type
//!   family, you can write `fn f<T, U: BitFlagsSpec<T>>(flags: BitFlags<T,
//!   U>)`. This is not possible with macro-generated structs.
//! - **Predictable expansion.** A new flag set adds a small trait impl, not a
//!   fresh copy of the entire API.
//! - **Inspectable source.** The spec is ordinary Rust. `rustdoc` renders it,
//!   `rust-analyzer` navigates it, `grep` finds it.
//!
//! But you have to give up automatic derivation of flag names and valid bits
//! and write [`BitFlagsSpec::VALID_BITS_MASK`] and [`BitFlagsSpec::FLAG_NAMES`]
//! by hand and keep them in sync with your constants.
//!
//!
//! # When to choose `bitflags`, `enumflags2`, or `bitflags-lite`
//!
//! - Choose [`bitflags`] if you want the mature and battle-tested crate, the
//!   ecosystem-standard macro, multi-bit convenience flags, and unnamed
//!   wildcard flags.
//! - Choose [`enumflags2`] if you want a single-flag type distinct from the set
//!   type, enum-variant-driven definitions, and match/iter over variants.
//! - Choose [`bitflags-lite`] if you want a lightweight flags definition with
//!   no macros and a reusable generic [`BitFlags<T, U>`], are comfortable with
//!   a per-bit naming model, and you don't need combined or multi-bit flags.
//!
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! bitflags-lite = "0.1.0"
//! ```
//!
//! With optional features:
//!
//! ```toml
//! [dependencies]
//! bitflags-lite = { version = "0.1.0", features = [ "serde", "bytemuck" ] }
//! ```
//!
//!
//! # Examples
//!
//! Define a tag type and implement [`BitFlagsSpec`] for it:
//!
//! ```rust
//! use core::str::FromStr;
//!
//! use bitflags_lite::{BitFlag, BitFlags, BitFlagsSpec, NameMatcher};
//!
//! #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
//! struct PermissionsTag;
//!
//! impl BitFlagsSpec<u32> for PermissionsTag {
//!     const VALID_BITS_MASK: u32 = 0b111;
//!     const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
//!     const COMBINED_NAMES: &[(u8, &str)] = &[(0, "none"), (0b111, "rwx")];
//!     const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
//! }
//!
//! type Permission = BitFlag<u32, PermissionsTag>;
//! type Permissions = BitFlags<u32, PermissionsTag>;
//!
//! // Constructors are `const` for concrete integer types.
//! const READ: Permission = Permission::try_from_bit(0).unwrap();
//! const WRITE: Permission = Permission::try_from_bit(1).unwrap();
//! const EXEC: Permission = Permission::try_from_bit(2).unwrap();
//! const RWX: Permissions = Permissions::try_from_bits(0b111).unwrap();
//!
//! assert!(RWX.contains(READ.into()));
//! assert!(RWX.contains(WRITE.into()));
//! assert!(RWX.contains(EXEC.into()));
//! assert!(RWX.contains(READ | WRITE));
//!
//! let read_exec = READ | EXEC;
//! assert!(!read_exec.contains(WRITE.into()));
//!
//! let flags = Permissions::from_str("read | 0x18 | write").unwrap();
//! assert_eq!(format!("{flags}"), "read | write | 0x18");
//!
//! let flags: Vec<Permission> = flags.into_iter().collect();
//! assert_eq!(
//!     flags,
//!     [
//!         READ,
//!         WRITE,
//!         Permission::from_bit_retain(3),
//!         Permission::from_bit_retain(4),
//!     ]
//! );
//!
//! let formatted: Vec<String> = flags.iter().map(ToString::to_string).collect();
//! assert_eq!(formatted, ["read", "write", "0x8", "0x10"]);
//!
//! let named_only: Vec<&'static str> = flags.iter().filter_map(BitFlag::name).collect();
//! assert_eq!(named_only, ["read", "write"]);
//! ```
//!
//! Constants can also be defined through an inherent implementation, a trait,
//! or an associated `const` on the tag type — whatever fits your code style.
//!
//! The tag type keeps flag sets distinct even when they share a storage type,
//! providing type-safety across otherwise-similar types:
//!
//! The following example deliberately fails to compile:
//! ```rust,compile_fail
//! use bitflags_lite::{BitFlags, BitFlagsSpec, NameMatcher};
//!
//! struct PermsTag;
//! type Perms = BitFlags<u8, PermsTag>;
//! impl BitFlagsSpec<u8> for PermsTag {
//!     const VALID_BITS_MASK: u8 = (1 << 3) - 1;
//!     const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
//!     const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::AsciiCaseInsensitive;
//! }
//!
//! struct KeyboardModifiersTag;
//! type KeyboardModifiers = BitFlags<u8, KeyboardModifiersTag>;
//! impl BitFlagsSpec<u8> for KeyboardModifiersTag {
//!     const VALID_BITS_MASK: u8 = (1 << 4) - 1;
//!     const FLAG_NAMES: &[&[&str]] = &[&["Shift"], &["Ctrl"], &["Alt"], &["Meta"]];
//!     const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::AsciiCaseInsensitive;
//! }
//!
//! let rwx: Perms = Perms::try_from_bits(0b111).unwrap();
//! let keyboard_modifiers: KeyboardModifiers = rwx;
//! //                                          ^^^
//! // ❌ error[E0308]: mismatched types
//! // expected struct `BitFlags<_, KeyboardModifiersTag>`
//! //    found struct `BitFlags<_, test::PermsTag>`
//! ```
//!
//!
//! # Design Highlights
//!
//! - **Single-flag vs. set distinction.** [`BitFlag<T, U>`] represents one bit
//!   index. [`BitFlags<T, U>`] represents a set. They convert into each other
//!   and interoperate through the usual operators.
//! - **Invalid bits are representable.** A [`BitFlags`] value may contain bits
//!   not set in [`BitFlagsSpec::VALID_BITS_MASK`]; they are preserved by
//!   raw-bit operations and cleared by [`BitFlags::truncate`],
//!   [`BitFlags::complement`], and [`Not`](core::ops::Not). Use
//!   [`BitFlags::is_valid`] to check.
//! - **Const-friendly.** Every constructor, accessor, and combinator on the
//!   concrete integer types is a `const fn`. The generic equivalents live in
//!   the [`BitFlagsExt`] and [`BitFlagExt`] extension traits.
//! - **No allocation.** Parsing and formatting work without `alloc`. The
//!   `alloc` feature only enriches error messages with the offending input.
//!
//!
//! # Comparison with Existing Crates
//!
//! | Aspect | `bitflags-lite` | `bitflags` v2 | `enumflags2` v0.7 |
//! |:--:|:--:|:--:|:--:|
//! | Definition mechanism | Trait `BitFlagsSpec<T>` for a tag type | `bitflags! { ... }` macro | `#[bitflags]` attribute macro |
//! | Generated/used type | `BitFlags<T, U>` over a tag type `U` | User-owned struct | `BitFlags<T, N = T::Numeric>` over an enum |
//!
//! | Feature | `bitflags-lite` | `bitflags` v2 | `enumflags2` v0.7 |
//! |:--:|:--:|:--:|:--:|
//! | Automatically deduced flag names | ⛔️ No | ✅ Yes | ✅ Yes |
//! | Automatically deduced valid bits | ⛔️ No | ✅ Yes | ✅ Yes |
//! | Automatically sync names and valid bits | ⛔️ No | ✅ Yes | ✅ Yes |
//! | Most methods const | ✅ Yes | ✅ Yes | ⚠️ Limited |
//! | Multi-bit flags | ⛔️ No | ✅ Yes | ⛔️ No |
//! | Single-flag vs set distinction | ✅ Yes | ⛔️ No | ✅ Yes |
//! | Alias names for parsing | ✅ Yes | ⛔️ No | ⛔️ No |
//! | Rename a flag | ✅ Yes | ✅ Yes | ⛔️ No |
//! | Custom default | ⛔️ No | ✅ Yes | ✅ Yes |
//! | Only valid bits | ⛔️ No | ⛔️ No | ✅ Yes |
//! | Are invalid bits allowed? | ✅ Yes | ✅ Yes | ⛔️ No |
//! | Format with names | ✅ Yes | ✅ Yes | ✅ Yes |
//! | Parse from names | ✅ Yes | ✅ Yes | ⛔️ No |
//! | Case-insensitive parsing | ✅ Yes | ⛔️ No | ⛔️ No |
//!
//! <!---
//! - Invalid bits are really not allowed in `enumflags2`. Its
//!   `from_bits_unchecked` unsafe method requires that "All bits set in val
//!   must correspond to a value of the enum".
//! -->
//!
//!
//! # Documentation
//!
//! [API Documentation]
//!
//!
//! # Feature flags
//!
//!
//! - `alloc` (implied by `std`, enabled by default): enriches parse errors with
//!   the offending input.
//! - `std` (enabled by default): enables `alloc` feature.
//! - `arbitrary`: implements `Arbitrary` for `BitFlag<T, U>` and `BitFlags<T,
//!   U>`.
//! - `serde`: implements `serde::Serialize` and `serde::Deserialize` for
//!   `BitFlag<T, U>` and `BitFlags<T, U>` round-tripping through flag strings.
//! - `bytemuck`: implements `Zeroable` and `Pod` for `BitFlag<T, U>` and
//!   `BitFlags<T, U>`, allowing them to be used in plain-old-data contexts.
//!
//!
//! # License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//!
//! # Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally
//! submitted for inclusion in the work by you, as defined in the Apache-2.0
//! license, shall be dual licensed as above, without any
//! additional terms or conditions.
//!
//! [API Documentation]: https://docs.rs/bitflags-lite/latest/bitflags-lite/
//! [`bitflags`]: https://crates.io/crates/bitflags
//! [`enumflags2`]: https://crates.io/crates/enumflags2
//! [`bitflags-lite`]: https://crates.io/crates/bitflags-lite
//! [`BitFlag<T, U>`]: struct.BitFlag.html
//! [`BitFlags<T, U>`]: struct.BitFlags.html
//! [`BitFlags`]: struct.BitFlags.html
//! [`BitFlagExt`]: trait.BitFlagExt.html
//! [`BitFlagsExt`]: trait.BitFlagsExt.html
//! [`BitFlagsSpec`]: trait.BitFlagsSpec.html

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod flag;
mod flag_ext;
mod flags;
mod flags_ext;
mod iter;
mod spec;
pub mod util;

pub use flag::*;
pub use flag_ext::*;
pub use flags::*;
pub use flags_ext::*;
pub use iter::*;
pub use spec::*;

#[cfg(all(test, not(doc)))]
mod integration_tests_deps {
    use serde_test as _;
    use version_sync as _;
}
