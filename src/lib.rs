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
    ($err:expr)   => (return ::core::result::Result::Err(::core::convert::From::from($err)))
}
