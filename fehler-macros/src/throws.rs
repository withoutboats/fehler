// This module implements the Throws folder.
//
// The Throws folder actually visits the item being processed and performs two
// processes:
// - It ok wraps return expressions and inserts terminal Ok(())s.
// - It delegates return type rewriting to the Args type.

use proc_macro::*;
use syn::fold::Fold;

use crate::Args;

pub struct Throws {
    args: Args,
    outer_fn: bool,
}

impl Throws {
    pub fn new(args: Args) -> Throws {
        Throws { args, outer_fn: true }
    }

    pub fn fold(&mut self, input: TokenStream) -> TokenStream {
        if let Ok(item_fn) = syn::parse(input.clone()) {
            let item_fn = self.fold_item_fn(item_fn);
            quote::quote!(#item_fn).into()
        } else if let Ok(method) = syn::parse(input.clone()) {
            let method = self.fold_impl_item_method(method);
            quote::quote!(#method).into()
        } else if let Ok(method) = syn::parse(input.clone()) {
            let method = self.fold_trait_item_method(method);
            quote::quote!(#method).into()
        } else {
            panic!("#[throws] attribute can only be applied to functions and methods")
        }
    }

    fn fold_propane_body(&mut self, mut block: syn::Block, is_async: bool) -> syn::Block {
        use std::mem;
        use syn::{Stmt::Local as L, Local, Expr::Closure as C, ExprClosure};
        let body = if let L(Local { init: Some((_, expr)), .. }) = &mut block.stmts[0] {
            if let C(ExprClosure { body, .. }) = &mut **expr {
                &mut **body
            } else { panic!("body did not have correct structure") }
        } else { panic!("body did not have correct structure") };
        let mut folder = YieldThrows { is_async };
        *body = folder.fold_expr(mem::replace(body, syn::parse_str("{}").unwrap()));
        block
    }

    fn fold_sig_and_body(
        &mut self,
        sig: syn::Signature,
        body: syn::Block,
    ) -> (syn::Signature, syn::Block) {
        if !self.outer_fn { return (sig, body); }

        let output = self.fold_return_type(sig.output);
        let sig = syn::Signature { output, ..sig };

        self.outer_fn = false;

        let body = match self.args.propane_integration {
            true    => self.fold_propane_body(body, is_async(&sig.output)),
            false   => make_fn_block(&self.fold_block(body)),
        };

        (sig, body)
    }
}

impl Fold for Throws {
    fn fold_item_fn(&mut self, i: syn::ItemFn) -> syn::ItemFn {
        let (sig, body) = self.fold_sig_and_body(i.sig, *i.block);
        syn::ItemFn { sig, block: Box::new(body), ..i }
    }

    fn fold_impl_item_method(&mut self, i: syn::ImplItemMethod) -> syn::ImplItemMethod {
        let (sig, block) = self.fold_sig_and_body(i.sig, i.block);
        syn::ImplItemMethod { sig, block, ..i }
    }

    fn fold_trait_item_method(&mut self, mut i: syn::TraitItemMethod) -> syn::TraitItemMethod {
        if !self.outer_fn { return i; }

        let sig = syn::Signature {
            output: self.fold_return_type(i.sig.output),
            ..i.sig
        };

        self.outer_fn = false;

        let default = i.default.take().map(|block| {
            let inner = self.fold_block(block);
            make_fn_block(&inner)
        });


        syn::TraitItemMethod { sig, default, ..i }
    }

    fn fold_expr_closure(&mut self, i: syn::ExprClosure) -> syn::ExprClosure {
        i // TODO
    }

    fn fold_expr_async(&mut self, i: syn::ExprAsync) -> syn::ExprAsync {
        i // TODO
    }

    fn fold_return_type(&mut self, i: syn::ReturnType) -> syn::ReturnType {
        if !self.outer_fn { return i; }
        self.args.ret(i)
    }

    fn fold_expr_return(&mut self, i: syn::ExprReturn) -> syn::ExprReturn {
        let ok = match &i.expr {
            Some(expr)  => ok(expr),
            None        => ok_unit(),
        };
        syn::ExprReturn { expr: Some(Box::new(ok)), ..i }
    }
}

struct YieldThrows {
    is_async: bool,
}

impl Fold for YieldThrows {
    fn fold_expr_yield(&mut self, i: syn::ExprYield) -> syn::ExprYield {
        let ok = match &i.expr {
            Some(expr)  => ok(expr),
            None        => ok_unit(),
        };
        syn::ExprYield { expr: Some(Box::new(ok)), ..i }
    }

    fn fold_expr_macro(&mut self, mut i: syn::ExprMacro) -> syn::ExprMacro {
        let name = &i.mac.path.segments.last().unwrap().ident;
        let replacement = if name == "throw" {
            if self.is_async {
                "async_gen_throw"
            } else {
                "gen_throw"
            }
        } else if name == "async_gen_yield" {
            "async_gen_yield_fehler"
        } else {
            return i;
        };
        i.mac.path = syn::parse_str(&format!("::fehler::{}", replacement)).unwrap();
        i
    }

    fn fold_item(&mut self, i: syn::Item) -> syn::Item {
        i
    }

    fn fold_expr_closure(&mut self, i: syn::ExprClosure) -> syn::ExprClosure {
        i 
    }

    fn fold_expr_async(&mut self, i: syn::ExprAsync) -> syn::ExprAsync {
        i 
    }
}

fn make_fn_block(inner: &syn::Block) -> syn::Block {
    syn::parse2(quote::quote! {{
        let __ret = #inner;

        #[allow(unreachable_code)]
        <_ as ::fehler::__internal::_Succeed>::from_ok(__ret)
    }}).unwrap()
}

fn ok(expr: &syn::Expr) -> syn::Expr {
    syn::parse2(quote::quote!(<_ as ::fehler::__internal::_Succeed>::from_ok(#expr))).unwrap()
}

fn ok_unit() -> syn::Expr {
    syn::parse2(quote::quote!(<_ as ::fehler::__internal::_Succeed>::from_ok(()))).unwrap()
}

fn is_async(ret: &syn::ReturnType) -> bool {
    if let syn::ReturnType::Type(_, ty) = ret {
      if let syn::Type::Paren(syn::TypeParen { elem, .. }) = &**ty {
        if let syn::Type::ImplTrait(ty) = &**elem {
          if let syn::TypeParamBound::Trait(bound) = &ty.bounds[0] {
            let bound = bound.path.segments.last().unwrap();
            return bound.ident == "Stream"
          }
        }
      }
    }

    panic!("return type did not have correct structure")
}
