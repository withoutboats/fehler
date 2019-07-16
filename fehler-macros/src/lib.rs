extern crate proc_macro;

mod error;
mod throws;

use proc_macro::*;

#[proc_macro_derive(Error, attributes(error))]
pub fn error(item: TokenStream) -> TokenStream {
    crate::error::entry(item)
}

#[proc_macro_attribute]
pub fn throws(args: TokenStream, input: TokenStream) -> TokenStream {
    crate::throws::entry(args, input)
}
