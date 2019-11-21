# Der Fehler

Der Fehler is a library to add support for "throwing functions" to Rust through
procedural macros. Functions marked with the `throws` attribute return
`Result`, but the "Ok" path is used by default and you don't need to wrap ok
return values in `Ok`. To throw errors, use `?` or the `throws` macro.

Der Fehler provides these items:

### The `#[throws]` attribute

The throws attribute modifies a function or method to make it return a
`Result`. It takes an optional typename as an argument to the attribute which
will be the error type of this function; if no typename is supplied, it uses
the default error type for this crate.

Within the function body, `return`s (including the implicit final return) are
automatically "Ok-wrapped." To raise errors, use `?` or the `throws!` macro.

For example, these two functions are equivalent:

```rust
#[throws(i32)]
fn foo(x: bool) -> i32 {
    if x {
        0
    } else {
        throw!(1);
    }
}

fn bar(x: bool) -> Result<i32, i32> {
    if x {
        Ok(0)
    } else {
        Err(1)
    }
}
```

### The `throw!` macro

`throw!` is a macro which is equivalent to the `Err($e)?` pattern. It takes an
error type and "throws" it.

One important aspect of the `throw!` macro is that it allows you to return
errors inside of functions marked with `throws`. You cannot just `return`
errors from these functions, you need to use this macro.

# TODO

* Make throws work on closures and async blocks (attributes are not allowed on
  expressions on stable)

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
