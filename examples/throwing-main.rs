use fehler::*;

#[derive(Debug)]
struct Error;

#[throws(_)]
fn do_it() -> i32 {
    if true {
        throw!(Error);
    }

    0
}

#[throws(_)]
fn main() {
    do_it()?;
}
