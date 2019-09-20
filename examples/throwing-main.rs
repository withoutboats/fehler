use fehler::*;

#[throws]
fn do_it() -> i32 {
    throw!(error!("oops, an error occurred"));
    0
}

#[throws]
fn main() {
    do_it().context("do it failed")?;
}
