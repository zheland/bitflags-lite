//! Tests for `util.rs` file.

#![allow(unused_crate_dependencies, reason = "okay in tests")]

use bitflags_lite::util;

#[test]
fn test_const_max() {
    assert_eq!(util::__const_max(&[1, 5, 3]), 5);
    assert_eq!(util::__const_max(&[9, 5, 1]), 9);
}
