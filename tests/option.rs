use fehler::*;

#[throws(as Option)]
fn foo(x: bool) -> i32 {
    if x { throw!(); }
    0
}

#[test]
fn test_outcome_true() {
    assert!(foo(true).is_none())
}

#[test]
fn test_outcome_false() {
    assert_eq!(Some(0), foo(false))
}
