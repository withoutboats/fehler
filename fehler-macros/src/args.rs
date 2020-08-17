// The Args type parses the arguments to the `#[throws]` macro.
//
// It is also responsible for transforming the return type by injecting
// the return type and the error type into the wrapper type.
use std::mem;

use proc_macro2::Span;
use syn::{GenericArgument, Path, PathArguments, ReturnType, Token, Type};
use syn::parse::*;

const WRAPPER_MUST_BE_PATH: &str = "Wrapper type must be a normal path type";

pub struct Args {
    error: Option<Type>,
    wrapper: Option<Type>,
    pub propane_integration: bool,
}

impl Args {
    pub fn ret(&mut self, mut ret: ReturnType) -> ReturnType {
        if self.propane_integration {
            self.propane_ret(&mut ret);
            ret
        } else {
            let (arrow, mut ret) = match ret {
                ReturnType::Default         => (arrow(), unit()),
                ReturnType::Type(arrow, ty) => (arrow, *ty),
            };
            self.inject_to_wrapper(&mut ret);
            ReturnType::Type(arrow, Box::new(ret))
        }
    }

    fn propane_ret(&mut self, ret: &mut ReturnType) {
        if let syn::ReturnType::Type(_, ty) = ret {
          if let syn::Type::Paren(syn::TypeParen { elem, .. }) = &mut **ty {
            if let syn::Type::ImplTrait(ty) = &mut **elem {
              if let syn::TypeParamBound::Trait(bound) = &mut ty.bounds[0] {
                let bound = bound.path.segments.last_mut().unwrap();
                if let syn::PathArguments::AngleBracketed(args) = &mut bound.arguments {
                  if let syn::GenericArgument::Binding(binding) = &mut args.args[0] {
                    let ty = &mut binding.ty;
                    self.inject_to_wrapper(ty);
                  }
                }
              }
            }
          }
        }
    }

    fn inject_to_wrapper(&mut self, ret: &mut Type) {
        let ty = mem::replace(ret, Type::Never(syn::TypeNever { bang_token: Default::default() }));
        let ty = if let Some(Type::Path(mut wrapper)) = self.wrapper.take() {
            let types = if let Some(error) = self.error.take() {
                vec![ty, error].into_iter().map(GenericArgument::Type)
            } else {
                vec![ty].into_iter().map(GenericArgument::Type)
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
        } else { panic!(WRAPPER_MUST_BE_PATH) };
        *ret = ty;
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Args> {
        let propane_integration = input.peek(Token![@]);

        if propane_integration {
            input.parse::<syn::token::At>().unwrap();
            let ident: syn::Ident = input.parse()?;
            assert_eq!(ident, "__internal_propane_integration");
        };

        if input.is_empty() {
            return Ok(Args {
                error: Some(default_error()),
                wrapper: Some(result()),
                propane_integration,
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

        Ok(Args { error, wrapper, propane_integration })
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
