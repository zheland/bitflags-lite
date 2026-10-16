# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.2.0] - 2026-10-16
### Added
- `bitflags!` macro for defining a flag set from a compact declaration.
  The macro generates a [`BitFlagsSpec`] implementation and named
  constants for individual bits and masks. It does not generate a new
  type: every flag set remains a `BitFlags<T, U>` over a user-defined
  tag. Type aliases for `BitFlags<T, U>` and `BitFlag<T, U>` are written
  by the user next to the macro invocation.
- `BitFlagsSpec::COMBINED_NAMES` associated const for names that parse to
  multi-bit values. Combined names are consulted by parsers but not by
  `Display`, which always writes the union of individual flag names.

## [0.1.0] - 2026-10-15
### Added
- `BitFlagsSpec<T>` trait for defining valid bits, flag names, and the
  default name matcher for a flag set.
- `NameMatcher` enum for exact and ASCII case-insensitive name matching.
- `BitFlags<T, U>` and `BitFlag<T, U>` types, tagged by a user-defined
  spec type `U` and stored in an integer `T`.
- `BitFlagsIter<T, U>` iterator over the set bits of a `BitFlags`.
- `BitFlagsExt<T, U>` and `BitFlagExt<T, U>` extension traits providing
  generic versions of the const methods on the concrete integer impls.
- Parse, format, and conversion APIs, including:
  - `TryFrom<BitFlags<T, U>> for BitFlag<T, U>` and
    `BitFlag::try_from_flags`.
  - `FromStr` for `BitFlags` and `BitFlag`.
  - `Display` with named-flag and hexadecimal output, `Debug`, `Binary`,
    `Octal`, `LowerHex`, and `UpperHex`.
- Standard comparison and hash trait implementations.
- `From`, `IntoIterator`, `Extend`, `FromIterator`.
- Error types: `BitFlagsParseError`, `BitFlagParseError`,
  `BitFlagParseNameError`, `BitFlagParseHexError`, and
  `BitFlagFromFlagsError`.
- Optional integrations: `arbitrary`, `serde`, and `bytemuck`.
- `no_std` and `no_alloc` support.

[Unreleased]: https://github.com/zheland/bitflags-lite/compare/v0.1.0...HEAD
[0.2.0]: https://github.com/zheland/bitflags-lite/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/zheland/bitflags-lite/compare/v0.0.0...v0.1.0
