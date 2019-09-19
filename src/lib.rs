#![feature(backtrace)]

mod as_error;
mod exception;
mod context;

#[doc(inline)]
pub use fehler_macros::{throws, Error};

pub use crate::as_error::AsError;
pub use crate::exception::{Exception, Errors};
pub use crate::context::ResultExt;

#[macro_export]
macro_rules! throw {
    ($err:expr)   => (return Err(::std::convert::From::from($err)));
}

#[macro_export]
macro_rules! err {
    ($e:expr)   => { $crate::Exception::new_adhoc($e) };
    ($($arg:tt)*) => { $crate::Exception::new_adhoc(format!($($arg)*)) };
}
