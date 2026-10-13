# bitflags-lite: Lightweight generics-based bitflags

<!---
TODO: Enable badges after publishing to crates.io.

[![Build Status](https://github.com/zheland/bitflags-lite/workflows/build/badge.svg)](https://github.com/zheland/bitflags-lite/actions)
[![Latest Version](https://img.shields.io/crates/v/bitflags-lite.svg)](https://crates.io/crates/bitflags-lite)
[![Documentation](https://docs.rs/bitflags-lite/badge.svg)](https://docs.rs/bitflags-lite)
[![Codecov](https://codecov.io/gh/zheland/bitflags-lite/graph/badge.svg)](https://codecov.io/gh/zheland/bitflags-lite)
[![Dependencies status](https://deps.rs/repo/github/zheland/bitflags-lite/status.svg)](https://deps.rs/repo/github/zheland/bitflags-lite)
[![Downloads](https://img.shields.io/crates/d/bitflags-lite)](https://crates.io/crates/bitflags-lite)
[![License](https://img.shields.io/crates/l/bitflags-lite)](https://github.com/zheland/bitflags-lite/#license)
[![MSRV 1.97+](https://img.shields.io/badge/rustc-1.97+-blue.svg)](https://blog.rust-lang.org/2026/07/09/Rust-1.97.0/)
-->

The `bitflags-lite` crate provides a lightweight, macro-free, generic bitflags
implementation for Rust.

`bitflags-lite` provides `BitFlags<T, U>`, a `#[repr(transparent)]` wrapper
around an unsigned integer `T` that adds a compile-time tag type `U`.
The tag type distinguishes flag sets that share the same storage type and
defines which bits are valid through the `BitFlagsSpec` trait.

It is designed to be simple, predictable, and const-friendly.


## Usage

```toml
[dependencies]
bitflags-lite = "0.1.0-dev.0"
```

With optional features:

```toml
[dependencies]
bitflags-lite = { version = "0.1.0-dev.0", features = [ "serde", "bytemuck" ] }
```


## Example

Define a tag type and implement `BitFlagsSpec` for it:

```rust
use bitflags_lite::{BitFlags, BitFlagsSpec};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct PermissionsTag;

impl BitFlagsSpec<u32> for PermissionsTag {
    const VALID_BITS: u32 = (1 << 3) - 1;
    const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
    const IS_DEFAULT_PARSING_ASCII_CASE_SENSITIVE: bool = true;
}

type Permissions = BitFlags<u32, PermissionsTag>;

// Compile-time const unwrapping is fine.
const READ: Permissions = Permissions::from_bits(1 << 0).unwrap();
const WRITE: Permissions = Permissions::from_bits(1 << 1).unwrap();
const EXEC: Permissions = Permissions::from_bits(1 << 2).unwrap();

fn main() {
    let rwx = READ | WRITE | EXEC;
    assert!(rwx.contains(READ));
    assert!(rwx.contains(WRITE));
    assert!(rwx.contains(EXEC));

    let read_only = READ;
    assert!(!read_only.contains(WRITE));
}
```

You can also define constants through a trait or an inherent implementation if
you prefer.


## Comparison with Existing Crates

| Aspect | `bitflags-lite` | `bitflags` v2 | `enumflags2` v0.7 | 
|:--:|:--:|:--:|:--:|
| Definition mechanism | Trait `BitFlagsSpec<T>` for a tag type | `bitflags! { ... }` macro | `#[bitflags]` attribute macro |
| Generated/used type | `BitFlags<T, U>` over a tag type `U` | User-owned struct | `BitFlags<T, N = T::Numeric>` over an enum |

| Aspect | `bitflags-lite` | `bitflags` v2 | `enumflags2` v0.7 | 
|:--:|:--:|:--:|:--:|
| Automatically deduced flag names | ⛔️ No | ✅ Yes | ✅ Yes |
| Automatically deduced valid bits | ⛔️ No | ✅ Yes | ✅ Yes |
| Most methods const | ✅ Yes | ✅ Yes | ⚠️ Limited |
| Multi-bit flags | ⛔️ No | ✅ Yes | ⛔️ No |
| Single-flag vs set distinction | ⛔️ No | ⛔️ No | ✅ Yes |
| Alias names for parsing | ✅ Yes | ⛔️ No | ⛔️ No |
| Rename a flag | ✅ Yes | ✅ Yes | ⛔️ No |
| Custom default | ⛔️ No | ✅ Yes | ✅ Yes |
| Are invalid bits allowed? | ✅ Yes | ✅ Yes | ⚠️ Unsafe |
| Format with names | ✅ Yes | ✅ Yes | ✅ Yes |
| Parse from names | ✅ Yes | ✅ Yes | ⛔️ No |

### 10. Behavioral notes worth documenting

These don’t fit into a single cell but should be mentioned in the README.

#### Multi-bit flags
`bitflags` allows combined flags (`const AB = A | B;`). `enumflags2` and `bitflags-lite` do not. In `bitflags-lite`, every named flag is exactly one bit; combinations are expressed only through operators and the set type.

#### Unknown bits
- `bitflags`: unknown bits can be preserved by raw operators; `!` truncates to known bits. Unnamed flags (`const _ = !0`) promote unknown bits to known.
- `enumflags2`: only bits corresponding to enum variants are valid; `from_bits` rejects others, `from_bits_truncate` clears them, and `!` truncates to `ALL_BITS`.
- `bitflags-lite`: `VALID_BITS` is explicit. `from_bits` rejects, `from_bits_truncate`/`truncate`/`complement`/`!` clear. `from_bits_retain` and raw operators preserve.

#### Parsing semantics
- `bitflags`: text format is `A | B | 0xc`; parse is case-sensitive; `std` controls whether errors include the offending string.
- `enumflags2`: no string parser in the attached source; serde deserializes from a numeric value.
- `bitflags-lite`: text format is `A | B | 0xc`; case sensitivity is configurable per tag; `alloc` controls whether errors include the offending string.

#### Serde semantics
- `bitflags`: human-readable formats get text, binary formats get a raw number.
- `enumflags2`: always a raw numeric value.
- `bitflags-lite`: always a string via `Display` (named flags plus hex for remaining bits). Note: the README currently says “binary string such as `"0b101"`”, but the code uses `Display`, which produces named-flag text. This is a doc/code mismatch worth fixing.

#### Const support
- `bitflags`: the macro-generated methods are mostly `const fn`, so no special token is needed.
- `enumflags2`: const traits are unstable, so const variants are suffixed `_c` and take a `ConstToken`.
- `bitflags-lite`: core methods are `const fn` on concrete integer types generated by `impl_bit_flags!`; no token or suffix is needed.

#### Type-level single-flag vs set distinction
- `enumflags2` distinguishes `T` (one flag) from `BitFlags<T>` (a set). This enables `PartialEq<T>`, `Into<BitFlags<T>>`, and iteration yielding `T`.
- `bitflags` and `bitflags-lite` use one type for both. `bitflags-lite` iteration yields single-bit `BitFlags` values (or names), not a separate flag type.

#### Layout and variance
- All three are `#[repr(transparent)]` over the storage integer.
- `bitflags-lite` uses `PhantomData<fn() -> *const U>` so `BitFlags<T, U>` is `Send + Sync`, covariant over `U`, and does not require `U: Sized`.

### Suggestion for the `bitflags-lite` README

I’d add a short section like this after the example:

1. **Design summary** — macro-free, generic, tag-based, per-bit names, explicit `VALID_BITS`.
2. **Comparison table** (condensed version of the above) with columns `bitflags` / `enumflags2` / `bitflags-lite`.
3. **When to choose which** — short bullets:
   - `bitflags` if you want the ecosystem-standard macro, multi-bit convenience flags, unnamed wildcard flags, and the widest feature set.
   - `enumflags2` if you want a single-flag type distinct from the set type, enum-variant-driven definitions, and `match`/`iter` over variants.
   - `bitflags-lite` if you want no macros, a reusable generic `BitFlags<T, U>`, explicit `VALID_BITS`, multiple names per bit, case-insensitive parsing, and const-friendly methods without a `ConstToken` workaround.
4. **Caveats** — no combined/multi-bit flags, unsigned storage only, per-bit naming model, serde string format (and fix the README mismatch).
5. **One sentence** noting that `bitvec`, `bitfield`, and `modular-bitfield` solve different problems (bit vectors or struct bitfields) and are not direct alternatives.


## Documentation

[API Documentation]


## Feature flags

- `arbitrary`: implements `Arbitrary` for `BitFlags<T, U>`.
- `serde`: implements `serde::Serialize` and `serde::Deserialize` for
  `BitFlags<T, U>` with serialization as a binary string such as `"0b101"` and
   deserialization from a binary string, optionally prefixed with `0b`.
- `bytemuck`: implements `Zeroable` and `Pod` for `BitFlags<T, U>`, allowing it
  to be used in plain-old-data contexts.


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any
additional terms or conditions.

[API Documentation]: https://docs.rs/bitflags-lite/latest/bitflags-lite/
