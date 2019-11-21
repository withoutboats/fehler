extern crate proc_macro;

mod throws;

use proc_macro::*;

#[proc_macro_attribute]
pub fn throws(args: TokenStream, input: TokenStream) -> TokenStream {
    crate::throws::entry(args, input)
}
