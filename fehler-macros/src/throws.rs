use proc_macro::*;
use syn::fold::Fold;

struct Throws {
    ty: syn::Type,
    outer_fn: bool,
}

pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let ty = if args.is_empty() {
        syn::parse2(quote::quote!(::fehler::Exception)).unwrap()
    } else {
        syn::parse(args).unwrap_or_else(|_| {
            panic!("argument to #[throws] attribute must be a type")
        })
    };
    let mut throws = Throws { ty, outer_fn: true };

    if let Ok(item_fn) = syn::parse(input.clone()) {
        let item_fn = throws.fold_item_fn(item_fn);
        quote::quote!(#item_fn).into()
    } else if let Ok(method) = syn::parse(input.clone()) {
        let method = throws.fold_impl_item_method(method);
        quote::quote!(#method).into()
    } else if let Ok(method) = syn::parse(input.clone()) {
        let method = throws.fold_trait_item_method(method);
        quote::quote!(#method).into()
    } else {
        panic!("#[throws] attribute can only be applied to functions and methods")
    }
}

impl Fold for Throws {
    fn fold_item_fn(&mut self, mut i: syn::ItemFn) -> syn::ItemFn {
        if !self.outer_fn { return i; }
        self.outer_fn = false;

        modify_tail(is_unit_fn(&i.sig.output), &mut i.block.stmts);

        let sig = syn::Signature {
            output: self.fold_return_type(i.sig.output),
            ..i.sig
        };


        let block = Box::new(self.fold_block(*i.block));

        syn::ItemFn { sig, block, ..i }
    }

    fn fold_impl_item_method(&mut self, mut i: syn::ImplItemMethod) -> syn::ImplItemMethod {
        if !self.outer_fn { return i; }
        self.outer_fn = false;

        modify_tail(is_unit_fn(&i.sig.output), &mut i.block.stmts);

        let sig = syn::Signature {
            output: self.fold_return_type(i.sig.output),
            ..i.sig
        };

        let block = self.fold_block(i.block);

        syn::ImplItemMethod { sig, block, ..i }
    }

    fn fold_trait_item_method(&mut self, mut i: syn::TraitItemMethod) -> syn::TraitItemMethod {
        if !self.outer_fn { return i; }
        self.outer_fn = false;

        let default = i.default.take().map(|mut block| {
            modify_tail(is_unit_fn(&i.sig.output), &mut block.stmts);
            self.fold_block(block)
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
        let error = &self.ty;
        match i {
            syn::ReturnType::Default        => {
                syn::parse2(quote::quote!(-> ::std::result::Result<(), #error>)).unwrap()
            }
            syn::ReturnType::Type(arrow, ty)    => {
                let result = syn::parse2(quote::quote!(::std::result::Result<#ty, #error>)).unwrap();
                syn::ReturnType::Type(arrow, result)
            }
        }
    }

    fn fold_expr_return(&mut self, i: syn::ExprReturn) -> syn::ExprReturn {
        let ok = match &i.expr {
            Some(expr)  => syn::parse2(quote::quote!(::std::result::Result::Ok(#expr))).unwrap(),
            None        => syn::parse2(quote::quote!(::std::result::Result::Ok(()))).unwrap(),
        };
        syn::ExprReturn { expr: Some(Box::new(ok)), ..i }
    }
}

fn modify_tail(is_unit_fn: bool, stmts: &mut Vec<syn::Stmt>) {
    let last_non_item_stmt = stmts.iter_mut().rev().filter(|s| {
        if let syn::Stmt::Item(_) = s { false } else { true }
    }).next();
    match last_non_item_stmt {
        Some(syn::Stmt::Expr(e)) if is_unit_fn => {
            let new = syn::parse2(quote::quote!(#e;)).unwrap();
            stmts.pop();
            stmts.push(new);
            stmts.push(syn::Stmt::Expr(syn::parse2(quote::quote!(::std::result::Result::Ok(()))).unwrap()));
        }
        Some(syn::Stmt::Expr(e))    => {
            *e = syn::parse2(quote::quote!(::std::result::Result::Ok(#e))).unwrap();
        }
        _ if is_unit_fn             => {
            stmts.push(syn::Stmt::Expr(syn::parse2(quote::quote!(::std::result::Result::Ok(()))).unwrap()));
        }
        _                           => { }
    }
}


fn is_unit_fn(i: &syn::ReturnType) -> bool {
    match i {
        syn::ReturnType::Default        => true,
        syn::ReturnType::Type(_, ty)    => {
            if let syn::Type::Tuple(syn::TypeTuple { elems, .. }) = &**ty {
                elems.is_empty()
            } else { false }
        }
    }
}
