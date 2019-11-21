#![no_std]

use fehler::{throw, throws};

#[throws(i32)]
fn no_std_fn() {
    throw!(0);
}

#[test]
fn test_expansion() {
    assert_eq!(Err(0), no_std_fn());
}
