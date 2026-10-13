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

The crate is `no_std` and `no_alloc`.
It is designed to be simple, predictable, and const-friendly.


## Features

- Macro-free: No declarative or procedural macros are required.
- Generic: Use any of the standard unsigned integer types as storage and any
  type as a tag.
- `no_std` / `no_alloc`: Works in embedded and other constrained environments.
- `const fn` API: Most operations are `const fn`.
- Optional `serde`: Serialize and deserialize flags as binary strings.
- Optional `bytemuck`: Use flags in plain-old-data contexts.


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

 TODO: Compare with existing crates.
- Mention that for a `bitflags` set of 3 flags, the `bitflags` crate generates over 800 lines of code.
- Mention that, unlike `bitflags`, this crate cannot work with string flag values at all.

### Differences from `bitflags`

- Macro-free: no `bitflags!` invocation or generated module is required.
- Generic: the storage type `T` and tag type `U` are both type parameters.
- No string flag names: parsing, formatting, and iteration by name are not
  provided.
- `all_valid()` is used instead of `all()`: [`BitFlags::from_bits_retain`]
  can create values with invalid bits, so "all" is ambiguous.
  `all_valid()` always means exactly [`BitFlagsSpec::VALID_BITS`].
- Most operations are `const fn`, and the crate is `no_std` / `no_alloc`.



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
