#![feature(generators, generator_trait, try_trait)]

use std::io;

#[fehler::throws(@__internal_propane_integration io::Error)]
fn simple() -> (impl Iterator<Item = i32>) {
    let __ret = || {
        for x in 0..10 {
            yield x;
        }
    };

    ::propane::__internal::GenIter(__ret)
}

#[fehler::throws(@__internal_propane_integration ())]
fn async_gen() -> (impl futures_core::Stream<Item = i32>) {
    let __ret = |_| {
        for x in 0..10 {
            propane::async_gen_yield!(x);
        }
    };

    unsafe { ::propane::__internal::GenStream::new(__ret) }
}
