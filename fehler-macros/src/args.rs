// The Args type parses the arguments to the `#[throws]` macro.
//
// It is also responsible for transforming the return type by injecting
// the return type and the error type into the wrapper type.

use proc_macro2::Span;
use syn::{GenericArgument, Path, PathArguments, ReturnType, Token, Type};
use syn::parse::*;

const WRAPPER_MUST_BE_PATH: &str = "Wrapper type must be a normal path type";

pub struct Args {
    error: Option<Type>,
    wrapper: Option<Type>,
}

impl Args {
    pub fn ret(&mut self, ret: ReturnType) -> ReturnType {
        let (arrow, ret) = match ret {
            ReturnType::Default         => (arrow(), unit()),
            ReturnType::Type(arrow, ty) => (arrow, *ty),
        };
        ReturnType::Type(arrow, Box::new(self.inject_to_wrapper(ret)))

    }

    fn inject_to_wrapper(&mut self, ret: Type) -> Type {
        if let Some(Type::Path(mut wrapper)) = self.wrapper.take() {
            let types = if let Some(error) = self.error.take() {
                vec![ret, error].into_iter().map(GenericArgument::Type)
            } else {
                vec![ret].into_iter().map(GenericArgument::Type)
            };

            match innermost_path_arguments(&mut wrapper.path) {
                args @ &mut PathArguments::None    => {
                    *args = PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Token![<](Span::call_site()),
                        args: types.collect(),
                        gt_token: Token![>](Span::call_site()),
                    });
                }
                PathArguments::AngleBracketed(args) => args.args.extend(types),
                _   => panic!(WRAPPER_MUST_BE_PATH)
            }

            Type::Path(wrapper)
        } else { panic!(WRAPPER_MUST_BE_PATH) }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Args> {
        if input.is_empty() {
            return Ok(Args {
                error: Some(default_error()),
                wrapper: Some(result()),
            })
        }

        let error = match input.peek(Token![as]) {
            true    => None,
            false   => {
                let error = input.parse()?;
                Some(match error {
                    Type::Infer(_)  => default_error(),
                    _               => error,
                })
            }
        };

        let wrapper = Some(match input.parse::<Token![as]>().is_ok() {
            true    => input.parse()?,
            false   => result(),
        });

        Ok(Args { error, wrapper })
    }
}

fn innermost_path_arguments(path: &mut Path) -> &mut PathArguments {
    let arguments = &mut path.segments.last_mut().expect(WRAPPER_MUST_BE_PATH).arguments;
    match arguments {
        PathArguments::None                 => arguments,
        PathArguments::AngleBracketed(args) => {
            match args.args.last_mut() {
                Some(GenericArgument::Type(Type::Path(inner)))  => {
                    innermost_path_arguments(&mut inner.path)
                }
                // Bizarre cases like `#[throw(_ as MyTryType<'a>)]` just not supported currently
                _   => panic!("Certain strange wrapper types not supported"),
            }
        }
        _                                   => panic!(WRAPPER_MUST_BE_PATH)
    }
}

fn arrow() -> syn::token::RArrow {
    Token![->](Span::call_site())
}

fn unit() -> Type {
    syn::parse_str("()").unwrap()
}

fn result() -> Type {
    syn::parse_str("::core::result::Result").unwrap()
}

fn default_error() -> Type {
    syn::parse_str("Error").unwrap()
}
