/// Defines a flag set from a compact declaration.
///
/// The macro generates a [`BitFlagsSpec`] implementation for a tag type,
/// along with named constants for individual bits and combined masks.
///
/// This is a convenience wrapper around the manual [`BitFlagsSpec`]
/// implementation.
/// The expansion is small and inspectable.
/// Running `cargo expand` shows code that matches what you would write by hand.
///
/// The macro does not generate a type alias for the flag set. Type aliases
/// are trivial to write in ordinary Rust, and generating one would create a
/// naming asymmetry: the flag set would have a name but the individual flag
/// would not. Write the aliases you need next to the macro invocation:
///
/// ```
/// use bitflags_lite::{BitFlag, BitFlags};
/// bitflags_lite::bitflags! {
///     #![storage(u32)]
///     pub struct PermissionsTag;
///     // ...
/// }
/// pub type Permissions = BitFlags<u32, PermissionsTag>;
/// pub type Permission = BitFlag<u32, PermissionsTag>;
/// ```
///
/// # Syntax
///
/// ```text
/// bitflags! {
///     // Storage type. Required.
///     #![storage( STORAGE )]
///
///     // Optional default name matcher.
///     $( #![name_matcher( EXPR )] )?
///
///     // Tag declaration. The `VIS struct` prefix is optional.
///     $( $( #[$ATTR] )* VIS struct )? TAG;
///
///     // Bit declarations. The name and the name slice are optional.
///     $( #[$ATTR] )* VIS bit NAME = INDEX, &[NAMES];
///     $( #[$ATTR] )* VIS bit NAME = INDEX;
///     $( #[$ATTR] )* VIS bit      = INDEX;
///
///     // Mask declarations. The display name is optional.
///     $( #[$ATTR] )* VIS mask NAME = [ A, B, C ], "NAME";
///     $( #[$ATTR] )* VIS mask NAME = EXPR, "NAME";
/// }
/// ```
///
/// where:
///
/// - `STORAGE` is a primitive integer type such as `u32`. It is required.
/// - `VIS` is an optional visibility modifier such as `pub` or `pub(crate)`.
/// - `TAG` is the tag type name. Writing `VIS struct TAG;` generates the tag
///   with standard derives. Writing just `TAG;` reuses an existing one.
/// - `INDEX` is the zero-based bit index.
/// - `&[NAMES]` is a slice of one or more names for the bit. The first is used
///   by [`Display`](core::fmt::Display). All are accepted when parsing.
/// - `bit NAME = INDEX;` declares a bit that has a named constant but no
///   display name.
/// - `bit = INDEX;` declares a valid bit with no constant and no name. It
///   contributes to [`VALID_BITS_MASK`] but not to [`FLAG_NAMES`].
/// - `[ A, B, C ]` computes a union of named bit constants into a multi-bit
///   value. The special syntax is required because the macro runs in const
///   context, where `BitFlag` does not implement `BitOr` in a way that can be
///   folded into a raw integer.
/// - `EXPR` is a bit-mask expression, for example `0` or `FIRST.mask_or_empty()
///   | SECOND.mask_or_empty()`.
/// - `#![name_matcher(...)]` sets [`BitFlagsSpec::DEFAULT_NAME_MATCHER`] and
///   defaults to [`NameMatcher::Exact`].
/// - `ATTR` is any outer attribute, including doc comments. Attributes are
///   copied to the generated item.
///
/// Indices that are not declared are invalid. They are excluded from
/// [`VALID_BITS_MASK`] and cleared by truncating operations.
///
/// # Tag declaration
///
/// The tag line may be written in two ways:
///
/// ```rust,ignore
/// // Reuse an existing tag type.
/// bitflags! {
///     #![storage(u32)]
///     SomeTag;
///     // ...
/// }
///
/// // Generate the tag type.
/// bitflags! {
///     #![storage(u32)]
///     pub struct SomeTag;
///     // ...
/// }
/// ```
///
/// Attributes on the tag declaration are copied to the generated struct.
///
/// # Generated items
///
/// The macro emits `impl BitFlagsSpec<STORAGE> for TAG`, plus one `const` per
/// named `bit` or `mask`.
/// The tag struct is emitted only when requested.
/// Attributes written on a declaration are copied to the corresponding
/// generated item.
///
/// The generated code is approximately equivalent to the following pattern:
///
/// ```rust,ignore
/// // Tag struct, if requested.
/// #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
/// pub struct TAG;
///
/// // Spec implementation for the tag.
/// impl BitFlagsSpec<STORAGE> for TAG {
///     const VALID_BITS_MASK: STORAGE = ...;
///     const FLAG_NAMES: &[&[&str]] = ...;
///     const COMBINED_NAMES: &[(STORAGE, &str)] = ...;
///     const DEFAULT_NAME_MATCHER: NameMatcher = ...;
/// }
///
/// // Bit constant, one per named `bit`.
/// pub const NAME: BitFlag<STORAGE, TAG> =
///     BitFlag::<STORAGE, TAG>::from_bit_retain(INDEX);
///
/// // Mask constant, one per named `mask`.
/// pub const NAME: BitFlags<STORAGE, TAG> =
///     BitFlags::<STORAGE, TAG>::from_bits_retain(VALUE);
/// ```
///
/// `FLAG_NAMES` is sized to the highest declared bit index.
/// Gaps are filled with empty slices, so unnamed and undeclared indices
/// coexist.
///
/// # Examples
///
/// Simple example with three consequent bits in bit flags.
///
/// ```rust
/// use bitflags_lite::{BitFlagsSpec, NameMatcher};
///
/// bitflags_lite::bitflags! {
///     #![storage(u32)]
///
///     pub struct PermissionsTag;
///
///     pub bit READ  = 0, &["read"];
///     pub bit WRITE = 1, &["write"];
///     pub bit EXEC  = 2, &["exec", "execute"];
///
///     pub mask RWX  = [ READ, WRITE, EXEC ], "all";
///     pub mask NONE = 0, "none";
/// }
///
/// assert_eq!(PermissionsTag::VALID_BITS_MASK, 0b111);
/// assert_eq!(PermissionsTag::DEFAULT_BITS, 0);
/// assert_eq!(
///     PermissionsTag::FLAG_NAMES,
///     &[&["read"][..], &["write"], &["exec", "execute"]]
/// );
/// assert_eq!(
///     PermissionsTag::COMBINED_NAMES,
///     &[(0b111, "all"), (0, "none")]
/// );
/// assert_eq!(PermissionsTag::DEFAULT_NAME_MATCHER, NameMatcher::Exact);
///
/// assert_eq!(READ.bit(), 0);
/// assert_eq!(WRITE.bit(), 1);
/// assert_eq!(EXEC.bit(), 2);
/// assert_eq!(RWX.bits(), 0b111);
/// assert_eq!(NONE.bits(), 0);
/// ```
///
/// <details>
/// <summary>Macro expansion (click to show)</summary>
///
/// ```rust
/// use bitflags_lite::{BitFlagsSpec, NameMatcher};
///
/// #[derive(
///     ::core::marker::Copy,
///     ::core::clone::Clone,
///     ::core::cmp::Eq,
///     ::core::cmp::PartialEq,
///     ::core::cmp::Ord,
///     ::core::cmp::PartialOrd,
///     ::core::hash::Hash,
///     ::core::default::Default,
///     ::core::fmt::Debug,
/// )]
/// pub struct PermissionsTag;
///
/// impl bitflags_lite::BitFlagsSpec<u32> for PermissionsTag {
///     const VALID_BITS_MASK: u32 =
///         const { (0 as u32) | (1 as u32) << 0 | (1 as u32) << 1 | (1 as u32) << 2 };
///     const DEFAULT_BITS: u32 = const { 0 };
///     const FLAG_NAMES: &[&[&str]] = const {
///         const MAX: usize = bitflags_lite::util::__const_max(&[0, 1, 2]);
///         const FLAG_NAMES: [&'static [&'static str]; MAX + 1] = const {
///             let mut flag_names: [&'static [&'static str]; MAX + 1] = [&[]; MAX + 1];
///             flag_names[0] = (&["read"]);
///             flag_names[1] = (&["write"]);
///             flag_names[2] = (&["exec", "execute"]);
///             flag_names
///         };
///         &FLAG_NAMES
///     };
///     const COMBINED_NAMES: &[(u32, &str)] = &[
///         (
///             (0 | bitflags_lite::BitFlag::<u32, PermissionsTag>::mask_or_empty(&READ)
///                 | bitflags_lite::BitFlag::<u32, PermissionsTag>::mask_or_empty(&WRITE)
///                 | bitflags_lite::BitFlag::<u32, PermissionsTag>::mask_or_empty(&EXEC)),
///             "all",
///         ),
///         (0, "none"),
///     ];
///     const DEFAULT_NAME_MATCHER: bitflags_lite::NameMatcher =
///         (bitflags_lite::NameMatcher::Exact);
/// }
///
/// pub const READ: bitflags_lite::BitFlag<u32, PermissionsTag> =
///     bitflags_lite::BitFlag::<u32, PermissionsTag>::from_bit_retain(0);
///
/// pub const WRITE: bitflags_lite::BitFlag<u32, PermissionsTag> =
///     bitflags_lite::BitFlag::<u32, PermissionsTag>::from_bit_retain(1);
///
/// pub const EXEC: bitflags_lite::BitFlag<u32, PermissionsTag> =
///     bitflags_lite::BitFlag::<u32, PermissionsTag>::from_bit_retain(2);
///
/// pub const RWX: bitflags_lite::BitFlags<u32, PermissionsTag> =
///     bitflags_lite::BitFlags::<u32, PermissionsTag>::from_bits_retain(
///         (0 | bitflags_lite::BitFlag::<u32, PermissionsTag>::mask_or_empty(&READ)
///             | bitflags_lite::BitFlag::<u32, PermissionsTag>::mask_or_empty(&WRITE)
///             | bitflags_lite::BitFlag::<u32, PermissionsTag>::mask_or_empty(&EXEC)),
///     );
///
/// pub const NONE: bitflags_lite::BitFlags<u32, PermissionsTag> =
///     bitflags_lite::BitFlags::<u32, PermissionsTag>::from_bits_retain(0);
/// ```
///
/// </details>
///
/// Example with invalid and unnamed but valid bits.
///
/// ```rust
/// use bitflags_lite::{BitFlagsSpec, NameMatcher};
///
/// bitflags_lite::bitflags! {
///     #![storage(u32)]
///     #![default([FIRST, TENTH])]
///     #![name_matcher(NameMatcher::AsciiCaseInsensitive)]
///
///     /// Flags tag doc-comment.
///     pub struct FlagsTag;
///
///     /// `FIRST` `BitFlag` doc-comment.
///     pub bit FIRST = 0, &["first", "1st"];
///
///     /// `THIRD` `BitFlag` doc-comment.
///     pub bit THIRD = 2, &["third", "3rd"];
///
///     pub bit       = 3;
///
///     /// `TENTH` `BitFlag` doc-comment.
///     pub bit TENTH = 9, &["tenth", "10th"];
///
///     /// `ALL` `BitFlags` doc-comment.
///     pub mask ALL     = [ FIRST, THIRD, TENTH ], "all";
///
///     /// `ALL_ALT` `BitFlags` doc-comment.
///     pub mask ALL_ALT = FIRST.mask_or_empty()
///                      | THIRD.mask_or_empty()
///                      | TENTH.mask_or_empty();
///
///     /// `NONE` `BitFlags` doc-comment.
///     pub mask NONE    = 0, "none";
/// }
///
/// assert_eq!(FlagsTag::VALID_BITS_MASK, 0b10_0000_1101);
/// assert_eq!(FlagsTag::DEFAULT_BITS, 0b10_0000_0001);
/// assert_eq!(
///     FlagsTag::FLAG_NAMES,
///     &[
///         &["first", "1st"][..],
///         &[],
///         &["third", "3rd"],
///         &[],
///         &[],
///         &[],
///         &[],
///         &[],
///         &[],
///         &["tenth", "10th"]
///     ]
/// );
/// assert_eq!(
///     FlagsTag::COMBINED_NAMES,
///     &[(0b10_0000_0101, "all"), (0, "none")]
/// );
/// assert_eq!(
///     FlagsTag::DEFAULT_NAME_MATCHER,
///     NameMatcher::AsciiCaseInsensitive
/// );
///
/// assert_eq!(FIRST.bit(), 0);
/// assert_eq!(THIRD.bit(), 2);
/// assert_eq!(TENTH.bit(), 9);
/// assert_eq!(ALL.bits(), 0b10_0000_0101);
/// assert_eq!(NONE.bits(), 0);
/// ```
///
/// <details>
/// <summary>Macro expansion (click to show)</summary>
///
/// ```rust
/// use bitflags_lite::{BitFlagsSpec, NameMatcher};
///
/// #[doc = r" Flags tag doc-comment."]
/// #[derive(
///     ::core::marker::Copy,
///     ::core::clone::Clone,
///     ::core::cmp::Eq,
///     ::core::cmp::PartialEq,
///     ::core::cmp::Ord,
///     ::core::cmp::PartialOrd,
///     ::core::hash::Hash,
///     ::core::default::Default,
///     ::core::fmt::Debug,
/// )]
/// pub struct FlagsTag;
///
/// impl bitflags_lite::BitFlagsSpec<u32> for FlagsTag {
///     const VALID_BITS_MASK: u32 = const {
///         (0 as u32) | (1 as u32) << 0 | (1 as u32) << 2 | (1 as u32) << 3 | (1 as u32) << 9
///     };
///     const DEFAULT_BITS: u32 = const {
///         (0 | bitflags_lite::BitFlag::<u32, FlagsTag>::mask_or_empty(&FIRST)
///             | bitflags_lite::BitFlag::<u32, FlagsTag>::mask_or_empty(&TENTH))
///     };
///     const FLAG_NAMES: &[&[&str]] = const {
///         const MAX: usize = bitflags_lite::util::__const_max(&[0, 2, 3, 9]);
///         const FLAG_NAMES: [&'static [&'static str]; MAX + 1] = const {
///             let mut flag_names: [&'static [&'static str]; MAX + 1] = [&[]; MAX + 1];
///             flag_names[0] = (&["first", "1st"]);
///             flag_names[2] = (&["third", "3rd"]);
///             flag_names[9] = (&["tenth", "10th"]);
///             flag_names
///         };
///         &FLAG_NAMES
///     };
///     const COMBINED_NAMES: &[(u32, &str)] = &[
///         (
///             (0 | bitflags_lite::BitFlag::<u32, FlagsTag>::mask_or_empty(&FIRST)
///                 | bitflags_lite::BitFlag::<u32, FlagsTag>::mask_or_empty(&THIRD)
///                 | bitflags_lite::BitFlag::<u32, FlagsTag>::mask_or_empty(&TENTH)),
///             "all",
///         ),
///         (0, "none"),
///     ];
///     const DEFAULT_NAME_MATCHER: bitflags_lite::NameMatcher =
///         (NameMatcher::AsciiCaseInsensitive);
/// }
///
/// #[doc = r" `FIRST` `BitFlag` doc-comment."]
/// pub const FIRST: bitflags_lite::BitFlag<u32, FlagsTag> =
///     bitflags_lite::BitFlag::<u32, FlagsTag>::from_bit_retain(0);
///
/// #[doc = r" `THIRD` `BitFlag` doc-comment."]
/// pub const THIRD: bitflags_lite::BitFlag<u32, FlagsTag> =
///     bitflags_lite::BitFlag::<u32, FlagsTag>::from_bit_retain(2);
///
/// #[doc = r" `TENTH` `BitFlag` doc-comment."]
/// pub const TENTH: bitflags_lite::BitFlag<u32, FlagsTag> =
///     bitflags_lite::BitFlag::<u32, FlagsTag>::from_bit_retain(9);
///
/// #[doc = r" `ALL` `BitFlags` doc-comment."]
/// pub const ALL: bitflags_lite::BitFlags<u32, FlagsTag> =
///     bitflags_lite::BitFlags::<u32, FlagsTag>::from_bits_retain(
///         (0 | bitflags_lite::BitFlag::<u32, FlagsTag>::mask_or_empty(&FIRST)
///             | bitflags_lite::BitFlag::<u32, FlagsTag>::mask_or_empty(&THIRD)
///             | bitflags_lite::BitFlag::<u32, FlagsTag>::mask_or_empty(&TENTH)),
///     );
///
/// #[doc = r" `ALL_ALT` `BitFlags` doc-comment."]
/// pub const ALL_ALT: bitflags_lite::BitFlags<u32, FlagsTag> =
///     bitflags_lite::BitFlags::<u32, FlagsTag>::from_bits_retain(
///         (FIRST.mask_or_empty() | THIRD.mask_or_empty() | TENTH.mask_or_empty()),
///     );
///
/// #[doc = r" `NONE` `BitFlags` doc-comment."]
/// pub const NONE: bitflags_lite::BitFlags<u32, FlagsTag> =
///     bitflags_lite::BitFlags::<u32, FlagsTag>::from_bits_retain(0);
/// ```
///
/// </details>
///
/// [`BitFlagsSpec`]: trait.BitFlagsSpec.html
/// [`BitFlagsSpec::VALID_BITS_MASK`]: trait.BitFlagsSpec.html#associatedconstant.VALID_BITS_MASK
/// [`BitFlagsSpec::FLAG_NAMES`]: trait.BitFlagsSpec.html#associatedconstant.FLAG_NAMES
/// [`BitFlagsSpec::DEFAULT_NAME_MATCHER`]: trait.BitFlagsSpec.html#associatedconstant.DEFAULT_NAME_MATCHER
/// [`VALID_BITS_MASK`]: trait.BitFlagsSpec.html#associatedconstant.VALID_BITS_MASK
/// [`FLAG_NAMES`]: trait.BitFlagsSpec.html#associatedconstant.FLAG_NAMES
/// [`NameMatcher::Exact`]: enum.NameMatcher.html#variant.Exact
#[macro_export]
macro_rules! bitflags {
    (
        $( #![ $($bitflags_meta:tt)* ] )*
        $tag_name:ident;
        $( $tt:tt )*
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] [ $( #![ $( $bitflags_meta )* ] )* ]
            [] [] [] [] [ $tag_name ] [] [] []
        }
    };
    (
        $( #![ $($bitflags_meta:tt)* ] )*
        $( #[ $($tag_meta:tt)* ] )*
        $tag_vis:vis struct $tag_name:ident;
        $( $tt:tt )*
    ) => {
        $( #[ $($tag_meta)* ] )*
        #[derive(
            ::core::marker::Copy,
            ::core::clone::Clone,
            ::core::cmp::Eq,
            ::core::cmp::PartialEq,
            ::core::cmp::Ord,
            ::core::cmp::PartialOrd,
            ::core::hash::Hash,
            ::core::default::Default,
            ::core::fmt::Debug,
        )]
        $tag_vis struct $tag_name;

        $crate::__bitflags_impl! {
            [ $( $tt )* ] [ $( #![ $( $bitflags_meta )* ] )* ]
            [] [] [] [] [ $tag_name ] [] [] []
        }
    };
}

/// # Syntax
///
/// ```rust,ignore
/// __bitflags_impl {
///     // Unparsed meta.
///     [ $( #![name_matcher( $name_matcher:expr )] )? ]
///
///     // Unparsed bits and masks.
///     [ $( $vis:vis $name:ident: $kind:tt = $value:expr $(, $display:expr )?; )* ]
///
///     // Parsed bits.
///     [ $( $value:expr $(, $display:expr )?; )* ]
///
///     // Parsed named bits.
///     [ $( $vis:vis $name:ident = $value:expr $(, $display:expr )?; )* ]
///
///     // Parsed masks.
///     [ $( $value:expr $(, $display:expr )?; )* ]
///
///     // Parsed named masks.
///     [ $( $vis:vis $name:ident = $value:expr $(, $display:expr )?; )* ]
///
///     // Tag typename.
///     [ $tag:ty ]
///
///     // Storage inner typename.
///     [ $storage:ty ]
///
///     // Default bits value.
///     [ $default:ty ]
///
///     // Parsed name matcher.
///     [ $( $name_matcher:expr )? ]
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __bitflags_impl {
    // Choose defaults.
    (
        [ $( $tt:tt )* ] []
        [] [] [] [] [ $tag:ty ] []
        [ $( $default:expr )? ] [ $( $name_matcher:expr )? ]
    ) => {
        ::core::compile_error!("`#![storage(...)]` attribute is required");
    };
    (
        [ $( $tt:tt )* ] []
        [] [] [] [] [ $tag:ty ] [ $storage:ty ] [] [ $( $name_matcher:expr )? ]
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] [] [] [] [] []
            [ $tag ] [ $storage ] [ 0 ] [ $( $name_matcher )? ]
        }
    };
    (
        [ $( $tt:tt )* ] []
        [] [] [] [] [ $tag:ty ] [ $storage:ty ] [ $( $default:expr )? ] []
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] [] [] [] [] []
            [ $tag ] [ $storage ] [ $( $default )? ] [ $crate::NameMatcher::Exact ]
        }
    };

    // Choose storage.
    (
        [ $( $tt:tt )* ]
        [ #![ storage( $storage:ty ) ] $( #![ $($meta:tt)* ] )* ]
        [] [] [] [] [ $tag:ty ] []
        [ $( $default:expr )? ] [ $( $name_matcher:expr )? ]
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] [ $( #![ $($meta)* ] )* ]
            [] [] [] [] [ $tag ] [ $storage ]
            [ $( $default )? ] [ $( $name_matcher )? ]
        }
    };

    // Choose default.
    (
        [ $( $tt:tt )* ]
        [ #![ default( [ $( $value:ident ),* $(,)? ] ) ] $( #![ $($meta:tt)* ] )* ]
        [] [] [] [] [ $tag:ty ] [ $storage:ty ]
        [] [ $( $name_matcher:expr )? ]
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] [ $( #![ $($meta)* ] )* ]
            [] [] [] [] [ $tag ] [ $storage ]
            [ 0 $( |
                $crate::BitFlag::<$storage, $tag>::mask_or_empty(&$value)
            )* ] [ $( $name_matcher )? ]
        }
    };
    (
        [ $( $tt:tt )* ]
        [ #![ default( $default:expr ) ] $( #![ $($meta:tt)* ] )* ]
        [] [] [] [] [ $tag:ty ] [ $( $storage:ty )? ]
        [] [ $( $name_matcher:expr )? ]
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] [ $( #![ $($meta)* ] )* ]
            [] [] [] [] [ $tag ] [ $( $storage )? ]
            [ $default ] [ $( $name_matcher )? ]
        }
    };

    // Choose name matcher.
    (
        [ $( $tt:tt )* ]
        [ #![ name_matcher( $name_matcher:expr ) ] $( #![ $($meta:tt)* ] )* ]
        [] [] [] [] [ $tag:ty ] [ $( $storage:ty )? ] [ $( $default:expr )? ] []
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] [ $( #![ $($meta)* ] )* ]
            [] [] [] [] [ $tag ] [ $( $storage )? ]
            [ $( $default )? ] [ $name_matcher ]
        }
    };

    // Parse unnamed bit item.
    (
        [
            $vis:vis bit = $value:expr $(, $display:expr )?;
            $( $tt:tt )*
        ]
        []
        [ $( $bits:tt )* ] [ $( $named_bits:tt )* ]
        [ $( $masks:tt )* ] [ $( $named_masks:tt )* ]
        [ $tag:ty ] [ $storage:ty ] [ $default:expr ] [ $name_matcher:expr ]
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] []
            [ $( $bits )* [ [ $value ] [ $( $display )? ] ] ]
            [ $( $named_bits )* ]
            [ $( $masks )* ]
            [ $( $named_masks )* ]
            [ $tag ] [ $storage ] [ $default ] [ $name_matcher ]
        }
    };

    // Parse named bit item.
    (
        [
            $( #[ $($meta:meta)* ] )*
            $vis:vis bit $name:ident = $value:expr $(, $display:expr )?;
            $( $tt:tt )*
        ]
        []
        [ $( $bits:tt )* ] [ $( $named_bits:tt )* ]
        [ $( $masks:tt )* ] [ $( $named_masks:tt )* ]
        [ $tag:ty ] [ $storage:ty ] [ $default:expr ] [ $name_matcher:expr ]
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] []
            [ $( $bits )* [ [ $value ] [ $( $display )? ] ] ]
            [ $( $named_bits )* [
                [ $( #[ $($meta)* ] )* ] [ $vis ] [ $name ] [ $value ]
            ] ]
            [ $( $masks )* ]
            [ $( $named_masks )* ]
            [ $tag ] [ $storage ] [ $default ] [ $name_matcher ]
        }
    };

    // Parse mask item with union array syntax.
    (
        [
            $( #[ $($meta:meta)* ] )*
            $vis:vis mask $( $name:ident )? =
                [ $( $value:ident ),* $(,)? ] $(, $display:expr )?;
            $( $tt:tt )*
        ]
        []
        [ $( $bits:tt )* ] [ $( $named_bits:tt )* ]
        [ $( $masks:tt )* ] [ $( $named_masks:tt )* ]
        [ $tag:ty ] [ $storage:ty ] [ $default:expr ] [ $name_matcher:expr ]
    ) => {
        $crate::__bitflags_impl! {
            [
                $( #[ $($meta)* ] )*
                $vis mask $( $name )? =
                    0 $( |
                        $crate::BitFlag::<$storage, $tag>::mask_or_empty(&$value)
                    )*
                 $(, $display )?;
                $( $tt )*
            ]
            []
            [ $( $bits )* ] [ $( $named_bits )* ]
            [ $( $masks )* ] [ $( $named_masks )* ]
            [ $tag ] [ $storage ] [ $default ] [ $name_matcher ]
        }
    };

    // Parse unnamed mask item.
    (
        [
            $( #[ $($meta:meta)* ] )*
            $vis:vis mask = $value:expr $(, $display:expr )?;
            $( $tt:tt )*
        ]
        []
        [ $( $bits:tt )* ] [ $( $named_bits:tt )* ]
        [ $( $masks:tt )* ] [ $( $named_masks:tt )* ]
        [ $tag:ty ] [ $storage:ty ] [ $default:expr ] [ $name_matcher:expr ]
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] []
            [ $( $bits )* ]
            [ $( $named_bits )* ]
            [ $( $masks )* [ [ $value ] [ $( $display )? ] ] ]
            [ $( $named_masks )* ]
            [ $tag ] [ $storage ] [ $default ] [ $name_matcher ]
        }
    };

    // Parse named mask item.
    (
        [
            $( #[ $($meta:meta)* ] )*
            $vis:vis mask $name:ident = $value:expr $(, $display:expr )?;
            $( $tt:tt )*
        ]
        []
        [ $( $bits:tt )* ] [ $( $named_bits:tt )* ]
        [ $( $masks:tt )* ] [ $( $named_masks:tt )* ]
        [ $tag:ty ] [ $storage:ty ] [ $default:expr ] [ $name_matcher:expr ]
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] []
            [ $( $bits )* ]
            [ $( $named_bits )* ]
            [ $( $masks )* [ [ $value ] [ $( $display )? ] ] ]
            [ $( $named_masks )* [
                [ $( #[ $($meta)* ] )* ] [ $vis ] [ $name ] [ $value ]
            ] ]
            [ $tag ] [ $storage ] [ $default ] [ $name_matcher ]
        }
    };

    (
        [] []
        [ $( [
            [ $b_value:expr ] [ $( $b_display:expr )? ]
        ] )* ]
        [ $( [
            [ $( #[ $($nb_meta:meta)* ] )* ]
            [ $nb_vis:vis ] [ $nb_name:ident ] [ $nb_value:expr ]
        ] )* ]
        [ $( [
            [ $m_value:expr ] [ $( $m_display:expr )? ]
        ] )* ]
        [ $( [
            [ $( #[ $($nm_meta:meta)* ] )* ]
            [ $nm_vis:vis ] [ $nm_name:ident ] [ $nm_value:expr ]
        ] )* ]
        [ $tag:ty ] [ $storage:ty ] [ $default:expr ] [ $name_matcher:expr ]
    ) => {
        #[allow(
            trivial_numeric_casts,
            unused_mut,
            clippy::allow_attributes,
            clippy::as_conversions,
            reason = "codegen"
        )]
        impl $crate::BitFlagsSpec<$storage> for $tag {
            const VALID_BITS_MASK: $storage = const {
                (0 as $storage) $( | (1 as $storage) << $b_value )*
            };
            const DEFAULT_BITS: $storage = const { $default };
            const FLAG_NAMES: &[&[&str]] = const {
                const MAX: usize = $crate::util::__const_max(&[ $( $b_value ),*]);
                const FLAG_NAMES: [&'static [&'static str]; MAX + 1] = const {
                    let mut flag_names: [&'static [&'static str]; MAX + 1] = [&[]; MAX + 1];
                    $( $( flag_names[ $b_value ] = $b_display; )? )*
                    flag_names
                };
                &FLAG_NAMES
            };
            const COMBINED_NAMES: &[($storage, &str)] = &[
                $( $( ( $m_value, $m_display ), )? )*
            ];
            const DEFAULT_NAME_MATCHER: $crate::NameMatcher = $name_matcher;
        }

        $(
            $( #[ $($nb_meta)* ] )*
            $nb_vis const $nb_name: $crate::BitFlag<$storage, $tag> =
                $crate::BitFlag::<$storage, $tag>::from_bit_retain($nb_value);
        )*
        $(
            $( #[ $($nm_meta)* ] )*
            $nm_vis const $nm_name: $crate::BitFlags<$storage, $tag> =
                $crate::BitFlags::<$storage, $tag>::from_bits_retain($nm_value);
        )*
    };

}
