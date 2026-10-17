# Migration guide

## [0.3.0]
- Methods `BitFlag::mask_or_default` and `BitFlagExt::mask_or_default` were
  replaced to `BitFlag::mask_or_empty` and `BitFlagExt::mask_or_empty`.

## [0.2.0]
- `BitFlagsSpec` now requires `COMBINED_NAMES` associated constant specified.
  Can be left empty if you don't need it.

[0.3.0]: https://github.com/zheland/typed-index-collections/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/zheland/typed-index-collections/compare/v0.1.0...v0.2.0
