#![feature(generator_trait)]

pub mod __internal {
    use std::ops::Generator;
    use std::pin::Pin;
    use std::task::*;

    use futures_core::Stream;

    pub struct GenIter<T>(pub T);

    impl<T: Generator> Iterator for GenIter<T> {
        type Item = T::Yield;

        fn next(&mut self) -> Option<Self::Item> {
            panic!()
        }
    }

    pub struct GenStream<T>(T);

    impl<T> GenStream<T> {
        pub unsafe fn new(val: T) -> GenStream<T> {
            GenStream(val)
        }
    }

    impl<G: Generator<*mut (), Yield = Poll<T>, Return = ()>, T> Stream for GenStream<G> {
        type Item = T;

        fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            panic!()
        }
    }
}

#[macro_export]
macro_rules! async_gen_yield {
    ($e:expr) => { $e }
}
