use fehler::throw;

#[test]
fn throw_works() {
    fn foo() -> Result<(), i32> { throw!(0) }
    assert_eq!(foo(), Err(0));
}

#[test]
fn throw_infers_ok_type() {
    fn foo(b: bool) -> Result<i32, ()> {
        let x = if b {
            0
        } else {
            throw!(());
        };
        Ok(x)
    }

    assert_eq!(foo(true), Ok(0));
    assert_eq!(foo(false), Err(()));
}
