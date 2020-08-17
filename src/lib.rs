#![no_std]
#[doc(inline)]
/// Annotations a function that "throws" a Result.
///
/// Inside functions tagged with `throws`, you can use `?` and the `throw!` macro to return errors,
/// but you don't need to wrap the successful return values in `Ok`.
///
/// Using this syntax, you can write fallible functions almost as if they were nonfallible. Every
/// time a function call would return a `Result`, you "re-raise" the error using `?`, and if you
/// wish to raise your own error, you can return it with the `throw!` macro.
///
/// ## Example
/// ```should_panic
/// use std::io::{self, Read};
///
/// use fehler::{throw, throws};
///
/// #[throws(io::Error)]
/// fn main() {
///     let mut file = std::fs::File::open("The_House_of_the_Spirits.txt")?;
///     let mut text = String::new();
///     file.read_to_string(&mut text)?;
///
///     if !text.starts_with("Barrabas came to us by sea, the child Clara wrote") {
///         throw!(io::Error::from_raw_os_error(22));
///     }
///
///     println!("Okay!");
/// }
/// ```
///
/// # Default Error Type
///
/// This macro supports a "default error type" - if you do not pass a type to the macro, it will
/// use the type named `Error` in this scope. So if you have defined an error type in this
/// module, that will be the error thrown by this function.
///
/// You can access this feature by omitting the arguments entirely or by passing `_` as the type.
///
/// ## Example
///
/// ```should_panic
/// use fehler::throws;
///
/// // Set the default error type for this module:
/// type Error = std::io::Error;
///
/// #[throws]
/// fn main() {
///    let file = std::fs::read_to_string("my_file.txt")?;
///    println!("{}", file);
/// }
/// ```
///
/// # Throwing as an Option
///
/// This syntax can also support functions which return an `Option` instead of a `Result`. The
/// way to access this is to pass `as Option` as the argument to `throw`.
///
/// In functions that return `Option`, you can use the `throw!()` macro without any argument to
/// return `None`.
///
/// ## Example
///
/// ```
/// use fehler::{throw, throws};
///
/// #[throws(as Option)]
/// fn example<T: Eq + Ord>(slice: &[T], needle: &T) -> usize {
///     if !slice.contains(needle) {
///         throw!();
///     }
///     slice.binary_search(needle).ok()?
/// }
/// ```
///
/// # Other `Try` types
///
/// The `?` syntax in Rust is controlled by a trait called `Try`, which is currently unstable.
/// Because this feature is unstable and I don't want to maintain compatibility if its interface
/// changes, this crate currently only works with two stable `Try` types: Result and Option.
/// However, its designed so that it will hopefully support other `Try` types as well in the
/// future.
///
/// It's worth noting that `Try` also has some other stable implementations: specifically `Poll`.
/// Because of the somewhat unusual implementation of `Try` for those types, this crate does not
/// support `throws` syntax on functions that return `Poll` (so you can't use this syntax when
/// implementing a Future by hand, for example). I hope to come up with a way to support Poll in
/// the future.
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
#[macro_export]
macro_rules! gen_throw {
    ($err:expr)   => (yield <_ as $crate::__internal::_Throw>::from_error((::core::convert::From::from($err))));
    ()            => (yield <_ as ::core::default::Default>::default());
}

#[doc(hidden)]
#[macro_export]
macro_rules! async_gen_throw {
    ($err:expr)   => (yield core::Poll::Ready(<_ as $crate::__internal::_Throw>::from_error((::core::convert::From::from($err)))));
    ()            => (yield core::Poll::Ready(<_ as ::core::default::Default>::default()));
}

#[doc(hidden)]
#[macro_export]
macro_rules! async_gen_yield_fehler {
    ($e:expr) => {{
        yield core::task::Poll::Ready(<_ as ::fehler::__internal::_Succeed>::from_ok($e))
    }}
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
}
