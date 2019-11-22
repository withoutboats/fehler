extern crate proc_macro;

mod args;
mod throws;

use proc_macro::*;

use args::Args;
use throws::Throws;

#[proc_macro_attribute]
pub fn throws(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    Throws::new(args).fold(input)
}
