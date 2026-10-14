//! Lightweight generics-based bitflags.
//!
//! This crate provides [`BitFlags`], a `#[repr(transparent)]` wrapper around an
//! unsigned integer that adds a compile-time tag type. The tag type
//! distinguishes flag sets that share the same storage type and defines which
//! bits are valid via the [`BitFlagsSpec`] trait.
//!
//! The crate is `no_std` and `no_alloc`. Optional `serde` and `bytemuck`
//! support can be enabled through features.

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
