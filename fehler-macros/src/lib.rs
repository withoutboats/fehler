extern crate proc_macro;

mod error;
mod throws;

use proc_macro::*;

/*
#[proc_macro_derive(Error)]
pub fn derive_error(input: TokenStream) -> TokenStream {
    crate::error::entry(input)
}
*/

synstructure::decl_derive!([Error, attributes(error)] => crate::error::entry);

#[proc_macro_attribute]
pub fn throws(args: TokenStream, input: TokenStream) -> TokenStream {
    crate::throws::entry(args, input)
}
