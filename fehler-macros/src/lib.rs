extern crate proc_macro;

mod error;
mod throws;

use proc_macro::*;

synstructure::decl_derive!([Error, attributes(error)] => crate::error::entry);

#[proc_macro_attribute]
pub fn throws(args: TokenStream, input: TokenStream) -> TokenStream {
    crate::throws::entry(args, input)
}
