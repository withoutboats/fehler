#![feature(backtrace)]

use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{self, Display};
use std::io;

macro_rules! null_display {
    ($t:ty) => { impl Display for $t {
        fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result { Ok(()) }
    } };
}

fn is_error<T: Error>(t: T) -> T { t }

#[derive(fehler::Error, Debug)]
struct UnitError;
null_display!(UnitError);

#[test]
fn unit_error() {
    let e = is_error(UnitError);
    assert!(e.backtrace().is_none());
    assert!(e.source().is_none());
}

#[derive(fehler::Error, Debug)]
struct StructError {
    x: i32,
}
null_display!(StructError);

#[test]
fn struct_error() {
    let e = is_error(StructError { x: 0 });
    assert!(e.backtrace().is_none());
    assert!(e.source().is_none());
}

#[derive(fehler::Error, Debug)]
struct BacktraceError {
    b: Backtrace,
}
null_display!(BacktraceError);

#[test]
fn backtrace_error() {
    let e = is_error(BacktraceError { b: Backtrace::capture() });
    assert!(e.backtrace().is_some());
    assert!(e.source().is_none());
}

#[derive(fehler::Error, Debug)]
struct SourceError {
    #[error(source)]
    e: io::Error,
}
null_display!(SourceError);

#[test]
fn source_error() {
    let e = is_error(SourceError { e: io::Error::last_os_error() });
    assert!(e.backtrace().is_none());
    assert!(e.source().is_some());
}
