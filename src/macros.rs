#[macro_export]
macro_rules! bitflags {
    (
        $( #[ $($meta:tt)* ] )*
        BitFlags<$inner:ty, $tag_name:ident>;
        $( $tt:tt )*
    ) => {
        $crate::__bitflags_impl! {
            [ $( $tt )* ] [ $( #[ $( $meta )* ] )* ]
            [] [] [ $inner ] [ $tag_name ] []
        }
    };
    (
        $( #[ $($meta:tt)* ] )*
        BitFlags<$inner:ty, $tag_vis:vis struct $tag_name:ident>;
        $( $tt:tt )*
    ) => {
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

        $crate::bitflags! {
            $( #[ $($meta)* ] )*
            BitFlags<$inner, $tag_name>;
            $( $tt )*
        }
    };
    (
        $( #[ $($meta:tt)* ] )*
        $flags_vis:vis type $flags_name:ident = BitFlags<$inner:ty, $tag_name:ident>;
        $( $tt:tt )*
    ) => {
        type $flags_name = $crate::BitFlags<$inner, $tag_name>;

        $crate::bitflags! {
            $( #[ $($meta)* ] )*
            BitFlags<$inner, $tag_name>;
            $( $tt )*
        }
    };
    (
        $( #[ $($meta:tt)* ] )*
        $flags_vis:vis type $flags_name:ident =
            BitFlags<$inner:ty, $tag_vis:vis struct $tag_name:ident>;
        $( $tt:tt )*
    ) => {
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
        type $flags_name = $crate::BitFlags<$inner, $tag_name>;

        $crate::bitflags! {
            $( #[ $($meta)* ] )*
            BitFlags<$inner, $tag_name>;
            $( $tt )*
        }
    };
}

/// # Syntax
///
/// ```rust
/// __bitflags_impl {
///     // Unparsed meta.
///     [ $( #[name_matcher( $name_matcher:expr )] )? ]
///
///     // Unparsed bits and masks.
///     [ $( $vis:vis $name:ident: $kind:tt = $value:expr $(, $display:expr )?; )* ]
///
///     // Parsed bits.
///     [ $( $vis:vis $name:ident = $value:expr $(, $display:expr )?; )* ]
///
///     // Parsed masks.
///     [ $( $vis:vis $name:ident = $value:expr $(, $display:expr )?; )* ]
///
///     // Storage inner typename.
///     [ $inner:ty ]
///
///     // Tag typename.
///     [ $tag:ty ]
///
///     // Parsed name matcher.
///     [ $( $name_matcher:expr )? ]
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __bitflags_impl {
    // Choose default name matcher.
    (
        [ $($tt:tt)* ] []
        [] [] [ $inner:ty ] [ $tag:ty ] []
    ) => {
        $crate::__bitflags_impl! {
            [ $($tt)* ] [] [] [] [ $inner ] [ $tag ] [ $crate::NameMatcher::Exact ]
        }
    };

    // Choose given name matcher.
    (
        [ $($tt:tt)* ] [ #[ name_matcher( $name_matcher:expr ) ] ]
        [] [] [ $inner:ty ] [ $tag:ty ] []
    ) => {
        $crate::__bitflags_impl! {
            [ $($tt)* ] [] [] [] [ $inner ] [ $tag ] [ $name_matcher ]
        }
    };

    // Parse bit entry.
    (
        [
            $vis:vis bit $( $name:ident )? = $value:expr $(, $display:expr )?;
            $($tt:tt)*
        ]
        [] [ $( $bits:tt )* ] [ $( $masks:tt )* ]
        [ $inner:ty ] [ $tag:ty ] [ $name_matcher:expr ]
    ) => {
        $crate::__bitflags_impl! {
            [ $($tt)* ] []
            [ $($bits)* [ [ $vis ] [ $( $name )? ] [ $value ] [ $( $display )? ] ] ]
            [ $($masks)* ]
            [ $inner ] [ $tag ] [ $name_matcher ]
        }
    };

    // Parse mask entry.
    (
        [
            $vis:vis mask $( $name:ident )? = mask $( $value:ident )|* $(, $display:expr )?;
            $($tt:tt)*
        ]
        [] [ $( $bits:tt )* ] [ $( $masks:tt )* ]
        [ $inner:ty ] [ $tag:ty ] [ $name_matcher:expr ]
    ) => {
        $crate::__bitflags_impl! {
            [ $($tt)* ] []
            [ $($bits)* ]
            [ $($masks)* [ [ $vis ] [ $( $name )? ]
            [ 0 $(
                | $crate::BitFlag::<$inner, $tag>::__mask_or_zero(&$value)
            )* ] [ $( $display )? ] ] ]
            [ $inner ] [ $tag ] [ $name_matcher ]
        }
    };
    (
        [
            $vis:vis mask $( $name:ident )? = $value:expr $(, $display:expr )?;
            $($tt:tt)*
        ]
        [] [ $( $bits:tt )* ] [ $( $masks:tt )* ]
        [ $inner:ty ] [ $tag:ty ] [ $name_matcher:expr ]
    ) => {
        $crate::__bitflags_impl! {
            [ $($tt)* ] []
            [ $($bits)* ]
            [ $($masks)* [ [ $vis ] [ $( $name )? ] [ $value ] [ $( $display )? ] ] ]
            [ $inner ] [ $tag ] [ $name_matcher ]
        }
    };

    (
        [] []
        [ $( [ [ $b_vis:vis ] [ $( $b_name:ident )? ] [ $b_value:expr ] [ $( $b_display:expr )? ] ] )* ]
        [ $( [ [ $m_vis:vis ] [ $( $m_name:ident )? ] [ $m_value:expr ] [ $( $m_display:expr )? ] ] )* ]
        [ $inner:ty ] [ $tag:ty ] [ $name_matcher:expr ]
    ) => {
        #[allow(
            trivial_numeric_casts,
            unused_mut,
            clippy::allow_attributes,
            clippy::as_conversions,
            reason = "codegen"
        )]
        impl $crate::BitFlagsSpec<$inner> for $tag {
            const VALID_BITS_MASK: $inner = const {
                (0 as $inner) $( | (1 as $inner) << $b_value )*
            };
            const FLAG_NAMES: &[&[&str]] = const {
                const MAX: usize = $crate::util::__const_max(&[ $( $b_value ),*]);
                const FLAG_NAMES: [&'static [&'static str]; MAX + 1] = const {
                    let mut flag_names: [&'static [&'static str]; MAX + 1] = [&[]; MAX + 1];
                    $( $( flag_names[ $b_value ] = $b_display; )? )*
                    flag_names
                };
                &FLAG_NAMES
            };
            const COMBINED_NAMES: &[($inner, &str)] = &[
                $( $( ( $m_value, $m_display ), )? )*
            ];
            const DEFAULT_NAME_MATCHER: $crate::NameMatcher = $name_matcher;
        }

        $( $(
            $b_vis const $b_name: $crate::BitFlag<$inner, $tag> =
                $crate::BitFlag::<$inner, $tag>::from_bit_retain($b_value);
        )? )*
        $( $(
            $m_vis const $m_name: $crate::BitFlags<$inner, $tag> =
                $crate::BitFlags::<$inner, $tag>::from_bits_retain($m_value);
        )? )*
    };

}
