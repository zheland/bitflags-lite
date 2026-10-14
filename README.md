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


## When to choose `bitflags`, `enumflags2`, and `bitflags-lite`

Choose
- [`bitflags`] if you want the ecosystem-standard macro, multi-bit convenience
flags, unnamed wildcard flags, and the widest feature set.
- [`enumflags2`] if you want a single-flag type distinct from the set type,
  enum-variant-driven definitions, and match/iter over variants.
- [`bitflags-lite`] if you want lightweight flags definition with no macros and
  a reusable generic BitFlags<T, U> and comfortable with per-bit naming model,
  and you don't need combined/multi-bit flags

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
use bitflags_lite::{BitFlags, BitFlagsSpec, NameMatcher};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct PermissionsTag;

impl BitFlagsSpec<u32> for PermissionsTag {
    const VALID_BITS_MASK: u32 = (1 << 3) - 1;
    const FLAG_NAMES: &[&[&str]] = &[&["read"], &["write"], &["exec", "execute"]];
    const DEFAULT_NAME_MATCHER: NameMatcher = NameMatcher::Exact;
}

type Permissions = BitFlags<u32, PermissionsTag>;

// Compile-time const unwrapping is fine.
const READ: Permissions = Permissions::try_from_bit(0).unwrap();
const WRITE: Permissions = Permissions::try_from_bit(1).unwrap();
const EXEC: Permissions = Permissions::try_from_bit(2).unwrap();

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

| Feature | `bitflags-lite` | `bitflags` v2 | `enumflags2` v0.7 | 
|:--:|:--:|:--:|:--:|
| Automatically deduced flag names | ⛔️ No | ✅ Yes | ✅ Yes |
| Automatically deduced valid bits | ⛔️ No | ✅ Yes | ✅ Yes |
| Most methods const | ✅ Yes | ✅ Yes | ⚠️ Limited |
| Multi-bit flags | ⛔️ No | ✅ Yes | ⛔️ No |
| Single-flag vs set distinction | ✅ Yes | ⛔️ No | ✅ Yes |
| Alias names for parsing | ✅ Yes | ⛔️ No | ⛔️ No |
| Rename a flag | ✅ Yes | ✅ Yes | ⛔️ No |
| Custom default | ⛔️ No | ✅ Yes | ✅ Yes |
| Only valid bits | ⛔️ No | ⛔️ No | ✅ Yes |
| Are invalid bits allowed? | ✅ Yes | ✅ Yes | ⛔️ No |
| Format with names | ✅ Yes | ✅ Yes | ✅ Yes |
| Parse from names | ✅ Yes | ✅ Yes | ⛔️ No |
| Case-insensitive parsing | ✅ Yes | ⛔️ No | ⛔️ No |

<!---
- Invalid bits are really not allowed in `enumflags2`.
  Its `from_bits_unchecked` unsafe method requires that
  "All bits set in val must correspond to a value of the enum".
-->


## Documentation

[API Documentation]


## Feature flags

- `arbitrary`: implements `Arbitrary` for `BitFlag<T, U>` and
  `BitFlags<T, U>`.
- `serde`: implements `serde::Serialize` and `serde::Deserialize` for
  `BitFlag<T, U>` and `BitFlags<T, U>`, serializing to and deserializing from
  flag strings.
- `bytemuck`: implements `Zeroable` and `Pod` for `BitFlag<T, U>` and
  `BitFlags<T, U>`, allowing them to be used in plain-old-data contexts.

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
[`bitflags`]: https://crates.io/crates/bitflags
[`enumflags2`]: https://crates.io/crates/enumflags2
[`bitflags-lite`]: https://crates.io/crates/bitflags-lite
