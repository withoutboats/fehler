#![no_std]
#![cfg_attr(feature = "nightly", feature(try_trait))]
#[doc(inline)]
/// Annotations a function that "throws" a Result.
///
/// Inside functions tagged with `throws`, you can use `?` and the `throw!` macro to return errors,
/// but you don't need to wrap the successful return values in `Ok`.
///
/// `throws` can optionally take a type as an argument, which will be the error type returned by
/// this function. By default, the function will throw this crate's "default error type." (see
/// below).
///
/// # Default Error Type
///
/// This macro supports a "default error type," if you give the macro `_` instead of a type name.
/// The default error type will be whatever the path `crate::Error` resolves to: so if you have
/// a type called `Error` in your crate root, that is the type the macro will use by default.
///
/// You can define your own error in your crate root, or you can use a type alias.
///
/// # Example
///
/// ```should_panic
/// // Set the default error type for this crate:
/// type Error = std::io::Error;
///
/// #[fehler::throws(_)]
/// fn main() {
///    let file = std::fs::read_to_string("my_file.txt")?;
///    println!("{}", file);
/// }
/// ```
pub use fehler_macros::throws;

/// Throw an error.
///
/// This macro is equivalent to `Err($err)?`.
#[macro_export]
macro_rules! throw {
    ($err:expr)   => (return <_ as $crate::__internal::_Throw>::from_error((::core::convert::From::from($err))));
    ()            => (return <_ as ::core::default::Default>::default());
}

#[doc(hidden)]
pub mod __internal {
    pub trait _Succeed {
        type Ok;
        fn from_ok(ok: Self::Ok) -> Self;
    }

    pub trait _Throw {
        type Error;
        fn from_error(error: Self::Error) -> Self;
    }

    #[cfg(not(feature = "nightly"))]
    mod stable {
        use core::task::Poll;

        impl<T, E> super::_Succeed for Result<T, E> {
            type Ok = T;
            fn from_ok(ok: T) -> Self {
                Ok(ok)
            }
        }

        impl<T, E> super::_Throw for Result<T, E> {
            type Error = E;
            fn from_error(error: Self::Error) -> Self {
                Err(error)
            }
        }

        impl<T, E> super::_Succeed for Poll<Result<T, E>> {
            type Ok = Poll<T>;

            fn from_ok(ok: Self::Ok) -> Self {
                match ok {
                    Poll::Ready(ok) => Poll::Ready(Ok(ok)),
                    Poll::Pending   => Poll::Pending,
                }
            }
        }

        impl<T, E> super::_Throw for Poll<Result<T, E>> {
            type Error = E;

            fn from_error(error: Self::Error) -> Self {
                Poll::Ready(Err(error))
            }
        }

        impl<T, E> super::_Succeed for Poll<Option<Result<T, E>>> {
            type Ok = Poll<Option<T>>;

            fn from_ok(ok: Self::Ok) -> Self {
                match ok {
                    Poll::Ready(Some(ok))   => Poll::Ready(Some(Ok(ok))),
                    Poll::Ready(None)       => Poll::Ready(None),
                    Poll::Pending           => Poll::Pending,
                }
            }
        }

        impl<T, E> super::_Throw for Poll<Option<Result<T, E>>> {
            type Error = E;

            fn from_error(error: Self::Error) -> Self {
                Poll::Ready(Some(Err(error)))
            }
        }

        impl<T> super::_Succeed for Option<T> {
            type Ok = T;

            fn from_ok(ok: Self::Ok) -> Self {
                Some(ok)
            }
        }
    }

    #[cfg(feature = "nightly")]
    mod nightly {
        use core::ops::Try;

        impl<T> super::_Succeed for T where T: Try {
            type Ok = <T as Try>::Ok;

            fn from_ok(ok: Self::Ok) -> Self {
                <T as Try>::from_ok(ok)
            }
        }

        impl<T> super::_Throw for T where T: Try {
            type Error = <T as Try>::Error;

            fn from_error(error: Self::Error) -> Self {
                <T as Try>::from_error(error)
            }
        }
    }
}
