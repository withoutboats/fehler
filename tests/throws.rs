use fehler::*;

type Error = ();

#[throws(_)]
pub fn unit_fn() { }

#[throws(_)]
pub fn returns_fn() -> i32 {
    return 0;
}

#[throws(_)]
pub fn returns_unit_fn() {
    if true { return; }
}

#[throws(_)]
pub fn explicit_unit() -> () { }

#[throws(_)]
pub fn tail_returns_value() -> i32 {
    0
}

#[throws(_)]
pub async fn async_fn() { }

#[throws(_)]
pub async fn async_fn_with_ret() -> i32 {
    0
}

#[throws(i32)]
pub fn throws_error() {
    if true { throw!(0); }
}

#[throws(i32)]
pub fn throws_and_has_return_type() -> &'static str {
    if true {
        return "success";
    } else if false {
        throw!(0);
    }
    "okay"
}

#[throws(E)]
pub fn throws_generics<E>() { }

pub struct Foo;

impl Foo {
    #[throws(_)]
    pub fn static_method() { }

    #[throws(_)]
    pub fn bar(&self) -> i32 { if true { return 1; } 0 }
}


#[throws(_)]
pub fn has_inner_fn() {
    fn inner_fn() -> i32 { 0 }
    let _: i32 = inner_fn();
}

#[throws(_)]
pub fn has_inner_closure() {
    let f = || 0;
    let _: i32 = f();
}

#[throws(_)]
pub async fn has_inner_async_block() {
    let f = async { 0 };
    let _: i32 = f.await;
}
