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
}

impl Fold for Throws {
    fn fold_item_fn(&mut self, i: syn::ItemFn) -> syn::ItemFn {
        if !self.outer_fn { return i; }
        self.outer_fn = false;

        let sig = syn::Signature {
            output: self.fold_return_type(i.sig.output),
            ..i.sig
        };


        let inner = self.fold_block(*i.block);
        let block = Box::new(make_fn_block(&inner));

        syn::ItemFn { sig, block, ..i }
    }

    fn fold_impl_item_method(&mut self, i: syn::ImplItemMethod) -> syn::ImplItemMethod {
        if !self.outer_fn { return i; }
        self.outer_fn = false;

        let sig = syn::Signature {
            output: self.fold_return_type(i.sig.output),
            ..i.sig
        };

        let inner = self.fold_block(i.block);
        let block = make_fn_block(&inner);

        syn::ImplItemMethod { sig, block, ..i }
    }

    fn fold_trait_item_method(&mut self, mut i: syn::TraitItemMethod) -> syn::TraitItemMethod {
        if !self.outer_fn { return i; }
        self.outer_fn = false;

        let default = i.default.take().map(|block| {
            let inner = self.fold_block(block);
            make_fn_block(&inner)
        });

        let sig = syn::Signature {
            output: self.fold_return_type(i.sig.output),
            ..i.sig
        };

        syn::TraitItemMethod { sig, default, ..i }
    }

    fn fold_expr_closure(&mut self, i: syn::ExprClosure) -> syn::ExprClosure {
        i // TODO
    }

    fn fold_expr_async(&mut self, i: syn::ExprAsync) -> syn::ExprAsync {
        i // TODO
    }

    fn fold_return_type(&mut self, i: syn::ReturnType) -> syn::ReturnType {
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
